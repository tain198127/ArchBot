//! 本地 SQLite 数据库后端
//!
//! 使用 SeaORM + SQLite 实现 [`DbBackend`] trait。
//! 支持连接管理、WAL 模式优化和通用 CRUD 操作。

use async_trait::async_trait;
use sea_orm::{
    ConnectionTrait, Database, DatabaseConnection, DbBackend as SeaDbBackend, Statement,
};
use serde_json::Value;

use super::{DbBackend, DbRow, QueryParams, QueryResult};

/// 本地 SQLite 后端
///
/// 封装 SeaORM 数据库连接，所有操作通过原始 SQL + 参数绑定执行。
/// 使用 WAL 模式提升并发读性能。
pub struct LocalSqliteDb {
    conn: DatabaseConnection,
}

impl LocalSqliteDb {
    /// 连接到 SQLite 数据库
    ///
    /// 数据库文件不存在时自动创建（`mode=rwc`）。
    /// 连接后立即设置 WAL 日志模式。
    pub async fn connect(path: &str) -> Result<Self, String> {
        let conn = Database::connect(&format!("sqlite:{}?mode=rwc", path))
            .await
            .map_err(|e| format!("连接 SQLite 失败: {e}"))?;

        conn.execute(Statement::from_string(
            SeaDbBackend::Sqlite,
            "PRAGMA journal_mode=WAL;",
        ))
        .await
        .map_err(|e| format!("设置 WAL 模式失败: {e}"))?;

        Ok(Self { conn })
    }

    /// 构建 WHERE 子句
    ///
    /// 业务逻辑：
    /// 1. 遍历所有 Filter，根据 operator 生成 SQL 片段
    /// 2. `in` 运算符特殊处理：展开数组为多个占位符
    /// 3. 返回 (WHERE 子句, 绑定参数值列表)
    fn build_where(params: &QueryParams) -> (String, Vec<Value>) {
        if params.filters.is_empty() {
            return (String::new(), vec![]);
        }
        let mut clauses = Vec::new();
        let mut values = Vec::new();
        for f in &params.filters {
            match f.operator.as_str() {
                "in" => {
                    if let Value::Array(arr) = &f.value {
                        let ph: Vec<String> = arr
                            .iter()
                            .enumerate()
                            .map(|(j, _)| format!("?{}", values.len() + j + 1))
                            .collect();
                        for v in arr {
                            values.push(v.clone());
                        }
                        clauses.push(format!("`{}` IN ({})", f.field, ph.join(", ")));
                    } else {
                        values.push(f.value.clone());
                        clauses.push(format!("`{}` = ?{}", f.field, values.len()));
                    }
                }
                _ => {
                    values.push(f.value.clone());
                    let op = match f.operator.as_str() {
                        "neq" => "!=",
                        "gt" => ">",
                        "gte" => ">=",
                        "lt" => "<",
                        "lte" => "<=",
                        "like" => "LIKE",
                        _ => "=",
                    };
                    clauses.push(format!("`{}` {} ?{}", f.field, op, values.len()));
                }
            }
        }
        (format!("WHERE {}", clauses.join(" AND ")), values)
    }

    /// 构建 ORDER BY 子句
    fn build_order(params: &QueryParams) -> String {
        if params.order_by.is_empty() {
            return String::new();
        }
        let parts: Vec<String> = params
            .order_by
            .iter()
            .map(|o| format!("`{}` {}", o.field, if o.descending { "DESC" } else { "ASC" }))
            .collect();
        format!("ORDER BY {}", parts.join(", "))
    }

    /// 构建 LIMIT / OFFSET 子句
    fn build_limit(params: &QueryParams) -> String {
        match (params.limit, params.offset) {
            (Some(limit), Some(offset)) => format!("LIMIT {} OFFSET {}", limit, offset),
            (Some(limit), None) => format!("LIMIT {}", limit),
            (None, Some(offset)) => format!("LIMIT -1 OFFSET {}", offset),
            (None, None) => String::new(),
        }
    }
}

#[async_trait]
impl DbBackend for LocalSqliteDb {
    async fn find_by_id(&self, table: &str, id: &str) -> Result<Option<DbRow>, String> {
        let sql = format!("SELECT * FROM `{}` WHERE id = ?", table);
        let rows = self
            .conn
            .query_all(Statement::from_sql_and_values(
                SeaDbBackend::Sqlite,
                &sql,
                vec![id.into()],
            ))
            .await
            .map_err(|e| format!("查询失败: {e}"))?;

        if rows.is_empty() {
            return Ok(None);
        }
        Ok(Some(query_result_to_dbrow(&rows[0])))
    }

