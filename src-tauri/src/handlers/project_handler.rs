use axum::{routing::post, Json, Router};
use serde::Deserialize;

pub fn routes() -> Router {
    Router::new()
        .route("/project/open", post(open_project))
        .route("/project/create", post(create_project))
        .route("/project/init-dir", post(init_archbot_dir))
        .route("/project/ensure-gitignore", post(ensure_gitignore))
}

#[derive(Deserialize)]
struct OpenProjectBody { path: String }

async fn open_project(Json(b): Json<OpenProjectBody>) -> Json<super::ApiResponse<crate::fs::FileContent>> {
    api_ok!(crate::fs::open_project(b.path))
}

#[derive(Deserialize)]
struct CreateProjectBody { dir: String, name: String }

async fn create_project(Json(b): Json<CreateProjectBody>) -> Json<super::ApiResponse<String>> {
    api_ok!(crate::fs::create_project(b.dir, b.name))
}

#[derive(Deserialize)]
struct InitDirBody { project_path: String }

async fn init_archbot_dir(Json(b): Json<InitDirBody>) -> Json<super::EmptyResponse> {
    api_empty_sync!(crate::fs::init_archbot_dir(b.project_path))
}

#[derive(Deserialize)]
struct EnsureGitignoreBody { project_path: String }

async fn ensure_gitignore(Json(b): Json<EnsureGitignoreBody>) -> Json<super::ApiResponse<bool>> {
    api_ok_sync!(crate::fs::ensure_gitignore(b.project_path))
}

// read_local_file is intentionally excluded from the HTTP API.
// It allows arbitrary file reads and is only safe via Tauri IPC.
