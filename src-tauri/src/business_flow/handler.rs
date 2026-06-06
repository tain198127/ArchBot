//! Business Flow — Tauri IPC commands
//!
//! All commands use the generic `db::*` dispatch layer, passing "local" as
//! db_type to hit SQLite. This matches the project convention established
//! by `db::db_find_all`, `db::db_insert`, etc.

use crate::business_flow::model::{
    self, FlowInput, FlowRunRow, FlowRow, FlowSummary, RunInput, ValidationResult,
};
use crate::business_flow::validation::{validate_flow, FlowGraph};
use crate::db;
use crate::now_iso;
use serde_json::json;

// ═══════════════════════════════════════════════════════════════
// Flow CRUD Commands
// ═══════════════════════════════════════════════════════════════

/// List all business flows (summary only, no graph payload).
#[tauri::command]
pub async fn bf_list_flows() -> Result<Vec<FlowSummary>, String> {
    let result = db::db_find_all(
        "business_flows".into(),
        db::QueryParams {
            order_by: vec![db::OrderBy {
                field: "updated_at".into(),
                descending: true,
            }],
            ..Default::default()
        },
        "local".into(),
    )
    .await?;

    let summaries: Vec<FlowSummary> = result
        .rows
        .iter()
        .map(|row| FlowSummary {
            id: row
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .into(),
            name: row
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .into(),
            description: row
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .into(),
            flow_type: row
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("custom")
                .into(),
            // SQLite stores booleans as INTEGER; handle both Bool and Number variants
            published: row.get("published").map(|v| {
                match v {
                    serde_json::Value::Bool(b) => *b,
                    serde_json::Value::Number(n) => n.as_i64().map(|i| i != 0).unwrap_or(false),
                    _ => false,
                }
            }).unwrap_or(false),
            scenario_bindings: row
                .get("scenario_bindings")
                .and_then(|v| v.as_str())
                .unwrap_or("[]")
                .into(),
            version: row.get("version").and_then(|v| v.as_i64()).unwrap_or(1),
            updated_at: row
                .get("updated_at")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .into(),
        })
        .collect();

    Ok(summaries)
}

/// Get a single flow by ID (full graph payload).
#[tauri::command]
pub async fn bf_get_flow(id: String) -> Result<FlowRow, String> {
    let row = db::db_find_by_id("business_flows".into(), id, "local".into())
        .await?
        .ok_or_else(|| "Flow not found".to_string())?;

    model::flow_row_from_db(&row)
}

/// Create a new business flow.
#[tauri::command]
pub async fn bf_create_flow(input: FlowInput) -> Result<FlowRow, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = now_iso();

    let data = json!({
        "id": id,
        "name": input.name,
        "description": input.description,
        "type": input.flow_type,
        "published": false,
        "flow_json": input.flow_json,
        "output_dir": input.output_dir,
        "output_filename_pattern": input.output_filename_pattern,
        "output_extension": input.output_extension,
        "scenario_bindings": input.scenario_bindings,
        "yaml_export": null,
        "created_at": now,
        "updated_at": now,
        "published_at": null,
        "version": 1,
    });

    let data_map: db::DbRow = serde_json::from_value(data)
        .map_err(|e| format!("failed to serialize flow data: {e}"))?;

    db::db_insert("business_flows".into(), data_map, "local".into()).await?;

    // Fetch back to return the full row
    bf_get_flow(id).await
}

/// Update an existing flow with optimistic locking.
#[tauri::command]
pub async fn bf_update_flow(id: String, input: FlowInput, expected_version: i64) -> Result<FlowRow, String> {
    let existing = bf_get_flow(id.clone()).await?;

    if existing.version != expected_version {
        return Err(format!(
            "Optimistic lock conflict: expected version {}, found {}",
            expected_version, existing.version
        ));
    }

    if existing.published {
        return Err("Cannot modify a published flow. Copy it first.".into());
    }
    if existing.flow_type == "builtin" {
        return Err("Built-in flows are read-only. Copy it to customize.".into());
    }

    let now = now_iso();
    let new_version = expected_version + 1;

    let data = json!({
        "name": input.name,
        "description": input.description,
        "flow_json": input.flow_json,
        "output_dir": input.output_dir,
        "output_filename_pattern": input.output_filename_pattern,
        "output_extension": input.output_extension,
        "scenario_bindings": input.scenario_bindings,
        "updated_at": now,
        "version": new_version,
    });

    let data_map: db::DbRow = serde_json::from_value(data)
        .map_err(|e| format!("failed to serialize update data: {e}"))?;

    db::db_update("business_flows".into(), id.clone(), data_map, "local".into()).await?;

    bf_get_flow(id).await
}

/// Delete a flow (reject for built-in flows, cascade runs + artifacts).
#[tauri::command]
pub async fn bf_delete_flow(id: String) -> Result<(), String> {
    let existing = bf_get_flow(id.clone()).await?;

    if existing.flow_type == "builtin" {
        return Err("Built-in flows cannot be deleted".into());
    }

    // Cascade delete: first artifacts, then runs, then the flow.
    // Uses parameterized db::db_delete to avoid SQL injection.
    let runs = bf_list_runs_inner(&id).await?;
    for run in &runs {
        // Delete artifacts for this run
        if let Ok(artifacts) = db::db_find_all(
            "flow_run_artifacts".into(),
            db::QueryParams {
                filters: vec![db::Filter {
                    field: "run_id".into(),
                    operator: "eq".into(),
                    value: serde_json::json!(run.id),
                }],
                ..Default::default()
            },
            "local".into(),
        ).await {
            for artifact in &artifacts.rows {
                if let Some(aid) = artifact.get("id").and_then(|v| v.as_str()) {
                    let _ = db::db_delete("flow_run_artifacts".into(), aid.into(), "local".into()).await;
                }
            }
        }
        // Delete the run
        let _ = db::db_delete("flow_runs".into(), run.id.clone(), "local".into()).await;
    }

    // Delete flow
    db::db_delete("business_flows".into(), id, "local".into()).await
}

