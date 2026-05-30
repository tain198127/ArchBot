use axum::{routing::get, Json, Router};
use serde::Deserialize;

pub fn routes() -> Router {
    Router::new()
        .route("/settings", get(load_settings).post(save_settings))
}

async fn load_settings() -> Json<super::ApiResponse<String>> {
    api_ok!(crate::fs::load_settings())
}

#[derive(Deserialize)]
struct SaveSettingsBody { content: String }

async fn save_settings(Json(b): Json<SaveSettingsBody>) -> Json<super::EmptyResponse> {
    api_empty!(crate::fs::save_settings(b.content))
}
