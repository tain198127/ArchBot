use axum::{routing::post, Json, Router};
use serde::Deserialize;

pub fn routes() -> Router {
    Router::new()
        .route("/db/connect", post(db_connect))
        .route("/db/disconnect", post(db_disconnect))
        .route("/db/find-all", post(db_find_all))
        .route("/db/find-by-id", post(db_find_by_id))
        .route("/db/insert", post(db_insert))
        .route("/db/update", post(db_update))
        .route("/db/delete", post(db_delete))
}

#[derive(Deserialize)]
struct ConnectBody { path: String }

async fn db_connect(Json(b): Json<ConnectBody>) -> Json<super::EmptyResponse> {
    api_empty!(crate::db::db_connect(b.path))
}

async fn db_disconnect() -> Json<super::EmptyResponse> {
    api_empty!(crate::db::db_disconnect())
}

#[derive(Deserialize)]
struct FindAllBody { table: String, params: crate::db::QueryParams, db_type: String }

async fn db_find_all(Json(b): Json<FindAllBody>) -> Json<super::ApiResponse<crate::db::QueryResult>> {
    api_ok!(crate::db::db_find_all(b.table, b.params, b.db_type))
}

#[derive(Deserialize)]
struct FindByIdBody { table: String, id: String, db_type: String }

async fn db_find_by_id(Json(b): Json<FindByIdBody>) -> Json<super::ApiResponse<Option<crate::db::DbRow>>> {
    api_ok!(crate::db::db_find_by_id(b.table, b.id, b.db_type))
}

#[derive(Deserialize)]
struct InsertBody { table: String, data: crate::db::DbRow, db_type: String }

async fn db_insert(Json(b): Json<InsertBody>) -> Json<super::ApiResponse<String>> {
    api_ok!(crate::db::db_insert(b.table, b.data, b.db_type))
}

#[derive(Deserialize)]
struct UpdateBody { table: String, id: String, data: crate::db::DbRow, db_type: String }

async fn db_update(Json(b): Json<UpdateBody>) -> Json<super::EmptyResponse> {
    api_empty!(crate::db::db_update(b.table, b.id, b.data, b.db_type))
}

#[derive(Deserialize)]
struct DeleteBody { table: String, id: String, db_type: String }

async fn db_delete(Json(b): Json<DeleteBody>) -> Json<super::EmptyResponse> {
    api_empty!(crate::db::db_delete(b.table, b.id, b.db_type))
}

// db_execute_raw is intentionally excluded from the HTTP API.
// It executes arbitrary SQL and is only safe via Tauri IPC (migrations).
