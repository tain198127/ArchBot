use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "agent_runtime")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment)]
    pub id: i32,
    pub runtime_type: String,
    pub enabled: bool,
    pub mode: String,
    pub current_version: String,
    pub executable_path: String,
    pub adapter_config: String,
    pub provider_config: String,
    pub model_config: String,
    pub env_vars: String,
    pub execution_config: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::agent_runtime_version::Entity")]
    Versions,
    #[sea_orm(has_many = "super::agent_adapter::Entity")]
    Adapters,
}

impl ActiveModelBehavior for ActiveModel {}
