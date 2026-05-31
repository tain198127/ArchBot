use axum::{extract::Query, routing::get, Json, Router};
use serde::Deserialize;

pub fn routes() -> Router {
    Router::new()
        .route("/ds/domains", get(list_domains).post(create_domain))
        .route("/ds/domains/{domain_code}", get(load_domain))
        .route("/ds/conventions", get(load_conventions))
        .route(
            "/ds/entities",
            get(list_entities).post(save_entity).delete(delete_entity),
        )
        .route(
            "/ds/enums",
            get(list_enums).post(save_enum).delete(delete_enum),
        )
}

#[derive(Deserialize)]
struct ListDomainsQuery {
    project_dir: String,
}

async fn list_domains(
    Query(q): Query<ListDomainsQuery>,
) -> Json<super::ApiResponse<Vec<crate::data_standard::DomainInfo>>> {
    api_ok!(crate::data_standard::ds_list_domains(q.project_dir))
}

#[derive(Deserialize)]
struct CreateDomainBody {
    project_dir: String,
    code: String,
    name: String,
    owner: String,
    description: String,
}

async fn create_domain(Json(b): Json<CreateDomainBody>) -> Json<super::ApiResponse<String>> {
    api_ok!(crate::data_standard::ds_create_domain(
        b.project_dir,
        b.code,
        b.name,
        b.owner,
        b.description
    ))
}

#[derive(Deserialize)]
struct LoadDomainQuery {
    project_dir: String,
}

async fn load_domain(
    axum::extract::Path(domain_code): axum::extract::Path<String>,
    Query(q): Query<LoadDomainQuery>,
) -> Json<super::ApiResponse<crate::data_standard::DomainInfo>> {
    api_ok!(crate::data_standard::ds_load_domain(
        q.project_dir,
        domain_code
    ))
}

#[derive(Deserialize)]
struct LoadConventionsQuery {
    project_dir: String,
    domain_code: String,
}

async fn load_conventions(
    Query(q): Query<LoadConventionsQuery>,
) -> Json<super::ApiResponse<crate::data_standard::Conventions>> {
    api_ok!(crate::data_standard::ds_load_conventions(
        q.project_dir,
        q.domain_code
    ))
}

#[derive(Deserialize)]
struct ListEntitiesQuery {
    project_dir: String,
    domain_code: String,
}

async fn list_entities(
    Query(q): Query<ListEntitiesQuery>,
) -> Json<super::ApiResponse<Vec<crate::data_standard::EntityDef>>> {
    api_ok!(crate::data_standard::ds_list_entities(
        q.project_dir,
        q.domain_code
    ))
}

#[derive(Deserialize)]
struct SaveEntityBody {
    project_dir: String,
    domain_code: String,
    entity: crate::data_standard::EntityDef,
}

async fn save_entity(
    Json(b): Json<SaveEntityBody>,
) -> Json<super::ApiResponse<crate::data_standard::EntityDef>> {
    api_ok!(crate::data_standard::ds_save_entity(
        b.project_dir,
        b.domain_code,
        b.entity
    ))
}

#[derive(Deserialize)]
struct DeleteEntityBody {
    project_dir: String,
    domain_code: String,
    entity_code: String,
}

async fn delete_entity(Json(b): Json<DeleteEntityBody>) -> Json<super::EmptyResponse> {
    api_empty!(crate::data_standard::ds_delete_entity(
        b.project_dir,
        b.domain_code,
        b.entity_code
    ))
}

#[derive(Deserialize)]
struct ListEnumsQuery {
    project_dir: String,
    domain_code: String,
}

async fn list_enums(
    Query(q): Query<ListEnumsQuery>,
) -> Json<super::ApiResponse<Vec<crate::data_standard::EnumDef>>> {
    api_ok!(crate::data_standard::ds_list_enums(
        q.project_dir,
        q.domain_code
    ))
}

#[derive(Deserialize)]
struct SaveEnumBody {
    project_dir: String,
    domain_code: String,
    enum_def: crate::data_standard::EnumDef,
}

async fn save_enum(
    Json(b): Json<SaveEnumBody>,
) -> Json<super::ApiResponse<crate::data_standard::EnumDef>> {
    api_ok!(crate::data_standard::ds_save_enum(
        b.project_dir,
        b.domain_code,
        b.enum_def
    ))
}

#[derive(Deserialize)]
struct DeleteEnumBody {
    project_dir: String,
    domain_code: String,
    enum_code: String,
}

async fn delete_enum(Json(b): Json<DeleteEnumBody>) -> Json<super::EmptyResponse> {
    api_empty!(crate::data_standard::ds_delete_enum(
        b.project_dir,
        b.domain_code,
        b.enum_code
    ))
}
