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
            .map(|o| {
                format!(
                    "`{}` {}",
                    o.field,
                    if o.descending { "DESC" } else { "ASC" }
                )
            })
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
        let columns = get_table_columns(&self.conn, table).await?;
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
        Ok(Some(query_result_to_dbrow(&rows[0], &columns)))
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

        // 获取列名
        let columns = get_table_columns(&self.conn, table).await?;

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

        let db_rows: Vec<DbRow> = rows
            .iter()
            .map(|r| query_result_to_dbrow(r, &columns))
            .collect();
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

        let db_rows: Vec<DbRow> = rows.iter().map(|r| query_result_to_dbrow(r, &[])).collect();
        let total = db_rows.len() as u64;
        Ok(QueryResult {
            rows: db_rows,
            total,
        })
    }
}

// ─── Helpers ──────────────────────────────────────────────────

/// 通过 PRAGMA table_info 获取表的所有列名（按顺序）
async fn get_table_columns(conn: &DatabaseConnection, table: &str) -> Result<Vec<String>, String> {
    let sql = format!("PRAGMA table_info('{}')", table);
    let rows = conn
        .query_all(Statement::from_string(SeaDbBackend::Sqlite, &sql))
        .await
        .map_err(|e| format!("获取表结构失败: {e}"))?;

    let mut columns = Vec::new();
    for row in &rows {
        let name: String = row
            .try_get_by_index(1)
            .map_err(|e| format!("获取列名失败: {e}"))?;
        columns.push(name);
    }
    Ok(columns)
}

