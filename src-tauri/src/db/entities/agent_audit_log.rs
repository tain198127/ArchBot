use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "agent_audit_log")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub log_id: String,
    pub session_id: String,
    pub turn_id: String,
    pub event_type: String,
    pub actor: String,
    pub action: String,
    pub detail: String,
    pub severity: String,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
