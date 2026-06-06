//! Business Flow — static validation logic
//!
//! Validates flow definitions for structural correctness:
//! - Exactly one Start node
//! - At least one End node
//! - No cycles (DFS-based cycle detection)
//! - No disconnected nodes (all reachable from Start)
//! - No orphan edges (both endpoints exist)

use crate::business_flow::model::ValidationIssue;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Parsed graph for validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowGraph {
    pub nodes: Vec<NodeEntry>,
    pub edges: Vec<EdgeEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgeEntry {
    pub id: String,
    pub source: String,
    pub target: String,
}

/// Run all static validation checks on a flow graph.
///
/// Returns a list of issues (errors and warnings). An empty list means valid.
pub fn validate_flow(graph: &FlowGraph) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // Collect node IDs and types
    let node_ids: HashSet<&str> = graph.nodes.iter().map(|n| n.id.as_str()).collect();
    let start_nodes: Vec<&str> = graph
        .nodes
        .iter()
        .filter(|n| n.node_type == "start")
        .map(|n| n.id.as_str())
        .collect();
    let end_nodes: Vec<&str> = graph
        .nodes
        .iter()
        .filter(|n| n.node_type == "end")
        .map(|n| n.id.as_str())
        .collect();

    // Check: exactly one Start node
    match start_nodes.len() {
        0 => issues.push(ValidationIssue {
            severity: "error".into(),
            message: "Flow must have exactly one Start node".into(),
            node_id: None,
            edge_id: None,
        }),
        1 => {}
        n => issues.push(ValidationIssue {
            severity: "error".into(),
            message: format!("Flow must have exactly one Start node, found {n}"),
            node_id: None,
            edge_id: None,
        }),
    }

    // Check: at least one End node
    if end_nodes.is_empty() {
        issues.push(ValidationIssue {
            severity: "error".into(),
            message: "Flow must have at least one End node".into(),
            node_id: None,
            edge_id: None,
        });
    }

    // Check: orphan edges (source or target doesn't exist)
    for edge in &graph.edges {
        if !node_ids.contains(edge.source.as_str()) {
            issues.push(ValidationIssue {
                severity: "error".into(),
                message: format!("Edge '{}' references non-existent source '{}'", edge.id, edge.source),
                node_id: None,
                edge_id: Some(edge.id.clone()),
            });
        }
        if !node_ids.contains(edge.target.as_str()) {
            issues.push(ValidationIssue {
                severity: "error".into(),
                message: format!("Edge '{}' references non-existent target '{}'", edge.id, edge.target),
                node_id: None,
                edge_id: Some(edge.id.clone()),
            });
        }
    }

    // Check: disconnected nodes (not reachable from Start via BFS)
    if start_nodes.len() == 1 {
        let start_id = start_nodes[0];
        let adj = build_adjacency_map(graph);
        let reachable = bfs_reachable(start_id, &adj);
        for node in &graph.nodes {
            if !reachable.contains(&node.id) && node.node_type != "start" {
                issues.push(ValidationIssue {
                    severity: "warning".into(),
                    message: format!("Node '{}' is not reachable from Start", node.id),
                    node_id: Some(node.id.clone()),
                    edge_id: None,
                });
            }
        }
    }

    // Check: cycles (DFS with recursion stack)
    if let Some(cycle_node) = detect_cycle(graph) {
        issues.push(ValidationIssue {
            severity: "error".into(),
            message: format!("Flow contains a cycle involving node '{}'", cycle_node),
            node_id: Some(cycle_node),
            edge_id: None,
        });
    }

    issues
}

/// Build adjacency map from edges (node_id → [downstream node_ids]).
fn build_adjacency_map(graph: &FlowGraph) -> HashMap<&str, Vec<&str>> {
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in &graph.edges {
        adj.entry(edge.source.as_str())
            .or_default()
            .push(edge.target.as_str());
    }
    adj
}

/// BFS to find all nodes reachable from `start`.
fn bfs_reachable(start: &str, adj: &HashMap<&str, Vec<&str>>) -> HashSet<String> {
    let mut visited = HashSet::new();
    let mut queue = vec![start];
    while let Some(current) = queue.pop() {
        if visited.contains(current) {
            continue;
        }
        visited.insert(current.to_string());
        if let Some(neighbors) = adj.get(current) {
            for &next in neighbors {
                if !visited.contains(next) {
                    queue.push(next);
                }
            }
        }
    }
    visited
}