/// 将 SeaORM QueryResult 行转为 DbRow（HashMap<String, Value>）
///
/// 按索引遍历每一列，尝试 i64 → f64 → String → null 的类型解码顺序，
/// 匹配 SQLite 的 INTEGER / REAL / TEXT 三种亲和类型。
fn query_result_to_dbrow(row: &sea_orm::QueryResult, columns: &[String]) -> DbRow {
    let mut map = DbRow::new();
    for (i, col_name) in columns.iter().enumerate() {
        let val: Value = row
            .try_get_by_index::<i64>(i)
            .map(Value::from)
            .or_else(|_| row.try_get_by_index::<f64>(i).map(|v| serde_json::json!(v)))
            .or_else(|_| row.try_get_by_index::<String>(i).map(Value::String))
            .unwrap_or(Value::Null);
        map.insert(col_name.clone(), val);
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::OrderBy;
    use crate::digital_employee;

    /// 创建内存数据库并执行完整迁移
    async fn setup_memory_db() -> LocalSqliteDb {
        let conn = Database::connect("sqlite::memory:")
            .await
            .expect("连接内存数据库失败");
        conn.execute(Statement::from_string(
            SeaDbBackend::Sqlite,
            "PRAGMA journal_mode=WAL;",
        ))
        .await
        .ok();
        LocalSqliteDb { conn }
    }

    /// 在数据库中执行 DDL 和种子 SQL
    async fn run_full_migration(db: &LocalSqliteDb) {
        let ddl = include_str!("migrations/m20260529_001_create_digital_tables.sql");
        let seed = include_str!("migrations/m20260529_002_seed_digital_employees.sql");

        for stmt in digital_employee::split_statements(ddl) {
            db.execute_raw(&stmt).await.expect("DDL 执行失败");
        }
        for stmt in digital_employee::split_statements(seed) {
            db.execute_raw(&stmt).await.expect("Seed 执行失败");
        }
    }

    /// 构建测试用的员工数据 HashMap
    fn make_employee_data(code: &str, name: &str, sort_order: i32) -> DbRow {
        let now = "2026-05-29T00:00:00+08:00";
        [
            ("code", Value::String(code.into())),
            ("name", Value::String(name.into())),
            ("is_builtin", Value::Bool(false)),
            ("avatar", Value::String("🤖".into())),
            ("personality_tags", Value::String("[]".into())),
            ("personality_desc", Value::String("".into())),
            ("comm_style", Value::String("formal".into())),
            ("decision_pref", Value::String("data_driven".into())),
            ("focus_areas", Value::String("[]".into())),
            ("deliverable_groups", Value::String("[]".into())),
            ("default_op", Value::String("write".into())),
            ("default_capability", Value::String("".into())),
            ("sort_order", Value::from(sort_order)),
            ("created_at", Value::String(now.into())),
            ("updated_at", Value::String(now.into())),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect()
    }

    // ─── Tests ────────────────────────────────────────────────

    #[tokio::test]
    async fn seed_creates_18_builtin_employees() {
        let db = setup_memory_db().await;
        run_full_migration(&db).await;

        let result = db
            .find_all("digital_employees", QueryParams::default())
            .await
            .expect("查询失败");

        assert_eq!(result.total, 18);
        assert_eq!(result.rows.len(), 18);

        // 验证第一个员工数据完整性
        let first = &result.rows[0];
        assert_eq!(first.get("code").unwrap().as_str().unwrap(), "ba-analyst");
        assert_eq!(first.get("name").unwrap().as_str().unwrap(), "需求分析师");
        assert!(first.get("is_builtin").unwrap().as_i64().unwrap() == 1);
    }

    #[tokio::test]
    async fn seed_creates_skills_agents_mcps() {
        let db = setup_memory_db().await;
        run_full_migration(&db).await;

        let skills = db.find_all("skills", QueryParams::default()).await.unwrap();
        let agents = db.find_all("agents", QueryParams::default()).await.unwrap();
        let mcps = db.find_all("mcps", QueryParams::default()).await.unwrap();

        assert_eq!(skills.total, 11);
        assert_eq!(agents.total, 5);
        assert_eq!(mcps.total, 4);
    }

    #[tokio::test]
    async fn insert_employee_auto_increment_id() {
        let db = setup_memory_db().await;
        run_full_migration(&db).await;

        let id_str = db
            .insert(
                "digital_employees",
                make_employee_data("test-user", "测试用户", 99),
            )
            .await
            .expect("插入失败");

        // SQLite 自增 ID 从已有最大 id + 1 开始（种子 18 条后应为 19）
        let id: i64 = id_str.parse().unwrap();
        assert!(id >= 19, "自增 ID 应 >= 19, 实际: {id}");

        let row = db
            .find_by_id("digital_employees", &id_str)
            .await
            .expect("查询失败")
            .expect("记录不存在");

        assert_eq!(row.get("code").unwrap().as_str().unwrap(), "test-user");
        assert_eq!(row.get("name").unwrap().as_str().unwrap(), "测试用户");
    }

    #[tokio::test]
    async fn update_employee_preserves_id() {
        let db = setup_memory_db().await;
        run_full_migration(&db).await;

        // 更新第一个员工的名字
        let mut data = DbRow::new();
        data.insert("name".into(), Value::String("需求分析师(已更名)".into()));
        data.insert(
            "updated_at".into(),
            Value::String("2026-06-01T00:00:00+08:00".into()),
        );

        db.update("digital_employees", "1", data)
            .await
            .expect("更新失败");

        let row = db
            .find_by_id("digital_employees", "1")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            row.get("name").unwrap().as_str().unwrap(),
            "需求分析师(已更名)"
        );
        // code 不应被修改
        assert_eq!(row.get("code").unwrap().as_str().unwrap(), "ba-analyst");
    }

    #[tokio::test]
    async fn find_all_with_ordering() {
        let db = setup_memory_db().await;
        run_full_migration(&db).await;

        let result = db
            .find_all(
                "digital_employees",
                QueryParams {
                    order_by: vec![OrderBy {
                        field: "sort_order".into(),
                        descending: true,
                    }],
                    limit: Some(3),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(result.rows.len(), 3);
        // 按 sort_order DESC, 前 3 应为 18, 17, 16
        let first_code = result.rows[0].get("code").unwrap().as_str().unwrap();
        assert_eq!(first_code, "external-coordinator");
    }

    #[tokio::test]
    async fn delete_employee() {
        let db = setup_memory_db().await;
        run_full_migration(&db).await;

        db.delete("digital_employees", "19")
            .await
            .expect("删除失败");

        let result = db
            .find_all("digital_employees", QueryParams::default())
            .await
            .unwrap();
        assert_eq!(result.total, 18); // 种子 18 条，第 19 条不存在时删除是 no-op？不对...

        // 先插入再删除
        let id = db
            .insert("digital_employees", make_employee_data("tmp", "临时", 100))
            .await
            .unwrap();
        let before = db
            .find_all("digital_employees", QueryParams::default())
            .await
            .unwrap();
        assert_eq!(before.total, 19);

        db.delete("digital_employees", &id).await.unwrap();
        let after = db
            .find_all("digital_employees", QueryParams::default())
            .await
            .unwrap();
        assert_eq!(after.total, 18);
    }
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
        Value::Array(_) | Value::Object(_) => sea_orm::Value::String(Some(Box::new(v.to_string()))),
    }
}
