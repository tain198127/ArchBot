use axum::{extract::Query, routing::get, Json, Router};
use serde::Deserialize;

pub fn routes() -> Router {
    Router::new()
        .route("/context/config", get(get_config).post(save_config))
        .route("/context/entries", get(list_entries))
        .route(
            "/context/entry",
            get(get_entry).post(save_entry).delete(delete_entry),
        )
}

#[derive(Deserialize)]
struct ConfigQuery {
    project_path: String,
    section: String,
}

async fn get_config(Query(q): Query<ConfigQuery>) -> Json<super::ApiResponse<String>> {
    api_ok_sync!(crate::context::get_context_config(
        q.project_path,
        q.section
    ))
}

#[derive(Deserialize)]
struct SaveConfigBody {
    project_path: String,
    section: String,
    content: String,
}

async fn save_config(Json(b): Json<SaveConfigBody>) -> Json<super::EmptyResponse> {
    api_empty_sync!(crate::context::save_context_config(
        b.project_path,
        b.section,
        b.content
    ))
}

#[derive(Deserialize)]
struct EntriesQuery {
    project_path: String,
    section: String,
}

async fn list_entries(Query(q): Query<EntriesQuery>) -> Json<super::ApiResponse<Vec<String>>> {
    api_ok_sync!(crate::context::list_context_entries(
        q.project_path,
        q.section
    ))
}

#[derive(Deserialize)]
struct EntryQuery {
    project_path: String,
    section: String,
    name: String,
}

async fn get_entry(
    Query(q): Query<EntryQuery>,
) -> Json<super::ApiResponse<crate::context::ContextEntry>> {
    api_ok_sync!(crate::context::get_context_entry(
        q.project_path,
        q.section,
        q.name
    ))
}

#[derive(Deserialize)]
struct SaveEntryBody {
    project_path: String,
    section: String,
    entry: crate::context::ContextEntry,
}

async fn save_entry(Json(b): Json<SaveEntryBody>) -> Json<super::EmptyResponse> {
    api_empty_sync!(crate::context::save_context_entry(
        b.project_path,
        b.section,
        b.entry
    ))
}

#[derive(Deserialize)]
struct DeleteEntryBody {
    project_path: String,
    section: String,
    name: String,
}

async fn delete_entry(Json(b): Json<DeleteEntryBody>) -> Json<super::EmptyResponse> {
    api_empty_sync!(crate::context::delete_context_entry(
        b.project_path,
        b.section,
        b.name
    ))
}
