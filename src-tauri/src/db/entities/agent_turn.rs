use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "agent_turn")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub turn_id: String,
    pub session_id: String,
    pub sequence_number: i32,
    pub user_message: String,
    pub interpreted_intent: String,
    pub input_file_path: String,
    pub prompt_file_path: String,
    pub status: String,
    pub runtime_type: String,
    pub runtime_version: String,
    pub model: String,
    pub started_at: String,
    pub finished_at: String,
    pub error_message: String,
    pub duration_ms: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::agent_session::Entity",
        from = "Column::SessionId",
        to = "super::agent_session::Column::SessionId"
    )]
    Session,
    #[sea_orm(has_many = "super::agent_event::Entity")]
    Events,
    #[sea_orm(has_many = "super::agent_artifact::Entity")]
    Artifacts,
    #[sea_orm(has_many = "super::agent_file_change::Entity")]
    FileChanges,
}

impl ActiveModelBehavior for ActiveModel {}
