//! 数字员工管理模块
//!
//! 提供数字员工的 CRUD Tauri commands，以及数据库迁移和种子数据初始化。
//! 所有命令通过 `dispatch_db!` 宏调用统一的 `DbBackend` trait。

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::db::{self, DbRow};

/// 数字员工 JSON 表示（前端使用，code 是唯一业务键）
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DigitalEmployee {
    #[serde(default)]
    pub id: i32,
    pub code: String,
    pub name: String,
    pub is_builtin: bool,
    pub avatar: String,
    pub personality_tags: String,
    pub personality_desc: String,
    pub comm_style: String,
    pub decision_pref: String,
    pub focus_areas: String,
    pub deliverable_groups: String,
    pub default_op: String,
    #[serde(default)]
    pub default_capability: String,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
    /// Transient: skill codes to sync to employee_skills table
    #[serde(default)]
    pub skills: Vec<String>,
}

fn row_to_employee(row: &DbRow) -> DigitalEmployee {
    DigitalEmployee {
        id: row.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        code: row
            .get("code")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        name: row
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        is_builtin: row.get("is_builtin").and_then(|v| v.as_i64()).unwrap_or(0) != 0,
        avatar: row
            .get("avatar")
            .and_then(|v| v.as_str())
            .unwrap_or("🤖")
            .to_string(),
        personality_tags: row
            .get("personality_tags")
            .and_then(|v| v.as_str())
            .unwrap_or("[]")
            .to_string(),
        personality_desc: row
            .get("personality_desc")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        comm_style: row
            .get("comm_style")
            .and_then(|v| v.as_str())
            .unwrap_or("formal")
            .to_string(),
        decision_pref: row
            .get("decision_pref")
            .and_then(|v| v.as_str())
            .unwrap_or("data_driven")
            .to_string(),
        focus_areas: row
            .get("focus_areas")
            .and_then(|v| v.as_str())
            .unwrap_or("[]")
            .to_string(),
        deliverable_groups: row
            .get("deliverable_groups")
            .and_then(|v| v.as_str())
            .unwrap_or("[]")
            .to_string(),
        default_op: row
            .get("default_op")
            .and_then(|v| v.as_str())
            .unwrap_or("write")
            .to_string(),
        default_capability: row
            .get("default_capability")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        sort_order: row.get("sort_order").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        created_at: row
            .get("created_at")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        updated_at: row
            .get("updated_at")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        skills: vec![],
    }
}

// ═══════════════════════════════════════════════════════════════════
// 数据库迁移
// ═══════════════════════════════════════════════════════════════════

const MIGRATION_DDL: &str = include_str!("db/migrations/m20260529_001_create_digital_tables.sql");
const MIGRATION_SEED: &str = include_str!("db/migrations/m20260529_002_seed_digital_employees.sql");

/// 执行数据库迁移（DDL + 种子数据）
///
/// 使用 `IF NOT EXISTS` 确保幂等。如果种子数据已存在则跳过。
/// `db_type` 为 "local" 或 "remote"。
async fn run_migrations(db_type: &str) -> Result<(), String> {
    // Execute DDL (idempotent via IF NOT EXISTS)
    for stmt in split_statements(MIGRATION_DDL) {
        db::db_execute_raw(stmt, db_type.to_string()).await?;
    }

    // ── Runtime migration: add columns that may not exist in older DBs ──
    // Try ALTER TABLE — ignore error if column already exists (SQLite < 3.35
    // doesn't support ADD COLUMN IF NOT EXISTS)
    let _ = db::db_execute_raw(
        "ALTER TABLE digital_employees ADD COLUMN default_capability VARCHAR(256) NOT NULL DEFAULT ''".to_string(),
        db_type.to_string(),
    )
    .await;

    // Check if seed already exists
    let existing = db::db_find_all(
        "digital_employees".to_string(),
        db::QueryParams::default(),
        db_type.to_string(),
    )
    .await?;

    if existing.total == 0 {
        for stmt in split_statements(MIGRATION_SEED) {
            db::db_execute_raw(stmt, db_type.to_string()).await?;
        }
    }

    Ok(())
}

/// 将 SQL 字符串按语句分割（处理 INSERT 多行值）
pub(crate) fn split_statements(sql: &str) -> Vec<String> {
    let mut stmts = Vec::new();
    let mut current = String::new();
    let mut in_parent = false;

    for line in sql.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("--") {
            continue;
        }
        current.push_str(trimmed);
        current.push(' ');

        // Track parenthesized blocks for multi-line INSERT VALUES
        for ch in trimmed.chars() {
            match ch {
                '(' => in_parent = true,
                ')' => in_parent = false,
                _ => {}
            }
        }

        if trimmed.ends_with(';') && !in_parent {
            let stmt = current.trim().trim_end_matches(';').to_string();
            if !stmt.is_empty() {
                stmts.push(stmt);
            }
            current.clear();
        }
    }

    stmts
}

// ═══════════════════════════════════════════════════════════════════
// Tauri Commands
// ═══════════════════════════════════════════════════════════════════

