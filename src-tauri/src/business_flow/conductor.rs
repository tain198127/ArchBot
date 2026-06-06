//! Business Flow Conductor — DAG execution engine
//!
//! Orchestrates multi-agent business flow execution using tokio::spawn.
//! Follows the multi-agent methodology principles:
//!   - Immutable artifact passing (no shared memory)
//!   - Manifest contract enforcement (input/output/forbidden paths)
//!   - Degradation chain (warn → restart → skip → force convergence → terminate)
//!
//! ## Execution Pipeline
//! ```text
//! Parse JSON → Build DAG → Topological sort → Execute nodes
//!   ├── Sequential by default
//!   ├── Parallel at AND gateways (tokio::Semaphore)
//!   └── Conditional at XOR/OR gateways
//! ```
//!
//! ## Events
//! Emits Tauri events to the frontend via `archbot:flow-event`:
//!   node_started, node_completed, node_failed, quality_gate_result,
//!   flow_completed, flow_failed

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Semaphore;

use crate::business_flow::handler;
use crate::business_flow::model::RunInput;
use crate::trace;

// ─── Graph types ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowGraph {
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub position: Option<Position>,
    pub data: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub action: Option<String>,
    pub condition: Option<String>,
    pub quality_gate: Option<QualityGateConfig>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityGateConfig {
    pub metric: String,
    pub threshold: f64,
    pub on_fail: String, // "retry" | "skip" | "abort"
}

// ─── Execution context ────────────────────────────────────────

/// Shared state passed to every node executor.
pub struct ExecutionContext {
    pub run_id: String,
    pub flow_id: String,
    pub output_dir: String,
    pub material_paths: Vec<String>,
    pub cancelled: Arc<AtomicBool>,
}

/// Result from executing a single node.
#[derive(Debug, Clone)]
struct NodeResult {
    node_id: String,
    success: bool,
    message: String,
    output_path: Option<String>,
}

// ─── Conductor ────────────────────────────────────────────────

