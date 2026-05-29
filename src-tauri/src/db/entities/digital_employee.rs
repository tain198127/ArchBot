use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "digital_employees")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment)]
    pub id: i32,
    pub code: String,
    pub name: String,
    pub is_builtin: bool,
    pub avatar: String,
    pub personality_tags: String,
    #[sea_orm(column_type = "Text")]
    pub personality_desc: String,
    pub comm_style: String,
    pub decision_pref: String,
    pub focus_areas: String,
    pub deliverable_groups: String,
    pub default_op: String,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::employee_skill::Entity")]
    Skills,
    #[sea_orm(has_many = "super::employee_agent::Entity")]
    Agents,
    #[sea_orm(has_many = "super::employee_mcp::Entity")]
    Mcps,
    #[sea_orm(has_many = "super::employee_handoff::Entity")]
    Handoffs,
}

impl ActiveModelBehavior for ActiveModel {}
