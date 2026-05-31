use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "agent_artifact")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub artifact_id: String,
    pub turn_id: String,
    pub artifact_type: String,
    pub file_path: String,
    pub mime_type: String,
    pub size_bytes: i32,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::agent_turn::Entity",
        from = "Column::TurnId",
        to = "super::agent_turn::Column::TurnId"
    )]
    Turn,
}

impl ActiveModelBehavior for ActiveModel {}
