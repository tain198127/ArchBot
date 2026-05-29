use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "employee_mcps")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment)]
    pub id: i32,
    pub employee_code: String,
    pub mcp_code: String,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::digital_employee::Entity",
        from = "Column::EmployeeCode",
        to = "super::digital_employee::Column::Code"
    )]
    Employee,
    #[sea_orm(
        belongs_to = "super::mcp::Entity",
        from = "Column::McpCode",
        to = "super::mcp::Column::Code"
    )]
    Mcp,
}

impl ActiveModelBehavior for ActiveModel {}
