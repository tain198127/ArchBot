//! SeaORM Entity 定义
//!
//! 每个实体文件对应一张数据库表。所有实体通过 `id` (自增主键) +
//! `code` (业务唯一键) 标识。关系表通过 `code` 列关联。
//!
//! ## 使用方式
//! - 类型安全查询: `Entity::find().filter(Column::Code.eq("ba-analyst"))`
//! - 通过现有 `DbBackend` trait 的泛型接口: `db_insert("digital_employees", row)`
//! - Entity 层提供类型校验和结构定义，最终通过 `DbBackend` 落库

pub mod agent;
pub mod approval_queue;
pub mod digital_employee;
pub mod employee_agent;
pub mod employee_handoff;
pub mod employee_mcp;
pub mod employee_skill;
pub mod mcp;
pub mod skill;
