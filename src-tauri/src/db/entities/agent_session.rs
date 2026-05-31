use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "agent_session")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub session_id: String,
    pub title: String,
    pub goal: String,
    pub project_id: String,
    pub runtime_type: String,
    pub default_model: String,
    pub current_state: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::agent_turn::Entity")]
    Turns,
    #[sea_orm(has_many = "super::agent_decision::Entity")]
    Decisions,
    #[sea_orm(has_many = "super::agent_context_snapshot::Entity")]
    ContextSnapshots,
}

impl ActiveModelBehavior for ActiveModel {}
