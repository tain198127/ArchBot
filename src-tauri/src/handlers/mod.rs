use axum::Router;
use serde::Serialize;

/// Unified API response envelope shared by all handlers.
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Await an async Tauri command and wrap its `Result<T, String>` in `Json<ApiResponse<T>>`.
#[macro_export]
macro_rules! api_ok {
    ($expr:expr) => {{
        match $expr.await {
            Ok(v) => axum::Json($crate::handlers::ApiResponse::ok(v)),
            Err(e) => axum::Json($crate::handlers::ApiResponse::err(e)),
        }
    }};
}

/// Wrap a sync Tauri command `Result<T, String>` in `Json<ApiResponse<T>>`.
#[macro_export]
macro_rules! api_ok_sync {
    ($expr:expr) => {{
        match $expr {
            Ok(v) => axum::Json($crate::handlers::ApiResponse::ok(v)),
            Err(e) => axum::Json($crate::handlers::ApiResponse::err(e)),
        }
    }};
}

/// Await an async Tauri command and wrap its `Result<(), String>` in `Json<EmptyResponse>`.
#[macro_export]
macro_rules! api_empty {
    ($expr:expr) => {{
        match $expr.await {
            Ok(()) => axum::Json($crate::handlers::EmptyResponse { success: true, error: None }),
            Err(e) => axum::Json($crate::handlers::EmptyResponse { success: false, error: Some(e) }),
        }
    }};
}

/// Wrap a sync Tauri command `Result<(), String>` in `Json<EmptyResponse>`.
#[macro_export]
macro_rules! api_empty_sync {
    ($expr:expr) => {{
        match $expr {
            Ok(()) => axum::Json($crate::handlers::EmptyResponse { success: true, error: None }),
            Err(e) => axum::Json($crate::handlers::EmptyResponse { success: false, error: Some(e) }),
        }
    }};
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self { success: false, data: None, error: Some(msg.into()) }
    }
}

/// Specialization for empty responses (no data field).
#[derive(Serialize)]
pub struct EmptyResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ── Sub-module declarations (implemented in Phase 3) ──
mod project_handler;
mod ds_handler;
mod db_handler;
mod context_handler;
mod scenario_handler;
mod de_handler;
mod fs_handler;
mod settings_handler;
mod license_handler;
mod vector_handler;

/// Build the combined Axum router with all /api routes.
pub fn router() -> Router {
    Router::new()
        .nest("/api", project_handler::routes())
        .nest("/api", ds_handler::routes())
        .nest("/api", db_handler::routes())
        .nest("/api", context_handler::routes())
        .nest("/api", scenario_handler::routes())
        .nest("/api", de_handler::routes())
        .nest("/api", fs_handler::routes())
        .nest("/api", settings_handler::routes())
        .nest("/api", license_handler::routes())
        .nest("/api", vector_handler::routes())
}
