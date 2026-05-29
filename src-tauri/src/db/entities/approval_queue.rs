use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "approval_queue")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment)]
    pub id: i32,
    pub from_employee_code: String,
    pub to_employee_code: String,
    pub operation_type: String,
    pub source_artifact: String,
    pub status: String,
    #[sea_orm(column_type = "Text")]
    pub result_data: String,
    pub project_code: String,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
