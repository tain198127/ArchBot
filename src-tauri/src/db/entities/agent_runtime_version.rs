use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "agent_runtime_version")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment)]
    pub id: i32,
    pub runtime_id: i32,
    pub version: String,
    pub install_path: String,
    pub checksum: String,
    pub status: String,
    pub installed_at: String,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::agent_runtime::Entity",
        from = "Column::RuntimeId",
        to = "super::agent_runtime::Column::Id"
    )]
    Runtime,
}

impl ActiveModelBehavior for ActiveModel {}
