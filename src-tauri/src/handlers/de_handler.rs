use axum::{extract::Query, routing::{get, post}, Json, Router};
use serde::Deserialize;

pub fn routes() -> Router {
    Router::new()
        .route("/de/init", post(de_init))
        .route("/de/list", get(de_list))
        .route("/de/get", get(de_get))
        .route("/de/save", post(de_save))
        .route("/de/delete", post(de_delete))
}

#[derive(Deserialize)]
struct InitBody { db_type: String, project_path: String }

async fn de_init(Json(b): Json<InitBody>) -> Json<super::EmptyResponse> {
    api_empty!(crate::digital_employee::de_init(b.db_type, b.project_path))
}

#[derive(Deserialize)]
struct ListQuery { db_type: String }

async fn de_list(Query(q): Query<ListQuery>) -> Json<super::ApiResponse<Vec<crate::digital_employee::DigitalEmployee>>> {
    api_ok!(crate::digital_employee::de_list(q.db_type))
}

#[derive(Deserialize)]
struct GetQuery { code: String, db_type: String }

async fn de_get(Query(q): Query<GetQuery>) -> Json<super::ApiResponse<Option<crate::digital_employee::DigitalEmployee>>> {
    api_ok!(crate::digital_employee::de_get(q.code, q.db_type))
}

#[derive(Deserialize)]
struct SaveBody { employee: crate::digital_employee::DigitalEmployee, db_type: String }

async fn de_save(Json(b): Json<SaveBody>) -> Json<super::EmptyResponse> {
    api_empty!(crate::digital_employee::de_save(b.employee, b.db_type))
}

#[derive(Deserialize)]
struct DeleteBody { id: i32, db_type: String }

async fn de_delete(Json(b): Json<DeleteBody>) -> Json<super::EmptyResponse> {
    api_empty!(crate::digital_employee::de_delete(b.id, b.db_type))
}