/// 初始化数据库并执行迁移
///
/// 本地 SQLite 路径: `{project_dir}/.archbot/db/archbot.db`
/// 若传入 `project_path` 为空字符串，则回退到旧路径 `~/.ArchBot/archbot.db`。
/// 自动创建 `.archbot/db/` 父目录。然后执行 DDL + 种子数据。
#[tauri::command]
pub async fn de_init(db_type: String, project_path: String) -> Result<(), String> {
    if db_type == "local" {
        let db_path = if project_path.is_empty() {
            // Fallback to global path for backward compatibility
            dirs::home_dir()
                .ok_or("无法获取用户主目录")?
                .join(".ArchBot")
                .join("archbot.db")
        } else {
            let ab_path = std::path::Path::new(&project_path);
            let project_dir = ab_path.parent().unwrap_or(ab_path);
            project_dir.join(".archbot").join("db").join("archbot.db")
        };

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建数据库目录失败: {e}"))?;
        }

        let db_path_str = db_path.to_string_lossy().to_string();
        let _ = crate::db::db_connect(db_path_str).await;
    }
    run_migrations(&db_type).await
}

/// 列出所有数字员工
#[tauri::command]
pub async fn de_list(db_type: String) -> Result<Vec<DigitalEmployee>, String> {
    let result = db::db_find_all(
        "digital_employees".to_string(),
        db::QueryParams {
            order_by: vec![db::OrderBy {
                field: "sort_order".to_string(),
                descending: false,
            }],
            ..Default::default()
        },
        db_type.clone(),
    )
    .await?;

    // Load all employee_skills in one query for merging
    let skills_result = db::db_find_all(
        "employee_skills".to_string(),
        db::QueryParams::default(),
        db_type,
    )
    .await
    .unwrap_or(db::QueryResult {
        rows: vec![],
        total: 0,
    });

    // Group skills by employee_code
    let mut skills_map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for row in &skills_result.rows {
        let emp_code = row.get("employee_code").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let skill_code = row.get("skill_code").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if !emp_code.is_empty() && !skill_code.is_empty() {
            skills_map.entry(emp_code).or_default().push(skill_code);
        }
    }

    Ok(result
        .rows
        .iter()
        .map(|row| {
            let mut emp = row_to_employee(row);
            let code = &emp.code;
            emp.skills = skills_map.get(code).cloned().unwrap_or_default();
            emp
        })
        .collect())
}

/// 按 code 获取单个数字员工
#[tauri::command]
pub async fn de_get(code: String, db_type: String) -> Result<Option<DigitalEmployee>, String> {
    let result = db::db_find_all(
        "digital_employees".to_string(),
        db::QueryParams {
            filters: vec![db::Filter {
                field: "code".to_string(),
                operator: "eq".to_string(),
                value: Value::String(code),
            }],
            limit: Some(1),
            ..Default::default()
        },
        db_type,
    )
    .await?;

    Ok(result.rows.first().map(row_to_employee))
}

/// 创建或更新数字员工
#[tauri::command]
pub async fn de_save(employee: DigitalEmployee, db_type: String) -> Result<(), String> {
    let now = crate::now_iso();
    let emp_code = employee.code.clone(); // clone before move into data
    let mut data = std::collections::HashMap::new();
    data.insert("code".to_string(), Value::String(employee.code));
    data.insert("name".to_string(), Value::String(employee.name));
    data.insert("is_builtin".to_string(), Value::from(employee.is_builtin));
    data.insert("avatar".to_string(), Value::String(employee.avatar));
    data.insert(
        "personality_tags".to_string(),
        Value::String(employee.personality_tags),
    );
    data.insert(
        "personality_desc".to_string(),
        Value::String(employee.personality_desc),
    );
    data.insert("comm_style".to_string(), Value::String(employee.comm_style));
    data.insert(
        "decision_pref".to_string(),
        Value::String(employee.decision_pref),
    );
    data.insert(
        "focus_areas".to_string(),
        Value::String(employee.focus_areas),
    );
    data.insert(
        "deliverable_groups".to_string(),
        Value::String(employee.deliverable_groups),
    );
    data.insert("default_op".to_string(), Value::String(employee.default_op));
    data.insert("default_capability".to_string(), Value::String(employee.default_capability.clone()));
    data.insert("sort_order".to_string(), Value::from(employee.sort_order));
    data.insert("updated_at".to_string(), Value::String(now.clone()));

    if employee.id > 0 {
        // Update
        data.insert("created_at".to_string(), Value::String(employee.created_at));
        db::db_update(
            "digital_employees".to_string(),
            employee.id.to_string(),
            data,
            db_type.clone(),
        )
        .await?;
    } else {
        // Create
        data.insert("created_at".to_string(), Value::String(now.clone()));
        db::db_insert("digital_employees".to_string(), data, db_type.clone()).await?;
    }

    // ── Sync employee_skills ──
    // Delete existing skills for this employee, then re-insert
    let _ = db::db_execute_raw(
        format!(
            "DELETE FROM employee_skills WHERE employee_code = '{}'",
            emp_code.replace('\'', "''")
        ),
        db_type.clone(),
    )
    .await;

    for skill_code in &employee.skills {
        let mut skill_data = std::collections::HashMap::new();
        skill_data.insert(
            "employee_code".to_string(),
            Value::String(emp_code.clone()),
        );
        skill_data.insert("skill_code".to_string(), Value::String(skill_code.clone()));
        skill_data.insert("created_at".to_string(), Value::String(now.clone()));
        let _ = db::db_insert("employee_skills".to_string(), skill_data, db_type.clone()).await;
    }

    Ok(())
}

/// 按 id 删除数字员工
#[tauri::command]
pub async fn de_delete(id: i32, db_type: String) -> Result<(), String> {
    db::db_delete("digital_employees".to_string(), id.to_string(), db_type).await
}
