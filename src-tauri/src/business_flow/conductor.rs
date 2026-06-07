//! Business Flow Conductor — thin Tauri command wrappers
//!
//! ADR-001: Rust only handles system capabilities (DB writes, event emission).
//! All DAG logic (topological sort, gateway routing, degradation, node dispatch)
//! now lives in the TypeScript DagEngine (src/orchestration/DagEngine.ts).
//!
//! These commands provide the IPC entry points that the TS-side DagEngine calls:
//!   - bf_run_flow  → creates a DB run record, returns { runId, flowJson, outputDir }
//!   - bf_abort_run → marks a run as aborted in the DB

use crate::business_flow::handler;
use crate::business_flow::model::RunInput;

/// Start a flow run — creates the DB run record and returns the data
/// that the TypeScript DagEngine needs to execute the flow.
///
/// The actual DAG execution happens in TypeScript (DagEngine.execute()),
/// which calls back into Rust for system capabilities:
///   - bf_update_run_status (status transitions)
///   - agent_execute_turn (LLM calls)
#[tauri::command]
pub async fn bf_run_flow(
    flow_id: String,
    material_paths: Vec<String>,
    output_dir_override: Option<String>,
) -> Result<serde_json::Value, String> {
    let flow = handler::bf_get_flow(flow_id.clone()).await?;
    let output_dir = output_dir_override.unwrap_or(flow.output_dir);

    let run = handler::create_run(RunInput {
        flow_id: flow.id.clone(),
        triggered_by: "manual".into(),
        material_paths: serde_json::to_string(&material_paths).unwrap_or_default(),
    })
    .await?;

    handler::update_run_status(&run.id, "running", None).await?;

    Ok(serde_json::json!({
        "runId": run.id,
        "flowJson": flow.flow_json,
        "outputDir": output_dir,
    }))
}

/// Abort a running flow. Only updates DB status — the TypeScript DagEngine
/// handles the actual cancellation via AbortController.
#[tauri::command]
pub async fn bf_abort_run(run_id: String) -> Result<(), String> {
    let run = handler::bf_get_run(run_id.clone()).await?;

    if run.status != "running" && run.status != "pending" {
        return Err(format!("Run is not running (status: {})", run.status));
    }

    handler::update_run_status(&run_id, "aborted", None).await?;
    Ok(())
}