    async fn find_all(&self, table: &str, params: QueryParams) -> Result<QueryResult, String> {
        let (where_clause, values) = Self::build_where(&params);
        let order_clause = Self::build_order(&params);
        let limit_clause = Self::build_limit(&params);

        // 总数
        let total: u64 = {
            let count_sql = format!("SELECT COUNT(*) as cnt FROM `{}` {}", table, where_clause);
            let has_filters = !values.is_empty();
            if has_filters {
                let sea_vals: Vec<sea_orm::Value> =
                    values.iter().map(sea_value_from_json).collect();
                let opt: Option<sea_orm::QueryResult> = self
                    .conn
                    .query_one(Statement::from_sql_and_values(
                        SeaDbBackend::Sqlite,
                        &count_sql,
                        sea_vals,
                    ))
                    .await
                    .map_err(|e| format!("计数查询失败: {e}"))?;
                opt.map(|r| r.try_get_by_index::<i64>(0).unwrap_or(0) as u64)
                    .unwrap_or(0)
            } else {
                let count_raw = format!("SELECT COUNT(*) FROM `{}`", table);
                let opt: Option<sea_orm::QueryResult> = self
                    .conn
                    .query_one(Statement::from_string(SeaDbBackend::Sqlite, &count_raw))
                    .await
                    .map_err(|e| format!("计数查询失败: {e}"))?;
                opt.map(|r| r.try_get_by_index::<i64>(0).unwrap_or(0) as u64)
                    .unwrap_or(0)
            }
        };

        // 查数据
        let sql = format!(
            "SELECT * FROM `{}` {} {} {}",
            table, where_clause, order_clause, limit_clause
        );
        let sea_vals: Vec<sea_orm::Value> = values.iter().map(sea_value_from_json).collect();
        let rows = self
            .conn
            .query_all(Statement::from_sql_and_values(
                SeaDbBackend::Sqlite,
                &sql,
                sea_vals,
            ))
            .await
            .map_err(|e| format!("查询失败: {e}"))?;

        let db_rows: Vec<DbRow> = rows.iter().map(query_result_to_dbrow).collect();
        Ok(QueryResult {
            rows: db_rows,
            total,
        })
    }

    async fn insert(&self, table: &str, data: DbRow) -> Result<String, String> {
        let fields: Vec<String> = data.keys().map(|k| format!("`{}`", k)).collect();
        let placeholders: Vec<String> = (1..=data.len()).map(|i| format!("?{}", i)).collect();
        let values: Vec<Value> = data.values().cloned().collect();

        let sql = format!(
            "INSERT INTO `{}` ({}) VALUES ({})",
            table,
            fields.join(", "),
            placeholders.join(", ")
        );

        let sea_vals: Vec<sea_orm::Value> = values.iter().map(sea_value_from_json).collect();
        self.conn
            .execute(Statement::from_sql_and_values(
                SeaDbBackend::Sqlite,
                &sql,
                sea_vals,
            ))
            .await
            .map_err(|e| format!("插入失败: {e}"))?;

        // 获取自增 id
        let opt: Option<sea_orm::QueryResult> = self
            .conn
            .query_one(Statement::from_string(
                SeaDbBackend::Sqlite,
                "SELECT last_insert_rowid()",
            ))
            .await
            .map_err(|e| format!("获取自增 id 失败: {e}"))?;
        let last_id = opt
            .and_then(|r| r.try_get_by_index::<i64>(0).ok())
            .unwrap_or(0);
        Ok(last_id.to_string())
    }

    async fn update(&self, table: &str, id: &str, data: DbRow) -> Result<(), String> {
        let sets: Vec<String> = data
            .keys()
            .enumerate()
            .map(|(i, k)| format!("`{}` = ?{}", k, i + 1))
            .collect();
        let mut values: Vec<Value> = data.values().cloned().collect();
        values.push(Value::String(id.to_string()));
        let id_idx = values.len();

        let sql = format!(
            "UPDATE `{}` SET {} WHERE id = ?{}",
            table,
            sets.join(", "),
            id_idx
        );

        let sea_vals: Vec<sea_orm::Value> = values.iter().map(sea_value_from_json).collect();
        self.conn
            .execute(Statement::from_sql_and_values(
                SeaDbBackend::Sqlite,
                &sql,
                sea_vals,
            ))
            .await
            .map_err(|e| format!("更新失败: {e}"))?;
        Ok(())
    }

    async fn delete(&self, table: &str, id: &str) -> Result<(), String> {
        let sql = format!("DELETE FROM `{}` WHERE id = ?", table);
        self.conn
            .execute(Statement::from_sql_and_values(
                SeaDbBackend::Sqlite,
                &sql,
                vec![id.into()],
            ))
            .await
            .map_err(|e| format!("删除失败: {e}"))?;
        Ok(())
    }

    async fn execute_raw(&self, sql: &str) -> Result<QueryResult, String> {
        let rows = self
            .conn
            .query_all(Statement::from_string(SeaDbBackend::Sqlite, sql))
            .await
            .map_err(|e| format!("执行 SQL 失败: {e}"))?;

        let db_rows: Vec<DbRow> = rows.iter().map(query_result_to_dbrow).collect();
        let total = db_rows.len() as u64;
        Ok(QueryResult {
            rows: db_rows,
            total,
        })
    }
}

// ─── Helpers ──────────────────────────────────────────────────

/// 将 SeaORM QueryResult 行转为 DbRow（HashMap<String, Value>）
///
/// 通过 JSON 序列化/反序列化中转实现列名到值的映射。
/// 生产环境建议使用预定义的 SeaORM Entity 来获得编译期类型安全。
fn query_result_to_dbrow(row: &sea_orm::QueryResult) -> DbRow {
    // SeaORM QueryResult 不支持按名称遍历列，这里用 JSON 兜底
    let json_val: Value = row
        .try_get::<serde_json::Value>("", "")
        .unwrap_or(Value::Null);
    if let Value::Object(obj) = json_val {
        return obj.into_iter().collect();
    }
    DbRow::new()
}

/// 将 serde_json::Value 转为 sea_orm::Value
fn sea_value_from_json(v: &Value) -> sea_orm::Value {
    match v {
        Value::Null => sea_orm::Value::String(None),
        Value::Bool(b) => sea_orm::Value::Bool(Some(*b)),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                sea_orm::Value::BigInt(Some(i))
            } else {
                sea_orm::Value::Double(n.as_f64())
            }
        }
        Value::String(s) => sea_orm::Value::String(Some(Box::new(s.clone()))),
        Value::Array(_) | Value::Object(_) => {
            sea_orm::Value::String(Some(Box::new(v.to_string())))
        }
    }
}