/// Detect cycles using DFS with a recursion stack.
/// Returns the ID of a node involved in a cycle, or None.
fn detect_cycle(graph: &FlowGraph) -> Option<String> {
    let adj = build_adjacency_map(graph);
    let node_ids: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
    let mut visited: HashSet<String> = HashSet::new();
    let mut rec_stack: HashSet<String> = HashSet::new();

    for node_id in &node_ids {
        if !visited.contains(node_id) {
            if let Some(cycle) = dfs_cycle(node_id, &adj, &mut visited, &mut rec_stack) {
                return Some(cycle);
            }
        }
    }
    None
}

fn dfs_cycle(
    node: &str,
    adj: &HashMap<&str, Vec<&str>>,
    visited: &mut HashSet<String>,
    rec_stack: &mut HashSet<String>,
) -> Option<String> {
    visited.insert(node.to_string());
    rec_stack.insert(node.to_string());

    if let Some(neighbors) = adj.get(node) {
        for &next in neighbors {
            if !visited.contains(next) {
                if let Some(cycle) = dfs_cycle(next, adj, visited, rec_stack) {
                    return Some(cycle);
                }
            } else if rec_stack.contains(next) {
                return Some(next.to_string());
            }
        }
    }

    rec_stack.remove(node);
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_graph(nodes: Vec<(&str, &str)>, edges: Vec<(&str, &str, &str)>) -> FlowGraph {
        FlowGraph {
            nodes: nodes
                .into_iter()
                .map(|(id, t)| NodeEntry {
                    id: id.into(),
                    node_type: t.into(),
                })
                .collect(),
            edges: edges
                .into_iter()
                .map(|(id, s, t)| EdgeEntry {
                    id: id.into(),
                    source: s.into(),
                    target: t.into(),
                })
                .collect(),
        }
    }

    #[test]
    fn valid_linear_flow() {
        let graph = make_graph(
            vec![("s", "start"), ("a", "agent"), ("e", "end")],
            vec![("e1", "s", "a"), ("e2", "a", "e")],
        );
        let issues = validate_flow(&graph);
        assert!(issues.is_empty(), "expected no issues, got: {:?}", issues);
    }

    #[test]
    fn missing_start_node() {
        let graph = make_graph(
            vec![("a", "agent"), ("e", "end")],
            vec![("e1", "a", "e")],
        );
        let issues = validate_flow(&graph);
        assert!(issues.iter().any(|i| i.message.contains("Start")));
    }

    #[test]
    fn missing_end_node() {
        let graph = make_graph(
            vec![("s", "start"), ("a", "agent")],
            vec![("e1", "s", "a")],
        );
        let issues = validate_flow(&graph);
        assert!(issues.iter().any(|i| i.message.contains("End")));
    }

    #[test]
    fn detects_cycle() {
        let graph = make_graph(
            vec![("s", "start"), ("a", "agent"), ("b", "agent"), ("e", "end")],
            vec![
                ("e1", "s", "a"),
                ("e2", "a", "b"),
                ("e3", "b", "a"), // cycle
                ("e4", "b", "e"),
            ],
        );
        let issues = validate_flow(&graph);
        assert!(issues.iter().any(|i| i.message.contains("cycle")));
    }

    #[test]
    fn disconnected_node_warning() {
        let graph = make_graph(
            vec![("s", "start"), ("a", "agent"), ("orphan", "agent"), ("e", "end")],
            vec![("e1", "s", "a"), ("e2", "a", "e")],
        );
        let issues = validate_flow(&graph);
        assert!(issues.iter().any(|i| i.message.contains("orphan") && i.severity == "warning"));
    }

    #[test]
    fn orphan_edge_detection() {
        let graph = make_graph(
            vec![("s", "start"), ("e", "end")],
            vec![("e1", "s", "ghost")], // target doesn't exist
        );
        let issues = validate_flow(&graph);
        assert!(issues.iter().any(|i| i.message.contains("non-existent target")));
    }
}