/// Publish a flow (set published=true, locked).
#[tauri::command]
pub async fn bf_publish_flow(id: String) -> Result<FlowRow, String> {
    let existing = bf_get_flow(id.clone()).await?;

    if existing.published {
        return Err("Flow is already published".into());
    }

    let now = now_iso();
    let data = json!({
        "published": true,
        "published_at": now,
        "updated_at": now,
    });

    let data_map: db::DbRow = serde_json::from_value(data)
        .map_err(|e| format!("failed to serialize publish data: {e}"))?;

    db::db_update("business_flows".into(), id.clone(), data_map, "local".into()).await?;

    bf_get_flow(id).await
}

/// Deep-copy a flow with "{Original} (Copy)" name.
#[tauri::command]
pub async fn bf_copy_flow(id: String) -> Result<FlowRow, String> {
    let existing = bf_get_flow(id.clone()).await?;

    let input = FlowInput {
        name: format!("{} (Copy)", existing.name),
        description: existing.description,
        flow_type: "custom".into(),
        flow_json: existing.flow_json,
        output_dir: existing.output_dir,
        output_filename_pattern: existing.output_filename_pattern,
        output_extension: existing.output_extension,
        scenario_bindings: "[]".into(), // bindings not copied
    };

    bf_create_flow(input).await
}

/// Run static validation on a flow's graph.
#[tauri::command]
pub async fn bf_validate_flow(id: String) -> Result<ValidationResult, String> {
    let flow = bf_get_flow(id).await?;
    let graph: FlowGraph = serde_json::from_str(&flow.flow_json)
        .map_err(|e| format!("failed to parse flow_json: {e}"))?;

    let issues = validate_flow(&graph);

    Ok(ValidationResult {
        valid: issues.iter().all(|i| i.severity != "error"),
        issues,
    })
}

/// Validate a flow graph directly (without saving).
#[tauri::command]
pub async fn bf_validate_graph(flow_json: String) -> Result<ValidationResult, String> {
    let graph: FlowGraph = serde_json::from_str(&flow_json)
        .map_err(|e| format!("failed to parse flow_json: {e}"))?;

    let issues = validate_flow(&graph);

    Ok(ValidationResult {
        valid: issues.iter().all(|i| i.severity != "error"),
        issues,
    })
}

// ═══════════════════════════════════════════════════════════════
// Run Tracking Commands
// ═══════════════════════════════════════════════════════════════

/// List all runs for a flow.
#[tauri::command]
pub async fn bf_list_runs(flow_id: String) -> Result<Vec<FlowRunRow>, String> {
    bf_list_runs_inner(&flow_id).await
}

/// Get a single run by ID.
#[tauri::command]
pub async fn bf_get_run(id: String) -> Result<FlowRunRow, String> {
    let row = db::db_find_by_id("flow_runs".into(), id, "local".into())
        .await?
        .ok_or_else(|| "Run not found".to_string())?;

    model::run_row_from_db(&row)
}

/// Create a new run record (called by Conductor before execution).
pub async fn create_run(input: RunInput) -> Result<FlowRunRow, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = now_iso();

    let data = json!({
        "id": id,
        "flow_id": input.flow_id,
        "status": "pending",
        "triggered_by": input.triggered_by,
        "material_paths": input.material_paths,
        "started_at": now,
        "completed_at": null,
        "output_log": "",
        "error_message": null,
    });

    let data_map: db::DbRow = serde_json::from_value(data)
        .map_err(|e| format!("failed to serialize run data: {e}"))?;

    db::db_insert("flow_runs".into(), data_map, "local".into()).await?;

    let row = db::db_find_by_id("flow_runs".into(), id, "local".into())
        .await?
        .ok_or("Run not found after insert")?;

    model::run_row_from_db(&row)
}

/// Update run status (called by Conductor during execution).
pub async fn update_run_status(id: &str, status: &str, error_message: Option<&str>) -> Result<(), String> {
    let now = now_iso();
    let mut data = json!({
        "status": status,
    });

    if status == "completed" || status == "failed" || status == "aborted" {
        data["completed_at"] = json!(now);
    }

    if let Some(msg) = error_message {
        data["error_message"] = json!(msg);
    }

    let data_map: db::DbRow = serde_json::from_value(data)
        .map_err(|e| format!("failed to serialize status update: {e}"))?;

    db::db_update("flow_runs".into(), id.into(), data_map, "local".into()).await
}

// ─── Internal helpers ──────────────────────────────────────────

async fn bf_list_runs_inner(flow_id: &str) -> Result<Vec<FlowRunRow>, String> {
    let result = db::db_find_all(
        "flow_runs".into(),
        db::QueryParams {
            filters: vec![db::Filter {
                field: "flow_id".into(),
                operator: "eq".into(),
                value: json!(flow_id),
            }],
            order_by: vec![db::OrderBy {
                field: "started_at".into(),
                descending: true,
            }],
            ..Default::default()
        },
        "local".into(),
    )
    .await?;

    result
        .rows
        .iter()
        .map(|row| model::run_row_from_db(row))
        .collect()
}

// ─── Ensure migration runs ────────────────────────────────────

/// Ensure the business_flow tables exist.
/// Called once during app startup (from bf_init).
#[tauri::command]
pub async fn bf_init() -> Result<(), String> {
    let sql = include_str!("../db/migrations/m20260604_001_create_business_flow_tables.sql");

    db::db_execute_raw(sql.into(), "local".into()).await?;
    Ok(())
}
