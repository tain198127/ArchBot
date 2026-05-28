//! 数据库 Repository 抽象
//!
//! 定义统一的数据库操作 trait，Local (`SeaORM + SQLite`) 和 Remote (`REST API`)
//! 两种实现共享此接口。前端通过 Tauri commands 调用，由 `db_type` 参数选择后端。
//!
//! ## 架构
//! ```text
//! Tauri commands
//!   ├── db_find_all / db_find_by_id / db_insert / db_update / db_delete / db_execute
//!   │     └── dispatch_db! macro → LocalSqliteDb 或 RemoteMySqlDb
//!   └── db_configure_remote
//! ```

pub mod local_sqlite;
pub mod remote_mysql;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::OnceLock;
use tokio::sync::Mutex;

use local_sqlite::LocalSqliteDb;
use remote_mysql::RemoteMySqlDb;

// ─── Types ────────────────────────────────────────────────────

/// 通用查询结果行：键值对
pub type DbRow = std::collections::HashMap<String, Value>;

/// 过滤条件
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Filter {
    pub field: String,
    pub operator: String, // "eq", "neq", "gt", "gte", "lt", "lte", "like", "in"
    pub value: Value,
}

/// 排序规则
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrderBy {
    pub field: String,
    pub descending: bool,
}

/// 查询参数
#[derive(Serialize, Deserialize, Clone, Debug)]
#[derive(Default)]
pub struct QueryParams {
    pub filters: Vec<Filter>,
    pub order_by: Vec<OrderBy>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}


/// 查询结果集
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QueryResult {
    pub rows: Vec<DbRow>,
    pub total: u64,
}

/// 数据库后端抽象 trait
///
/// 定义 6 个统一的数据库操作方法。每种操作有 Local 和 Remote 两种实现：
/// - `LocalSqliteDb` 使用 SeaORM + SQLite
/// - `RemoteMySqlDb` 使用 reqwest 调用 REST API
#[async_trait]
pub trait DbBackend: Send + Sync {
    /// 根据主键 id 查询单条记录
    async fn find_by_id(&self, table: &str, id: &str) -> Result<Option<DbRow>, String>;

    /// 条件查询，返回分页结果
    async fn find_all(&self, table: &str, params: QueryParams) -> Result<QueryResult, String>;

    /// 插入一条记录，返回自增 id
    async fn insert(&self, table: &str, data: DbRow) -> Result<String, String>;

    /// 根据主键 id 更新记录
    async fn update(&self, table: &str, id: &str, data: DbRow) -> Result<(), String>;

    /// 根据主键 id 删除记录
    async fn delete(&self, table: &str, id: &str) -> Result<(), String>;

    /// 执行原始 SQL 并返回结果
    async fn execute_raw(&self, sql: &str) -> Result<QueryResult, String>;
}

// ─── Global backend instances ──────────────────────────────────

static LOCAL_DB: OnceLock<Mutex<Option<LocalSqliteDb>>> = OnceLock::new();
static REMOTE_DB: OnceLock<Mutex<Option<RemoteMySqlDb>>> = OnceLock::new();

fn local_db_cell() -> &'static Mutex<Option<LocalSqliteDb>> {
    LOCAL_DB.get_or_init(|| Mutex::new(None))
}

fn remote_db_cell() -> &'static Mutex<Option<RemoteMySqlDb>> {
    REMOTE_DB.get_or_init(|| Mutex::new(None))
}

// ─── Dispatch macro ───────────────────────────────────────────

macro_rules! dispatch_db {
    ($db_type:expr, $method:ident, $($arg:expr),*) => {{
        match $db_type.as_str() {
            "local" => {
                let guard = local_db_cell().lock().await;
                let backend = guard.as_ref().ok_or("本地数据库未连接，请先调用 db_connect")?;
                backend.$method($($arg),*).await
            }
            "remote" => {
                let guard = remote_db_cell().lock().await;
                let backend = guard.as_ref().ok_or("远程数据库未配置，请先调用 db_configure_remote")?;
                backend.$method($($arg),*).await
            }
            _ => Err(format!("未知的 db_type: {}", $db_type))
        }
    }};
}

// ═══════════════════════════════════════════════════════════════
// Tauri Commands
// ═══════════════════════════════════════════════════════════════

/// 连接或创建本地 SQLite 数据库
///
/// `path` 为数据库文件路径（如 `~/.ArchBot/archbot.db`）。
/// 文件不存在时自动创建。
#[tauri::command]
pub async fn db_connect(path: String) -> Result<(), String> {
    let db = LocalSqliteDb::connect(&path).await?;
    let mut guard = local_db_cell().lock().await;
    *guard = Some(db);
    Ok(())
}

/// 配置远程数据库后端
///
/// `base_url` 为 REST API 根地址。
/// `token` 为可选的 Bearer token。
#[tauri::command]
pub async fn db_configure_remote(base_url: String, token: Option<String>) -> Result<(), String> {
    let mut guard = remote_db_cell().lock().await;
    *guard = Some(RemoteMySqlDb::new(&base_url, token.as_deref()));
    Ok(())
}

/// 条件查询
#[tauri::command]
pub async fn db_find_all(
    table: String,
    params: QueryParams,
    db_type: String,
) -> Result<QueryResult, String> {
    validate_identifier(&table)?;
    for f in &params.filters {
        validate_identifier(&f.field)?;
    }
    for o in &params.order_by {
        validate_identifier(&o.field)?;
    }
    dispatch_db!(db_type, find_all, &table, params)
}

/// 按 id 查询
#[tauri::command]
pub async fn db_find_by_id(
    table: String,
    id: String,
    db_type: String,
) -> Result<Option<DbRow>, String> {
    validate_identifier(&table)?;
    dispatch_db!(db_type, find_by_id, &table, &id)
}

/// 插入记录
#[tauri::command]
pub async fn db_insert(
    table: String,
    data: DbRow,
    db_type: String,
) -> Result<String, String> {
    validate_identifier(&table)?;
    for k in data.keys() {
        validate_identifier(k)?;
    }
    dispatch_db!(db_type, insert, &table, data)
}

/// 更新记录
#[tauri::command]
pub async fn db_update(
    table: String,
    id: String,
    data: DbRow,
    db_type: String,
) -> Result<(), String> {
    validate_identifier(&table)?;
    for k in data.keys() {
        validate_identifier(k)?;
    }
    dispatch_db!(db_type, update, &table, &id, data)
}

/// 删除记录
#[tauri::command]
pub async fn db_delete(table: String, id: String, db_type: String) -> Result<(), String> {
    validate_identifier(&table)?;
    dispatch_db!(db_type, delete, &table, &id)
}

/// 执行原始 SQL
#[tauri::command]
pub async fn db_execute_raw(sql: String, db_type: String) -> Result<QueryResult, String> {
    dispatch_db!(db_type, execute_raw, &sql)
}

// ─── Identifier validation ────────────────────────────────────

/// 校验 SQL 标识符（表名、字段名）防止注入
///
/// 仅允许字母、数字、下划线，且必须以字母或下划线开头。
pub(crate) fn validate_identifier(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 64 {
        return Err(format!("标识符长度不合法: {name}"));
    }
    let mut chars = name.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' {
        return Err(format!("标识符必须以字母或下划线开头: {name}"));
    }
    for c in chars {
        if !c.is_ascii_alphanumeric() && c != '_' {
            return Err(format!("标识符含非法字符: {name}"));
        }
    }
    Ok(())
}
