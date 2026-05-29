use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "agents")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment)]
    pub id: i32,
    pub code: String,
    pub name: String,
    pub r#type: String,
    pub source_path: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::employee_agent::Entity")]
    EmployeeAgents,
}

impl ActiveModelBehavior for ActiveModel {}
