use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "employee_handoffs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment)]
    pub id: i32,
    pub employee_code: String,
    pub trigger_op: String,
    pub target_employee_code: String,
    #[sea_orm(column_type = "Text")]
    pub transfer_data: String,
    pub transfer_mode: String,
    #[sea_orm(column_type = "Text")]
    pub context_ref: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::digital_employee::Entity",
        from = "Column::EmployeeCode",
        to = "super::digital_employee::Column::Code"
    )]
    Employee,
}

impl ActiveModelBehavior for ActiveModel {}
