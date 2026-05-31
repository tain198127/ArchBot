use axum::{routing::get, Json, Router};
use serde::Deserialize;

pub fn routes() -> Router {
    Router::new().route("/scenario", get(get_scenario).post(save_scenario))
}

#[derive(Deserialize)]
struct ScenarioQuery {
    project_path: String,
}

async fn get_scenario(
    axum::extract::Query(q): axum::extract::Query<ScenarioQuery>,
) -> Json<super::ApiResponse<crate::scenario::ProjectScenario>> {
    api_ok_sync!(crate::scenario::get_scenario(q.project_path))
}

#[derive(Deserialize)]
struct SaveScenarioBody {
    project_path: String,
    scenario: crate::scenario::ProjectScenario,
}

async fn save_scenario(Json(b): Json<SaveScenarioBody>) -> Json<super::EmptyResponse> {
    api_empty_sync!(crate::scenario::save_scenario(b.project_path, b.scenario))
}