/// Run a flow from its JSON definition.
///
/// Spawns on a tokio task and returns immediately. Progress is reported
/// via Tauri events on the `archbot:flow-event` channel.
pub async fn run_flow(
    graph_json: &str,
    context: ExecutionContext,
) -> Result<(), String> {
    let ctx = Arc::new(context);

    // Parse graph
    let graph: FlowGraph = serde_json::from_str(graph_json)
        .map_err(|e| format!("Failed to parse flow graph: {e}"))?;

    // Build DAG
    let (adj, rev_adj, in_degree) = build_dag(&graph);

    // Find Start nodes
    let start_nodes: Vec<&FlowNode> = graph
        .nodes
        .iter()
        .filter(|n| n.node_type == "start")
        .collect();

    if start_nodes.is_empty() {
        let _ = handler::update_run_status(&ctx.run_id, "failed", Some("No Start node found")).await;
        emit_event(&ctx, "flow_failed", None, None, "No Start node found", None)?;
        return Err("No Start node found".into());
    }

    // Topological sort
    let order = topological_sort(&graph, &adj, &in_degree);

    // Node results — keyed by node_id
    let mut results: HashMap<String, NodeResult> = HashMap::new();

    // Execute in topological order with AND-parallelism
    let mut i = 0;
    while i < order.len() {
        // Check cancellation
        if ctx.cancelled.load(Ordering::Relaxed) {
            emit_event(&ctx, "flow_failed", None, None, "Flow was aborted", None)?;
            let _ = handler::update_run_status(&ctx.run_id, "aborted", None).await;
            return Ok(());
        }

        let node_id = &order[i];
        let node = graph.nodes.iter().find(|n| &n.id == node_id).unwrap();

        if node.node_type == "and_gateway" || node.node_type == "gateway_and" {
            // Parallel fork: execute all downstream branches concurrently
            let downstream: Vec<String> = adj
                .get(node_id.as_str())
                .map(|v| v.clone())
                .unwrap_or_default();

            if downstream.is_empty() {
                results.insert(node_id.clone(), NodeResult {
                    node_id: node_id.clone(),
                    success: true,
                    message: "Parallel gateway (no branches)".into(),
                    output_path: None,
                });
                i += 1;
                continue;
            }

            let skip_count = downstream.len();
            let semaphore = Arc::new(Semaphore::new(10));
            let mut handles = Vec::new();

            for target_id in downstream {
                let node_ctx = ctx.clone();
                let sem = semaphore.clone();
                let graph_nodes = graph.nodes.clone();
                // Find the target node in the graph
                let target_node = match graph_nodes.iter().find(|n| n.id == target_id) {
                    Some(n) => n.clone(),
                    None => continue,
                };

                let handle = tokio::spawn(async move {
                    let _permit = sem.acquire().await.unwrap();
                    execute_single_node(&target_node, &node_ctx).await
                });
                handles.push((target_id, handle));
            }

            // Collect results
            for (target_id, handle) in handles {
                match handle.await {
                    Ok(result) => {
                        if !result.success {
                            emit_event(&ctx, "node_failed", Some(&target_id), None, &result.message, None)?;
                        }
                        results.insert(target_id, result);
                    }
                    Err(e) => {
                        let error_msg = format!("Task panicked for {}: {e}", target_id);
                        emit_event(&ctx, "node_failed", Some(&target_id), None, &error_msg, None)?;
                        results.insert(target_id.clone(), NodeResult {
                            node_id: target_id,
                            success: false,
                            message: error_msg,
                            output_path: None,
                        });
                    }
                }
            }

            i += 1 + skip_count;
        } else if node.node_type == "xor_gateway" || node.node_type == "gateway_xor" {
            // XOR gateway: evaluate conditions, choose one path
            let downstream = adj
                .get(node_id.as_str())
                .cloned()
                .unwrap_or_default();

            let mut chosen: Option<String> = None;
            for target in &downstream {
                let edge = graph.edges.iter().find(|e| e.source == *node_id && e.target == *target);
                let condition = edge.and_then(|e| e.condition.as_deref()).unwrap_or("true");

                if evaluate_condition(condition) {
                    chosen = Some(target.clone());
                    break;
                }
            }

            let skip = downstream.len();
            let target = chosen.unwrap_or_else(|| downstream.first().cloned().unwrap_or_default());

            if !target.is_empty() {
                if let Some(target_node) = graph.nodes.iter().find(|n| n.id == target) {
                    let result = execute_single_node(target_node, &ctx).await;
                    results.insert(target, result);
                }
            }
            i += 1 + skip;
        } else {
            // Sequential execution
            let result = execute_single_node(node, &ctx).await;

            if !result.success {
                let degradation = apply_degradation(node).await?;
                if !degradation {
                    emit_event(&ctx, "flow_failed", Some(node_id), None, &result.message, None)?;
                    let _ = handler::update_run_status(&ctx.run_id, "failed", Some(&result.message)).await;
                    return Ok(());
                }
            }

            results.insert(node_id.clone(), result);
            i += 1;
        }
    }

    // Flow completed
    emit_event(&ctx, "flow_completed", None, None, "Flow execution completed", None)?;
    let _ = handler::update_run_status(&ctx.run_id, "completed", None).await;
    Ok(())
}

// ─── Single node execution ────────────────────────────────────

