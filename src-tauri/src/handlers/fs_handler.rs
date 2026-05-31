use axum::{routing::post, Json, Router};
use serde::Deserialize;

pub fn routes() -> Router {
    Router::new()
        .route("/fs/read", post(fs_read))
        .route("/fs/write", post(fs_write))
        .route("/fs/list", post(fs_list))
        .route("/fs/delete", post(fs_delete))
        .route("/fs/exists", post(fs_exists))
        .route("/fs/mkdir", post(fs_mkdir))
}

// fs_configure_local is intentionally excluded from the HTTP API.
// It sets the file-system sandbox base directory and is only safe via Tauri IPC.

#[derive(Deserialize)]
struct ReadBody {
    path: String,
    fs_type: String,
}

async fn fs_read(Json(b): Json<ReadBody>) -> Json<super::ApiResponse<String>> {
    api_ok!(crate::fs::fs_read(b.path, b.fs_type))
}

#[derive(Deserialize)]
struct WriteBody {
    path: String,
    content: String,
    fs_type: String,
}

async fn fs_write(Json(b): Json<WriteBody>) -> Json<super::EmptyResponse> {
    api_empty!(crate::fs::fs_write(b.path, b.content, b.fs_type))
}

#[derive(Deserialize)]
struct ListBody {
    path: String,
    fs_type: String,
}

async fn fs_list(Json(b): Json<ListBody>) -> Json<super::ApiResponse<Vec<crate::fs::FileEntry>>> {
    api_ok!(crate::fs::fs_list(b.path, b.fs_type))
}

#[derive(Deserialize)]
struct DeleteBody {
    path: String,
    fs_type: String,
}

async fn fs_delete(Json(b): Json<DeleteBody>) -> Json<super::EmptyResponse> {
    api_empty!(crate::fs::fs_delete(b.path, b.fs_type))
}

#[derive(Deserialize)]
struct ExistsBody {
    path: String,
    fs_type: String,
}

async fn fs_exists(Json(b): Json<ExistsBody>) -> Json<super::ApiResponse<bool>> {
    api_ok!(crate::fs::fs_exists(b.path, b.fs_type))
}

#[derive(Deserialize)]
struct MkdirBody {
    path: String,
    fs_type: String,
}

async fn fs_mkdir(Json(b): Json<MkdirBody>) -> Json<super::EmptyResponse> {
    api_empty!(crate::fs::fs_mkdir(b.path, b.fs_type))
}
