use axum::{
    extract::Path,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::agent_runtime::event_stream::{self, EventQuery};
use crate::agent_runtime::session_manager::{CreateSessionRequest, SessionManager, TurnRequest};

pub fn routes() -> Router {
    Router::new()
        // Session
        .route("/agent/sessions", post(create_session))
        .route("/agent/sessions", get(list_sessions))
        .route("/agent/sessions/{session_id}", get(get_session))
        .route(
            "/agent/sessions/{session_id}/status",
            post(update_session_status),
        )
        // Turn
        .route("/agent/sessions/{session_id}/turns", post(create_turn))
        .route("/agent/sessions/{session_id}/turns", get(list_turns))
        .route(
            "/agent/sessions/{session_id}/turns/{turn_id}",
            get(get_turn),
        )
        // Events (JSON replay)
        .route(
            "/agent/sessions/{session_id}/turns/{turn_id}/events",
            get(get_events),
        )
        // Events (SSE stream — real-time + replay)
        .route(
            "/agent/sessions/{session_id}/turns/{turn_id}/stream",
            get(event_stream::sse_handler),
        )
        // File changes
        .route("/agent/turns/{turn_id}/file-changes", get(get_file_changes))
        // Audit log
        .route("/agent/audit-log", get(get_audit_log))
}

/// Run a blocking operation on a dedicated thread to avoid nested-runtime panics.
/// SessionManager methods use `block_on` internally, which panics inside tokio worker threads.
async fn spawn_blocking_op<T, F>(f: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .unwrap_or_else(|e| Err(format!("spawn_blocking error: {}", e)))
}

// ── Session ──

#[derive(Deserialize)]
struct CreateSessionBody {
    title: String,
    goal: Option<String>,
    project_id: Option<String>,
    runtime_type: Option<String>,
}

async fn create_session(
    Json(b): Json<CreateSessionBody>,
) -> Json<super::ApiResponse<crate::agent_runtime::session_manager::AgentSession>> {
    let result = spawn_blocking_op(move || {
        SessionManager::new().create(CreateSessionRequest {
            title: b.title,
            goal: b.goal.unwrap_or_default(),
            project_id: b.project_id.unwrap_or_default(),
            runtime_type: b.runtime_type.unwrap_or_else(|| "claude_code".into()),
        })
    })
    .await;
    match result {
        Ok(session) => Json(super::ApiResponse::ok(session)),
        Err(e) => Json(super::ApiResponse::err(e)),
    }
}

async fn list_sessions(
) -> Json<super::ApiResponse<Vec<crate::agent_runtime::session_manager::AgentSession>>> {
    let result = spawn_blocking_op(|| SessionManager::new().list_all()).await;
    match result {
        Ok(sessions) => Json(super::ApiResponse::ok(sessions)),
        Err(e) => Json(super::ApiResponse::err(e)),
    }
}

#[derive(Deserialize)]
struct UpdateStatusBody {
    status: String,
}

async fn get_session(
    Path(session_id): Path<String>,
) -> Json<super::ApiResponse<crate::agent_runtime::session_manager::AgentSession>> {
    let result = spawn_blocking_op(move || {
        match SessionManager::new().get(&session_id) {
            Ok(Some(s)) => Ok(s),
            Ok(None) => Err("session not found".into()),
            Err(e) => Err(e),
        }
    })
    .await;
    match result {
        Ok(s) => Json(super::ApiResponse::ok(s)),
        Err(e) => Json(super::ApiResponse::err(e)),
    }
}

async fn update_session_status(
    Path(session_id): Path<String>,
    Json(b): Json<UpdateStatusBody>,
) -> Json<super::EmptyResponse> {
    let result = spawn_blocking_op(move || {
        SessionManager::new().update_status(&session_id, &b.status)
    })
    .await;
    match result {
        Ok(()) => Json(super::EmptyResponse {
            success: true,
            error: None,
        }),
        Err(e) => Json(super::EmptyResponse {
            success: false,
            error: Some(e),
        }),
    }
}

// ── Turn ──

#[derive(Deserialize)]
struct CreateTurnBody {
    user_message: String,
    context_files: Option<Vec<String>>,
    runtime_type: Option<String>,
}

async fn create_turn(
    Path(session_id): Path<String>,
    Json(b): Json<CreateTurnBody>,
) -> Json<super::ApiResponse<crate::agent_runtime::turn_config::TurnResult>> {
    let result = spawn_blocking_op(move || {
        let mgr = SessionManager::new();
        let session = mgr
            .get(&session_id)?
            .ok_or(format!("session not found: {}", session_id))?;

        let workspace_root = session.project_id.clone();
        let turn_req = TurnRequest {
            session_id,
            user_message: b.user_message,
            context_files: b.context_files.unwrap_or_default(),
            runtime_type: b.runtime_type.unwrap_or(session.runtime_type),
            workspace_root,
        };
        mgr.create_and_execute_turn(&turn_req)
    })
    .await;
    match result {
        Ok(result) => Json(super::ApiResponse::ok(result)),
        Err(e) => Json(super::ApiResponse::err(e)),
    }
}

async fn list_turns(
    Path(session_id): Path<String>,
) -> Json<super::ApiResponse<Vec<crate::agent_runtime::session_manager::AgentTurnInfo>>> {
    let result = spawn_blocking_op(move || SessionManager::new().list_turns(&session_id)).await;
    match result {
        Ok(turns) => Json(super::ApiResponse::ok(turns)),
        Err(e) => Json(super::ApiResponse::err(e)),
    }
}

async fn get_turn(
    Path((_session_id, turn_id)): Path<(String, String)>,
) -> Json<super::ApiResponse<crate::agent_runtime::session_manager::AgentTurnInfo>> {
    let result = spawn_blocking_op(move || {
        match SessionManager::new().get_turn(&turn_id) {
            Ok(Some(t)) => Ok(t),
            Ok(None) => Err("turn not found".into()),
            Err(e) => Err(e),
        }
    })
    .await;
    match result {
        Ok(t) => Json(super::ApiResponse::ok(t)),
        Err(e) => Json(super::ApiResponse::err(e)),
    }
}

// ── Events ──

#[derive(Serialize)]
struct EventListResponse {
    events: Vec<EventQuery>,
    total: usize,
}

async fn get_events(
    Path((session_id, turn_id)): Path<(String, String)>,
) -> Json<super::ApiResponse<EventListResponse>> {
    let result = spawn_blocking_op(move || SessionManager::new().get_events(&session_id, &turn_id)).await;
    match result {
        Ok(events) => Json(super::ApiResponse::ok(EventListResponse {
            total: events.len(),
            events,
        })),
        Err(e) => Json(super::ApiResponse::err(e)),
    }
}

// ── File Changes ──

#[derive(Serialize)]
struct FileChangesResponse {
    changes: Vec<crate::agent_runtime::turn_config::FileChange>,
    total: usize,
}

async fn get_file_changes(
    Path(turn_id): Path<String>,
) -> Json<super::ApiResponse<FileChangesResponse>> {
    let result = spawn_blocking_op(move || SessionManager::new().get_file_changes(&turn_id)).await;
    match result {
        Ok(changes) => Json(super::ApiResponse::ok(FileChangesResponse {
            total: changes.len(),
            changes,
        })),
        Err(e) => Json(super::ApiResponse::err(e)),
    }
}

// ── Audit Log ──

#[derive(Serialize)]
struct AuditLogResponse {
    entries: Vec<crate::agent_runtime::audit::AuditEntry>,
    total: usize,
}

async fn get_audit_log() -> Json<super::ApiResponse<AuditLogResponse>> {
    let result = spawn_blocking_op(|| SessionManager::new().get_audit_log()).await;
    match result {
        Ok(entries) => Json(super::ApiResponse::ok(AuditLogResponse {
            total: entries.len(),
            entries,
        })),
        Err(e) => Json(super::ApiResponse::err(e)),
    }
}
