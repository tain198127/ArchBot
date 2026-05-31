use axum::{routing::get, Json, Router};
use serde::Deserialize;

pub fn routes() -> Router {
    Router::new().route("/settings", get(load_settings).post(save_settings))
}

async fn load_settings() -> Json<super::ApiResponse<String>> {
    api_ok!(crate::fs::load_settings())
}

#[derive(Deserialize)]
struct SaveSettingsBody {
    content: String,
}

async fn save_settings(Json(b): Json<SaveSettingsBody>) -> Json<super::EmptyResponse> {
    // Validate that the content is well-formed JSON before writing.
    if serde_json::from_str::<serde_json::Value>(&b.content).is_err() {
        return Json(super::EmptyResponse {
            success: false,
            error: Some("invalid JSON".into()),
        });
    }
    api_empty!(crate::fs::save_settings(b.content))
}
