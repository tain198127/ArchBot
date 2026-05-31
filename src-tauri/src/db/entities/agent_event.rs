use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "agent_event")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub event_id: String,
    pub session_id: String,
    pub turn_id: String,
    pub event_type: String,
    pub sequence_number: i32,
    pub payload: String,
    pub timestamp: String,
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
