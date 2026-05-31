use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::audit::{AuditEntry, AuditSeverity};
use super::event_stream::{EventBus, StandardEvent};
use super::turn_config::{FileChange, TurnResult};
use super::turn_executor::execute_turn;
use crate::db::{self, DbBackend};
use serde_json::Value;

// ─── Session ───

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentSession {
    pub session_id: String,
    pub title: String,
    pub goal: String,
    pub project_id: String,
    pub runtime_type: String,
    pub default_model: String,
    pub current_state: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

// ─── Turn Info (for listing) ───

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentTurnInfo {
    pub turn_id: String,
    pub session_id: String,
    pub sequence_number: i32,
    pub user_message: String,
    pub status: String,
    pub runtime_type: String,
    pub runtime_version: String,
    pub model: String,
    pub started_at: String,
    pub finished_at: String,
    pub error_message: String,
    pub duration_ms: i32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub title: String,
    pub goal: String,
    pub project_id: String,
    pub runtime_type: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TurnRequest {
    pub session_id: String,
    pub user_message: String,
    pub context_files: Vec<String>,
    pub runtime_type: String,
    pub workspace_root: String,
}

// ─── Session Manager ───

pub struct SessionManager;

impl SessionManager {
    pub fn new() -> Self {
        Self
    }

    fn now(&self) -> String {
        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
    }

    // ── Session CRUD ──

    pub fn create(&self, req: CreateSessionRequest) -> Result<AgentSession, String> {
        let session_id = Uuid::new_v4().to_string();
        let now = self.now();

        let session = AgentSession {
            session_id: session_id.clone(),
            title: req.title,
            goal: req.goal,
            project_id: req.project_id,
            runtime_type: req.runtime_type,
            default_model: String::new(),
            current_state: String::new(),
            status: "active".into(),
            created_at: now.clone(),
            updated_at: now,
        };

        // Emit event
        let bus = EventBus::global();
        bus.publish(StandardEvent::session_created(
            &session.session_id,
            &session.runtime_type,
        ));

        Ok(session)
    }

    pub fn get(&self, session_id: &str) -> Result<Option<AgentSession>, String> {
        // Query from DB via the generic DbBackend
        let rt = tokio::runtime::Handle::current();
        let session_id = session_id.to_string();
        rt.block_on(async {
            let db_cell = db::local_db_cell().lock().await;
            let backend = db_cell.as_ref().ok_or("db not connected")?;
            backend.find_by_id("agent_session", &session_id).await
        })
        .map(|row_opt| {
            row_opt.map(|row| AgentSession {
                session_id: row
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
                title: row
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
                goal: row
                    .get("goal")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
                project_id: row
                    .get("project_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
                runtime_type: row
                    .get("runtime_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
                default_model: row
                    .get("default_model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
                current_state: row
                    .get("current_state")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
                status: row
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("active")
                    .into(),
                created_at: row
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
                updated_at: row
                    .get("updated_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
            })
        })
    }

    pub fn list_all(&self) -> Result<Vec<AgentSession>, String> {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let db_cell = db::local_db_cell().lock().await;
            let backend = db_cell.as_ref().ok_or("db not connected")?;
            let result = backend
                .find_all("agent_session", db::QueryParams::default())
                .await?;
            Ok(result
                .rows
                .iter()
                .map(|row| AgentSession {
                    session_id: row
                        .get("session_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .into(),
                    title: row
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .into(),
                    goal: row
                        .get("goal")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .into(),
                    project_id: row
                        .get("project_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .into(),
                    runtime_type: row
                        .get("runtime_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .into(),
                    default_model: row
                        .get("default_model")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .into(),
                    current_state: row
                        .get("current_state")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .into(),
                    status: row
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("active")
                        .into(),
                    created_at: row
                        .get("created_at")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .into(),
                    updated_at: row
                        .get("updated_at")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .into(),
                })
                .collect())
        })
    }

    pub fn update_status(&self, session_id: &str, new_status: &str) -> Result<(), String> {
        // Validate state transition
        let session = self
            .get(session_id)?
            .ok_or(format!("session not found: {}", session_id))?;
        let valid = matches!(
            (session.status.as_str(), new_status),
            ("active", "paused" | "closed")
                | ("paused", "active" | "closed")
                | ("closed", "archived")
        );
        if !valid {
            return Err(format!(
                "invalid transition: {} -> {}",
                session.status, new_status
            ));
        }

        let rt = tokio::runtime::Handle::current();
        let sid = session_id.to_string();
        let status = new_status.to_string();
        rt.block_on(async {
            let db_cell = db::local_db_cell().lock().await;
            let backend = db_cell.as_ref().ok_or("db not connected")?;
            let mut data = db::DbRow::new();
            data.insert("status".into(), Value::String(status.clone()));
            data.insert(
                "updated_at".into(),
                Value::String(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()),
            );
            backend.update("agent_session", &sid, data).await
        })?;

        // Emit event
        if new_status == "closed" {
            let bus = EventBus::global();
            bus.publish(StandardEvent::session_closed(session_id));
        }

        Ok(())
    }

    // ── Turn Management ──

    pub fn create_and_execute_turn(&self, req: &TurnRequest) -> Result<TurnResult, String> {
        // Verify session exists and is active
        let session = self
            .get(&req.session_id)?
            .ok_or(format!("session not found: {}", req.session_id))?;

        if session.status != "active" && session.status != "paused" {
            return Err(format!("cannot create turn in {} session", session.status));
        }

        // If paused, re-activate
        if session.status == "paused" {
            self.update_status(&req.session_id, "active")?;
        }

        let start_time = self.now();

        // Emit turn.started
        let bus = EventBus::global();
        let turn_id = Uuid::new_v4().to_string();
        bus.publish(StandardEvent::turn_started(
            &req.session_id,
            &turn_id,
            &req.runtime_type,
        ));

        // Record turn start in DB
        let _ = self.save_turn_start(&req.session_id, &turn_id, req, &start_time);

        // Execute via the existing turn_executor
        let result = execute_turn(super::turn_config::TurnConfig {
            runtime: req.runtime_type.clone(),
            workspace_root: req.workspace_root.clone(),
            user_message: req.user_message.clone(),
            context_files: req.context_files.clone(),
            git_user_name: None,
            git_user_email: None,
        })?;

        // Emit turn.completed or turn.failed
        if result.status.contains("failed") || result.status.contains("timeout") {
            bus.publish(StandardEvent::turn_failed(
                &req.session_id,
                &turn_id,
                &result.status,
            ));
        } else {
            bus.publish(StandardEvent::turn_completed(&req.session_id, &turn_id));
        }

        // Record turn finish in DB
        let _ = self.save_turn_finish(&turn_id, &result);

        // Persist file changes
        for change in &result.file_changes {
            let _ = self.save_file_change(&turn_id, change);
        }

        Ok(result)
    }

    pub fn list_turns(&self, session_id: &str) -> Result<Vec<AgentTurnInfo>, String> {
        let rt = tokio::runtime::Handle::current();
        let sid = session_id.to_string();
        rt.block_on(async {
            let db_cell = db::local_db_cell().lock().await;
            let backend = db_cell.as_ref().ok_or("db not connected")?;
            let params = db::QueryParams {
                filters: vec![db::Filter {
                    field: "session_id".into(),
                    operator: "eq".into(),
                    value: Value::String(sid),
                }],
                order_by: vec![db::OrderBy {
                    field: "sequence_number".into(),
                    descending: true,
                }],
                ..Default::default()
            };
            let result = backend.find_all("agent_turn", params).await?;
            Ok(result
                .rows
                .iter()
                .map(|row| AgentTurnInfo {
                    turn_id: row
                        .get("turn_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .into(),
                    session_id: row
                        .get("session_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .into(),
                    sequence_number: row
                        .get("sequence_number")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0) as i32,
                    user_message: row
                        .get("user_message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .into(),
                    status: row
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .into(),
                    runtime_type: row
                        .get("runtime_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .into(),
                    runtime_version: row
                        .get("runtime_version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .into(),
                    model: row
                        .get("model")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .into(),
                    started_at: row
                        .get("started_at")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .into(),
                    finished_at: row
                        .get("finished_at")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .into(),
                    error_message: row
                        .get("error_message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .into(),
                    duration_ms: row.get("duration_ms").and_then(|v| v.as_i64()).unwrap_or(0)
                        as i32,
                })
                .collect())
        })
    }

    pub fn get_turn(&self, turn_id: &str) -> Result<Option<AgentTurnInfo>, String> {
        let rt = tokio::runtime::Handle::current();
        let tid = turn_id.to_string();
        rt.block_on(async {
            let db_cell = db::local_db_cell().lock().await;
            let backend = db_cell.as_ref().ok_or("db not connected")?;
            backend.find_by_id("agent_turn", &tid).await
        })
        .map(|row_opt| {
            row_opt.map(|row| AgentTurnInfo {
                turn_id: row
                    .get("turn_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
                session_id: row
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
                sequence_number: row
                    .get("sequence_number")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32,
                user_message: row
                    .get("user_message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
                status: row
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
                runtime_type: row
                    .get("runtime_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
                runtime_version: row
                    .get("runtime_version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
                model: row
                    .get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
                started_at: row
                    .get("started_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
                finished_at: row
                    .get("finished_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
                error_message: row
                    .get("error_message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
                duration_ms: row.get("duration_ms").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            })
        })
    }

    // ── Events ──

    pub fn get_events(
        &self,
        _session_id: &str,
        turn_id: &str,
    ) -> Result<Vec<super::event_stream::EventQuery>, String> {
        // Query from event bus backlog
        let bus = EventBus::global();
        Ok(bus.query_by_turn(turn_id))
    }

    // ── File Changes ──

    pub fn get_file_changes(&self, turn_id: &str) -> Result<Vec<FileChange>, String> {
        let rt = tokio::runtime::Handle::current();
        let tid = turn_id.to_string();
        rt.block_on(async {
            let db_cell = db::local_db_cell().lock().await;
            let backend = db_cell.as_ref().ok_or("db not connected")?;
            let params = db::QueryParams {
                filters: vec![db::Filter {
                    field: "turn_id".into(),
                    operator: "eq".into(),
                    value: Value::String(tid),
                }],
                ..Default::default()
            };
            let result = backend.find_all("agent_file_change", params).await?;
            Ok(result
                .rows
                .iter()
                .map(|row| FileChange {
                    path: row
                        .get("file_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .into(),
                    change_type: row
                        .get("change_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("modified")
                        .into(),
                })
                .collect())
        })
    }

    // ── Audit Log ──

    pub fn get_audit_log(&self) -> Result<Vec<AuditEntry>, String> {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let db_cell = db::local_db_cell().lock().await;
            let backend = db_cell.as_ref().ok_or("db not connected")?;
            let params = db::QueryParams {
                order_by: vec![db::OrderBy {
                    field: "created_at".into(),
                    descending: true,
                }],
                limit: Some(200),
                ..Default::default()
            };
            let result = backend.find_all("agent_audit_log", params).await?;
            Ok(result
                .rows
                .iter()
                .map(|row| AuditEntry {
                    log_id: row
                        .get("log_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .into(),
                    action: row
                        .get("action")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .into(),
                    severity: match row
                        .get("severity")
                        .and_then(|v| v.as_str())
                        .unwrap_or("info")
                    {
                        "critical" => AuditSeverity::Critical,
                        "high" => AuditSeverity::High,
                        "warning" => AuditSeverity::Warning,
                        _ => AuditSeverity::Info,
                    },
                    detail: row
                        .get("detail")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .into(),
                    created_at: row
                        .get("created_at")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .into(),
                })
                .collect())
        })
    }

    // ── Internal helpers ──

    fn save_turn_start(
        &self,
        session_id: &str,
        turn_id: &str,
        req: &TurnRequest,
        now: &str,
    ) -> Result<(), String> {
        let rt = tokio::runtime::Handle::current();
        let mut data = db::DbRow::new();
        data.insert("turn_id".into(), Value::String(turn_id.into()));
        data.insert("session_id".into(), Value::String(session_id.into()));
        data.insert("sequence_number".into(), Value::from(0));
        data.insert(
            "user_message".into(),
            Value::String(req.user_message.clone()),
        );
        data.insert("interpreted_intent".into(), Value::String(String::new()));
        data.insert("input_file_path".into(), Value::String(String::new()));
        data.insert("prompt_file_path".into(), Value::String(String::new()));
        data.insert("status".into(), Value::String("running".into()));
        data.insert(
            "runtime_type".into(),
            Value::String(req.runtime_type.clone()),
        );
        data.insert("runtime_version".into(), Value::String(String::new()));
        data.insert("model".into(), Value::String(String::new()));
        data.insert("started_at".into(), Value::String(now.into()));
        data.insert("finished_at".into(), Value::String(String::new()));
        data.insert("error_message".into(), Value::String(String::new()));
        data.insert("duration_ms".into(), Value::from(0));

        let sid = session_id.to_string();
        let tid = turn_id.to_string();
        rt.block_on(async {
            let db_cell = db::local_db_cell().lock().await;
            let backend = db_cell.as_ref().ok_or("db not connected")?;
            backend.insert("agent_turn", data).await.map(|_| ())
        })
    }

    fn save_turn_finish(&self, turn_id: &str, result: &TurnResult) -> Result<(), String> {
        let rt = tokio::runtime::Handle::current();
        let now = self.now();
        let mut data = db::DbRow::new();
        let status = if result.status.contains("failed") || result.status.contains("timeout") {
            result.status.clone()
        } else {
            "completed".into()
        };
        data.insert("status".into(), Value::String(status));
        data.insert("finished_at".into(), Value::String(now));
        data.insert("duration_ms".into(), Value::from(result.duration_ms as i64));

        let tid = turn_id.to_string();
        rt.block_on(async {
            let db_cell = db::local_db_cell().lock().await;
            let backend = db_cell.as_ref().ok_or("db not connected")?;
            backend.update("agent_turn", &tid, data).await
        })
    }

    fn save_file_change(&self, turn_id: &str, change: &FileChange) -> Result<(), String> {
        let rt = tokio::runtime::Handle::current();
        let change_id = Uuid::new_v4().to_string();
        let now = self.now();
        let mut data = db::DbRow::new();
        data.insert("change_id".into(), Value::String(change_id));
        data.insert("turn_id".into(), Value::String(turn_id.into()));
        data.insert("file_path".into(), Value::String(change.path.clone()));
        data.insert(
            "change_type".into(),
            Value::String(change.change_type.clone()),
        );
        data.insert("diff_content".into(), Value::String(String::new()));
        data.insert("file_hash_before".into(), Value::String(String::new()));
        data.insert("file_hash_after".into(), Value::String(String::new()));
        data.insert("size_before".into(), Value::from(0));
        data.insert("size_after".into(), Value::from(0));
        data.insert("created_at".into(), Value::String(now));

        rt.block_on(async {
            let db_cell = db::local_db_cell().lock().await;
            let backend = db_cell.as_ref().ok_or("db not connected")?;
            backend.insert("agent_file_change", data).await.map(|_| ())
        })
    }
}

// ─── Tauri Commands ───

#[tauri::command]
pub fn agent_create_session(
    title: String,
    goal: Option<String>,
    project_id: Option<String>,
    runtime_type: Option<String>,
) -> Result<AgentSession, String> {
    let mgr = SessionManager::new();
    mgr.create(CreateSessionRequest {
        title,
        goal: goal.unwrap_or_default(),
        project_id: project_id.unwrap_or_default(),
        runtime_type: runtime_type.unwrap_or_else(|| "claude_code".into()),
    })
}

#[tauri::command]
pub fn agent_list_sessions() -> Result<Vec<AgentSession>, String> {
    SessionManager::new().list_all()
}

#[tauri::command]
pub fn agent_get_session(session_id: String) -> Result<Option<AgentSession>, String> {
    SessionManager::new().get(&session_id)
}

#[tauri::command]
pub fn agent_update_session_status(session_id: String, status: String) -> Result<(), String> {
    SessionManager::new().update_status(&session_id, &status)
}

#[tauri::command]
pub fn agent_create_turn(
    session_id: String,
    user_message: String,
    context_files: Option<Vec<String>>,
    runtime_type: Option<String>,
    workspace_root: Option<String>,
) -> Result<TurnResult, String> {
    let mgr = SessionManager::new();
    let session = mgr
        .get(&session_id)?
        .ok_or(format!("session not found: {}", session_id))?;
    let req = TurnRequest {
        session_id,
        user_message,
        context_files: context_files.unwrap_or_default(),
        runtime_type: runtime_type.unwrap_or(session.runtime_type),
        workspace_root: workspace_root.unwrap_or(session.project_id),
    };
    mgr.create_and_execute_turn(&req)
}