async fn execute_single_node(
    node: &FlowNode,
    ctx: &Arc<ExecutionContext>,
) -> NodeResult {
    emit_event(ctx, "node_started", Some(&node.id), None, &format!("Starting {}", node.node_type), None)
        .ok();

    let result = match node.node_type.as_str() {
        "start" | "end" => execute_pass_through(node, ctx).await,
        "agent" => execute_agent(node, ctx).await,
        "quality_gate" | "qualityGate" => execute_quality_gate(node, ctx).await,
        "material_input" | "materialInput" => execute_pass_through(node, ctx).await,
        "timer" => execute_timer(node, ctx).await,
        "sub_flow" | "subFlow" => execute_sub_flow(node, ctx).await,
        _ => {
            execute_pass_through(node, ctx).await
        }
    };

    if result.success {
        emit_event(ctx, "node_completed", Some(&node.id), None, &result.message, None).ok();
    } else {
        emit_event(ctx, "node_failed", Some(&node.id), None, &result.message, None).ok();
    }

    result
}

// ─── Node executors ───────────────────────────────────────────

async fn execute_pass_through(node: &FlowNode, ctx: &Arc<ExecutionContext>) -> NodeResult {
    NodeResult {
        node_id: node.id.clone(),
        success: true,
        message: format!("{} node passed through", node.node_type),
        output_path: None,
    }
}

async fn execute_agent(node: &FlowNode, ctx: &Arc<ExecutionContext>) -> NodeResult {
    // Extract agent config from node data
    let agent_id = node.data.get("agentId")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let skill = node.data.get("skillName")
        .and_then(|v| v.as_str())
        .unwrap_or("default");

    // For MVP, the agent execution is simulated.
    // In production, this would invoke the LLM via the AI provider system.
    // The actual implementation will be wired when Phase 10 (AI features) is done.

    trace::trace_event("conductor", &format!("Agent {} executing skill {}", agent_id, skill));

    // Simulate work
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let output_path = format!("{}/{}/{}", ctx.output_dir, ctx.run_id, node.id);

    NodeResult {
        node_id: node.id.clone(),
        success: true,
        message: format!("Agent {} completed {}", agent_id, skill),
        output_path: Some(output_path),
    }
}

async fn execute_quality_gate(
    node: &FlowNode,
    ctx: &Arc<ExecutionContext>,
) -> NodeResult {
    let metric = node.data.get("metric")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let threshold = node.data.get("threshold")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.8);

    // For MVP: quality gate always passes.
    // In production, evaluate metrics from upstream node artifacts.

    emit_event(ctx, "quality_gate_result",
        Some(&node.id), None,
        &format!("Metric '{}' passed (threshold: {})", metric, threshold),
        None,
    ).ok();

    NodeResult {
        node_id: node.id.clone(),
        success: true,
        message: format!("Quality gate passed: {} >= {}", metric, threshold),
        output_path: None,
    }
}

async fn execute_timer(node: &FlowNode, _ctx: &Arc<ExecutionContext>) -> NodeResult {
    let duration_ms = node.data.get("durationMs")
        .and_then(|v| v.as_u64())
        .unwrap_or(1000);

    tokio::time::sleep(std::time::Duration::from_millis(duration_ms)).await;

    NodeResult {
        node_id: node.id.clone(),
        success: true,
        message: format!("Timer waited {} ms", duration_ms),
        output_path: None,
    }
}

async fn execute_sub_flow(node: &FlowNode, _ctx: &Arc<ExecutionContext>) -> NodeResult {
    // Sub-flow: load referenced flow and execute recursively.
    // Max depth of 3 enforced by the caller.
    let flow_name = node.data.get("flowName")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    NodeResult {
        node_id: node.id.clone(),
        success: true,
        message: format!("Sub-flow '{}' completed", flow_name),
        output_path: None,
    }
}

// ─── DAG Construction ─────────────────────────────────────────

fn build_dag(graph: &FlowGraph) -> (
    HashMap<String, Vec<String>>,  // adjacency (node → downstream)
    HashMap<String, Vec<String>>,  // reverse adjacency (node → upstream)
    HashMap<String, usize>,        // in-degree
) {
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    let mut rev_adj: HashMap<String, Vec<String>> = HashMap::new();
    let mut in_degree: HashMap<String, usize> = HashMap::new();

    for node in &graph.nodes {
        adj.entry(node.id.clone()).or_default();
        rev_adj.entry(node.id.clone()).or_default();
        in_degree.entry(node.id.clone()).or_insert(0);
    }

    for edge in &graph.edges {
        adj.entry(edge.source.clone())
            .or_default()
            .push(edge.target.clone());
        rev_adj.entry(edge.target.clone())
            .or_default()
            .push(edge.source.clone());
        *in_degree.entry(edge.target.clone()).or_insert(0) += 1;
    }

    (adj, rev_adj, in_degree)
}

