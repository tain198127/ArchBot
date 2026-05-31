use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "agent_context_snapshot")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub snapshot_id: String,
    pub session_id: String,
    pub turn_id: Option<String>,
    pub snapshot_type: String,
    pub content: String,
    pub token_count: i32,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::agent_session::Entity",
        from = "Column::SessionId",
        to = "super::agent_session::Column::SessionId"
    )]
    Session,
}

impl ActiveModelBehavior for ActiveModel {}
