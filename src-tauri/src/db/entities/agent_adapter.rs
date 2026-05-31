use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "agent_adapter")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment)]
    pub id: i32,
    pub runtime_id: i32,
    pub adapter_type: String,
    pub host: String,
    pub port: i32,
    pub auth_token_hash: String,
    pub status: String,
    pub pid: i32,
    pub started_at: String,
    pub stopped_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::agent_runtime::Entity",
        from = "Column::RuntimeId",
        to = "super::agent_runtime::Column::Id"
    )]
    Runtime,
}

impl ActiveModelBehavior for ActiveModel {}
