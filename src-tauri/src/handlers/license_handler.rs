use axum::{routing::{get, post}, Json, Router};
use serde::Deserialize;

pub fn routes() -> Router {
    Router::new()
        .route("/license/status", get(get_license_status))
        .route("/license/register", post(register_software))
}

async fn get_license_status() -> Json<super::ApiResponse<crate::license::LicenseStatus>> {
    api_ok!(crate::license::get_license_status())
}

#[derive(Deserialize)]
struct RegisterBody { verification_code: String }

async fn register_software(Json(b): Json<RegisterBody>) -> Json<super::ApiResponse<bool>> {
    api_ok!(crate::license::register_software(b.verification_code))
}