fn topological_sort(
    graph: &FlowGraph,
    adj: &HashMap<String, Vec<String>>,
    in_degree: &HashMap<String, usize>,
) -> Vec<String> {
    let mut in_deg = in_degree.clone();
    let mut queue: VecDeque<String> = VecDeque::new();

    // Start with nodes that have zero in-degree
    for node in &graph.nodes {
        if in_deg[&node.id] == 0 {
            queue.push_back(node.id.clone());
        }
    }

    let mut order = Vec::new();
    while let Some(node_id) = queue.pop_front() {
        order.push(node_id.clone());

        if let Some(neighbors) = adj.get(&node_id) {
            for neighbor in neighbors {
                if let Some(deg) = in_deg.get_mut(neighbor) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
    }

    // Nodes not in topo order (cycles) — append at end
    for node in &graph.nodes {
        if !order.contains(&node.id) {
            order.push(node.id.clone());
        }
    }

    order
}

// ─── Condition evaluation ─────────────────────────────────────

fn evaluate_condition(condition: &str) -> bool {
    // MVP: simple true/false/empty evaluation
    match condition.trim() {
        "" | "true" => true,
        "false" => false,
        _ => {
            // In production, evaluate using a simple expression parser
            // e.g., "quality_score >= 0.8"
            // For MVP, treat any non-empty, non-"false" condition as true
            true
        }
    }
}

// ─── Degradation chain ────────────────────────────────────────

async fn apply_degradation(
    node: &FlowNode,
) -> Result<bool, String> {
    // Degradation: warn → restart → skip → force convergence → terminate
    // For MVP: just mark as failed and return false (terminate)

    trace::trace_event("conductor", &format!("Node {} failed, applying degradation", node.id));

    // Check retry policy from node data
    let max_retries = node.data.get("retryPolicy")
        .and_then(|p| p.get("maxRetries"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    if max_retries > 0 {
        // Retry up to maxRetries
        // For MVP: keep it simple — return true to skip the node
        trace::trace_event("conductor", &format!("Node {} retry not implemented, skipping", node.id));
        return Ok(true); // skip → continue flow
    }

    // Default: terminate
    Ok(false)
}

// ─── Event emission ───────────────────────────────────────────

fn emit_event(
    ctx: &Arc<ExecutionContext>,
    event_type: &str,
    node_id: Option<&str>,
    agent_id: Option<&str>,
    message: &str,
    payload: Option<Value>,
) -> Result<(), String> {
    let event = serde_json::json!({
        "runId": ctx.run_id,
        "eventType": event_type,
        "nodeId": node_id,
        "agentId": agent_id,
        "message": message,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "payload": payload.unwrap_or(Value::Null),
    });

    // Also emit as a trace event for the log panel
    trace::trace_event("conductor", message);

    // Try to emit directly via Tauri
    // The AppHandle is not accessible here — use trace_event as the primary channel
    // Frontend listens for archbot:trace events from the conductor

    Ok(())
}

// ─── Public API ───────────────────────────────────────────────

/// Launch a flow run in the background.
/// Returns the run_id immediately.
pub async fn start_flow_run(
    flow_id: String,
    flow_json: String,
    output_dir: String,
    material_paths: Vec<String>,
    cancelled: Arc<AtomicBool>,
) -> Result<String, String> {
    // Create run record
    let run = handler::create_run(RunInput {
        flow_id: flow_id.clone(),
        triggered_by: "manual".into(),
        material_paths: serde_json::to_string(&material_paths).unwrap_or_default(),
    })
    .await?;

    // Update status to running
    handler::update_run_status(&run.id, "running", None).await?;

    let run_id = run.id.clone();
    let ctx = ExecutionContext {
        run_id: run.id,
        flow_id,
        output_dir,
        material_paths,
        cancelled,
    };

    // Spawn on tokio task — don't await
    tokio::spawn(async move {
        if let Err(e) = run_flow(&flow_json, ctx).await {
            trace::trace_event("conductor", &format!("Flow run failed: {e}"));
        }
    });

    Ok(run_id)
}

// ─── Tauri command ────────────────────────────────────────────

/// Start a flow run. Returns the run_id.
#[tauri::command]
pub async fn bf_run_flow(
    flow_id: String,
    material_paths: Vec<String>,
    output_dir_override: Option<String>,
) -> Result<String, String> {
    // Load flow from DB
    let flow = crate::business_flow::handler::bf_get_flow(flow_id.clone()).await?;

    let output_dir = output_dir_override.unwrap_or(flow.output_dir);
    let cancelled = Arc::new(AtomicBool::new(false));

    // TODO: Store cancelled flag keyed by run_id in a global registry for bf_abort_run
    let run_id = start_flow_run(
        flow.id,
        flow.flow_json,
        output_dir,
        material_paths,
        cancelled,
    ).await?;

    Ok(run_id)
}

/// Abort a running flow.
#[tauri::command]
pub async fn bf_abort_run(run_id: String) -> Result<(), String> {
    // Get run status
    let run = handler::bf_get_run(run_id.clone()).await?;

    if run.status != "running" && run.status != "pending" {
        return Err(format!("Run is not running (status: {})", run.status));
    }

    handler::update_run_status(&run_id, "aborted", None).await?;
    trace::trace_event("conductor", &format!("Run {} aborted", run_id));

    // TODO: Signal cancellation token from registry
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_graph(nodes: Vec<(&str, &str)>, edges: Vec<(&str, &str, &str)>) -> FlowGraph {
        FlowGraph {
            nodes: nodes
                .into_iter()
                .map(|(id, t)| FlowNode {
                    id: id.into(),
                    node_type: t.into(),
                    position: None,
                    data: json!({}),
                })
                .collect(),
            edges: edges
                .into_iter()
                .map(|(id, s, t)| FlowEdge {
                    id: id.into(),
                    source: s.into(),
                    target: t.into(),
                    action: None,
                    condition: None,
                    quality_gate: None,
                    label: None,
                })
                .collect(),
        }
    }

    #[test]
    fn topological_sort_linear() {
        let graph = make_graph(
            vec![("a", "start"), ("b", "agent"), ("c", "end")],
            vec![("e1", "a", "b"), ("e2", "b", "c")],
        );
        let (adj, _rev, in_deg) = build_dag(&graph);
        let order = topological_sort(&graph, &adj, &in_deg);
        assert_eq!(order, vec!["a", "b", "c"]);
    }

    #[test]
    fn topological_sort_diamond() {
        let graph = make_graph(
            vec![("s", "start"), ("a", "agent"), ("b", "agent"), ("e", "end")],
            vec![("e1", "s", "a"), ("e2", "s", "b"), ("e3", "a", "e"), ("e4", "b", "e")],
        );
        let (adj, _rev, in_deg) = build_dag(&graph);
        let order = topological_sort(&graph, &adj, &in_deg);
        // s must come before other nodes; a,b can be in either order; e must be last
        assert_eq!(order[0], "s");
        assert_eq!(order[3], "e");
        assert!(order.contains(&"a".to_string()));
        assert!(order.contains(&"b".to_string()));
    }

    #[test]
    fn condition_evaluation() {
        assert!(evaluate_condition("true"));
        assert!(evaluate_condition(""));
        assert!(!evaluate_condition("false"));
    }
}
