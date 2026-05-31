use axum::{
    extract::Query,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

pub fn routes() -> Router {
    Router::new()
        .route("/vec/connect", post(vec_connect))
        .route("/vec/create-table", post(vec_create_table))
        .route("/vec/insert", post(vec_insert))
        .route("/vec/search", post(vec_search))
        .route("/vec/delete", post(vec_delete))
        .route("/vec/tables", get(vec_list_tables))
        .route("/vec/table-info", get(vec_table_info))
}

#[derive(Deserialize)]
struct ConnectBody {
    path: String,
}

async fn vec_connect(Json(b): Json<ConnectBody>) -> Json<super::EmptyResponse> {
    api_empty!(crate::vector::vec_connect(b.path))
}

#[derive(Deserialize)]
struct CreateTableBody {
    name: String,
    dimension: u32,
    vec_type: String,
}

async fn vec_create_table(Json(b): Json<CreateTableBody>) -> Json<super::EmptyResponse> {
    api_empty!(crate::vector::vec_create_table(
        b.name,
        b.dimension,
        b.vec_type
    ))
}

#[derive(Deserialize)]
struct InsertBody {
    table: String,
    id: String,
    vector: Vec<f32>,
    vec_type: String,
}

async fn vec_insert(Json(b): Json<InsertBody>) -> Json<super::EmptyResponse> {
    api_empty!(crate::vector::vec_insert(
        b.table, b.id, b.vector, b.vec_type
    ))
}

#[derive(Deserialize)]
struct SearchBody {
    table: String,
    query_vector: Vec<f32>,
    top_k: usize,
    vec_type: String,
}

async fn vec_search(
    Json(b): Json<SearchBody>,
) -> Json<super::ApiResponse<Vec<crate::vector::SearchResult>>> {
    api_ok!(crate::vector::vec_search(
        b.table,
        b.query_vector,
        b.top_k,
        b.vec_type
    ))
}

#[derive(Deserialize)]
struct VecDeleteBody {
    table: String,
    id: String,
    vec_type: String,
}

async fn vec_delete(Json(b): Json<VecDeleteBody>) -> Json<super::EmptyResponse> {
    api_empty!(crate::vector::vec_delete(b.table, b.id, b.vec_type))
}

#[derive(Deserialize)]
struct TablesQuery {
    vec_type: String,
}

async fn vec_list_tables(Query(q): Query<TablesQuery>) -> Json<super::ApiResponse<Vec<String>>> {
    api_ok!(crate::vector::vec_list_tables(q.vec_type))
}

#[derive(Deserialize)]
struct TableInfoQuery {
    table: String,
    vec_type: String,
}

async fn vec_table_info(
    Query(q): Query<TableInfoQuery>,
) -> Json<super::ApiResponse<crate::vector::TableInfo>> {
    api_ok!(crate::vector::vec_table_info(q.table, q.vec_type))
}
