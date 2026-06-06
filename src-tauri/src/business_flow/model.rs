//! Business Flow — data types and database access
//!
//! Defines the Rust structs that map to the 3-table SQLite schema.
//! Uses the existing `db::local_sqlite::LocalSqliteDb` for data access
//! rather than dedicated SeaORM entities, matching project convention.

use serde::{Deserialize, Serialize};

// ─── Flow Definition ──────────────────────────────────────────

/// Full flow row from `business_flows` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowRow {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub flow_type: String, // "builtin" | "custom"
    pub published: bool,
    pub flow_json: String,
    pub output_dir: String,
    pub output_filename_pattern: String,
    pub output_extension: String,
    pub scenario_bindings: String, // JSON array
    pub yaml_export: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub published_at: Option<String>,
    pub version: i64,
}

/// Summary row for list queries (no graph payload).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub flow_type: String,
    pub published: bool,
    pub scenario_bindings: String,
    pub version: i64,
    pub updated_at: String,
}

/// Input payload for creating or updating a flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowInput {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub flow_type: String,
    pub flow_json: String,
    pub output_dir: String,
    pub output_filename_pattern: String,
    pub output_extension: String,
    pub scenario_bindings: String,
}

// ─── Run Tracking ─────────────────────────────────────────────

/// Row from `flow_runs` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowRunRow {
    pub id: String,
    pub flow_id: String,
    pub status: String, // "pending" | "running" | "completed" | "failed" | "aborted"
    pub triggered_by: String,
    pub material_paths: String, // JSON array
    pub started_at: String,
    pub completed_at: Option<String>,
    pub output_log: String,
    pub error_message: Option<String>,
}

/// Input for creating a new run.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunInput {
    pub flow_id: String,
    pub triggered_by: String,
    pub material_paths: String,
}

// ─── Artifacts ────────────────────────────────────────────────

/// Row from `flow_run_artifacts` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowRunArtifactRow {
    pub id: String,
    pub run_id: String,
    pub node_id: String,
    pub agent_id: String,
    pub artifact_path: String,
    pub artifact_type: String,
    pub created_at: String,
    pub checksum: String,
}

// ─── Validation ───────────────────────────────────────────────

/// Single validation issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationIssue {
    pub severity: String, // "error" | "warning"
    pub message: String,
    pub node_id: Option<String>,
    pub edge_id: Option<String>,
}

/// Validation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    pub valid: bool,
    pub issues: Vec<ValidationIssue>,
}

// ─── DB Helper Functions ──────────────────────────────────────

use crate::db::DbRow;

/// Convert a `FlowRow` from a generic `DbRow`.
pub fn flow_row_from_db(row: &DbRow) -> Result<FlowRow, String> {
    Ok(FlowRow {
        id: string_field(row, "id")?,
        name: string_field(row, "name")?,
        description: string_field(row, "description")?,
        flow_type: string_field(row, "type")?,
        published: bool_field(row, "published")?,
        flow_json: string_field(row, "flow_json")?,
        output_dir: string_field(row, "output_dir")?,
        output_filename_pattern: string_field(row, "output_filename_pattern")?,
        output_extension: string_field(row, "output_extension")?,
        scenario_bindings: string_field(row, "scenario_bindings")?,
        yaml_export: opt_string_field(row, "yaml_export"),
        created_at: string_field(row, "created_at")?,
        updated_at: string_field(row, "updated_at")?,
        published_at: opt_string_field(row, "published_at"),
        version: int_field(row, "version")?,
    })
}

/// Convert a `FlowRunRow` from a generic `DbRow`.
pub fn run_row_from_db(row: &DbRow) -> Result<FlowRunRow, String> {
    Ok(FlowRunRow {
        id: string_field(row, "id")?,
        flow_id: string_field(row, "flow_id")?,
        status: string_field(row, "status")?,
        triggered_by: string_field(row, "triggered_by")?,
        material_paths: string_field(row, "material_paths")?,
        started_at: string_field(row, "started_at")?,
        completed_at: opt_string_field(row, "completed_at"),
        output_log: string_field(row, "output_log").unwrap_or_default(),
        error_message: opt_string_field(row, "error_message"),
    })
}

/// Convert a `FlowRunArtifactRow` from a generic `DbRow`.
pub fn artifact_row_from_db(row: &DbRow) -> Result<FlowRunArtifactRow, String> {
    Ok(FlowRunArtifactRow {
        id: string_field(row, "id")?,
        run_id: string_field(row, "run_id")?,
        node_id: string_field(row, "node_id")?,
        agent_id: string_field(row, "agent_id")?,
        artifact_path: string_field(row, "artifact_path")?,
        artifact_type: string_field(row, "artifact_type")?,
        created_at: string_field(row, "created_at")?,
        checksum: string_field(row, "checksum")?,
    })
}

// ─── Field extraction helpers ─────────────────────────────────

fn string_field(row: &DbRow, key: &str) -> Result<String, String> {
    row.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("missing string field: {key}"))
}

fn opt_string_field(row: &DbRow, key: &str) -> Option<String> {
    row.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

fn bool_field(row: &DbRow, key: &str) -> Result<bool, String> {
    row.get(key)
        .and_then(|v| {
            // SQLite stores booleans as INTEGER (0/1).
            // serde_json::Value::as_bool() only works with Value::Bool,
            // so we also check for integer 0/1.
            if v.as_bool() == Some(true) {
                return Some(true);
            }
            if let Some(n) = v.as_i64() {
                return Some(n != 0);
            }
            if let Some(n) = v.as_u64() {
                return Some(n != 0);
            }
            None
        })
        .ok_or_else(|| format!("missing bool field: {key}"))
}

fn int_field(row: &DbRow, key: &str) -> Result<i64, String> {
    row.get(key)
        .and_then(|v| v.as_i64())
        .ok_or_else(|| format!("missing int field: {key}"))
}
