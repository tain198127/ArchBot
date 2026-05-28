//! 向量数据库抽象模块
//!
//! 提供统一的向量数据库操作接口：
//! - [`VectorBackend`] trait — 建表、插入、搜索、删除、列表
//! - `LocalVectorDb` 使用 LanceDB 嵌入式向量数据库
//! - `RemoteVectorDb` 通过 REST API 调用远程向量服务
//!
//! ## 架构
//! ```text
//! Tauri commands
//!   ├── vec_create_table / vec_insert / vec_search / vec_delete / vec_list_tables
//!   │     └── dispatch_vec! macro → LocalVectorDb 或 RemoteVectorDb
//!   └── vec_configure_remote
//! ```

pub mod local_vector;
pub mod remote_vector;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tokio::sync::Mutex;

use local_vector::LocalVectorDb;
use remote_vector::RemoteVectorDb;

// ─── Types ────────────────────────────────────────────────────

/// 搜索返回的单条结果
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchResult {
    pub id: String,
    /// 距离值（越小越相似）
    pub distance: f32,
    /// 可选的附加元数据
    pub metadata: Option<String>,
}

/// 向量表信息
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TableInfo {
    pub name: String,
    pub dimension: u32,
}

/// 向量数据库后端抽象 trait
///
/// 定义 6 个统一的向量数据库操作方法。
#[async_trait]
pub trait VectorBackend: Send + Sync {
    /// 创建向量表，指定名称和向量维度
    async fn create_table(&self, name: &str, dimension: u32) -> Result<(), String>;

    /// 插入一条向量数据
    async fn insert(&self, table: &str, id: &str, vector: Vec<f32>) -> Result<(), String>;

    /// ANN 向量搜索，返回 top_k 条最相似结果
    async fn search(
        &self,
        table: &str,
        query: Vec<f32>,
        top_k: usize,
    ) -> Result<Vec<SearchResult>, String>;

    /// 根据 id 删除一条向量
    async fn delete(&self, table: &str, id: &str) -> Result<(), String>;

    /// 列出所有表名
    async fn list_tables(&self) -> Result<Vec<String>, String>;

    /// 获取表信息（维度等）
    async fn table_info(&self, name: &str) -> Result<TableInfo, String>;
}

// ─── Global backend instances ──────────────────────────────────

static LOCAL_VEC: OnceLock<Mutex<Option<LocalVectorDb>>> = OnceLock::new();
static REMOTE_VEC: OnceLock<Mutex<Option<RemoteVectorDb>>> = OnceLock::new();

fn local_vec_cell() -> &'static Mutex<Option<LocalVectorDb>> {
    LOCAL_VEC.get_or_init(|| Mutex::new(None))
}

fn remote_vec_cell() -> &'static Mutex<Option<RemoteVectorDb>> {
    REMOTE_VEC.get_or_init(|| Mutex::new(None))
}

// ─── Dispatch macro ───────────────────────────────────────────

macro_rules! dispatch_vec {
    // 带参数版本
    ($vec_type:expr, $method:ident, $($arg:expr),+) => {{
        match $vec_type.as_str() {
            "local" => {
                let guard = local_vec_cell().lock().await;
                let backend = guard.as_ref().ok_or("本地向量库未连接，请先调用 vec_connect")?;
                backend.$method($($arg),*).await
            }
            "remote" => {
                let guard = remote_vec_cell().lock().await;
                let backend = guard.as_ref().ok_or("远程向量库未配置，请先调用 vec_configure_remote")?;
                backend.$method($($arg),*).await
            }
            _ => Err(format!("未知的 vec_type: {}", $vec_type))
        }
    }};
    // 无参数版本
    ($vec_type:expr, $method:ident) => {{
        match $vec_type.as_str() {
            "local" => {
                let guard = local_vec_cell().lock().await;
                let backend = guard.as_ref().ok_or("本地向量库未连接，请先调用 vec_connect")?;
                backend.$method().await
            }
            "remote" => {
                let guard = remote_vec_cell().lock().await;
                let backend = guard.as_ref().ok_or("远程向量库未配置，请先调用 vec_configure_remote")?;
                backend.$method().await
            }
            _ => Err(format!("未知的 vec_type: {}", $vec_type))
        }
    }};
}

// ─── Identifier validation ────────────────────────────────────

fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 64 {
        return Err(format!("名称长度不合法: {name}"));
    }
    for c in name.chars() {
        if !c.is_ascii_alphanumeric() && c != '_' && c != '-' {
            return Err(format!("名称含非法字符: {name}"));
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════
// Tauri Commands
// ═══════════════════════════════════════════════════════════════

/// 连接本地 LanceDB 向量数据库
///
/// `path` 为数据库文件存储目录。目录不存在时自动创建。
#[tauri::command]
pub async fn vec_connect(path: String) -> Result<(), String> {
    let db = LocalVectorDb::connect(&path).await?;
    let mut guard = local_vec_cell().lock().await;
    *guard = Some(db);
    Ok(())
}

/// 配置远程向量数据库后端
#[tauri::command]
pub async fn vec_configure_remote(base_url: String, token: Option<String>) -> Result<(), String> {
    let mut guard = remote_vec_cell().lock().await;
    *guard = Some(RemoteVectorDb::new(&base_url, token.as_deref()));
    Ok(())
}

/// 创建向量表
#[tauri::command]
pub async fn vec_create_table(
    name: String,
    dimension: u32,
    vec_type: String,
) -> Result<(), String> {
    validate_name(&name)?;
    dispatch_vec!(vec_type, create_table, &name, dimension)
}

/// 插入向量
#[tauri::command]
pub async fn vec_insert(
    table: String,
    id: String,
    vector: Vec<f32>,
    vec_type: String,
) -> Result<(), String> {
    validate_name(&table)?;
    dispatch_vec!(vec_type, insert, &table, &id, vector)
}

/// 向量搜索
#[tauri::command]
pub async fn vec_search(
    table: String,
    query_vector: Vec<f32>,
    top_k: usize,
    vec_type: String,
) -> Result<Vec<SearchResult>, String> {
    validate_name(&table)?;
    dispatch_vec!(vec_type, search, &table, query_vector, top_k)
}

/// 删除向量
#[tauri::command]
pub async fn vec_delete(table: String, id: String, vec_type: String) -> Result<(), String> {
    validate_name(&table)?;
    dispatch_vec!(vec_type, delete, &table, &id)
}

/// 列出所有表
#[tauri::command]
pub async fn vec_list_tables(vec_type: String) -> Result<Vec<String>, String> {
    dispatch_vec!(vec_type, list_tables)
}

/// 获取表信息
#[tauri::command]
pub async fn vec_table_info(table: String, vec_type: String) -> Result<TableInfo, String> {
    validate_name(&table)?;
    dispatch_vec!(vec_type, table_info, &table)
}
