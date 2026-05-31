use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "agent_file_change")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub change_id: String,
    pub turn_id: String,
    pub file_path: String,
    pub change_type: String,
    pub diff_content: String,
    pub file_hash_before: String,
    pub file_hash_after: String,
    pub size_before: i32,
    pub size_after: i32,
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
