use std::sync::{Arc, Mutex, OnceLock};

use futures::StreamExt;
use lancedb::connection::Connection;
use lancedb::query::{ExecutableQuery, QueryBase};
use serde::Serialize;

static DB: OnceLock<Mutex<Option<Arc<Connection>>>> = OnceLock::new();

fn db_cell() -> &'static Mutex<Option<Arc<Connection>>> {
    DB.get_or_init(|| Mutex::new(None))
}

fn db_path() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".ArchBot")
        .join("lancedb")
}

async fn get_conn() -> Result<Arc<Connection>, String> {
    // Fast path: already initialized
    {
        let guard = db_cell().lock().map_err(|e| format!("Lock error: {e}"))?;
        if let Some(conn) = guard.as_ref() {
            return Ok(Arc::clone(conn));
        }
    }

    // Slow path: async init outside the lock
    let path = db_path();
    std::fs::create_dir_all(&path).map_err(|e| format!("创建目录失败: {e}"))?;
    let conn = lancedb::connect(path.to_str().unwrap_or("/tmp/archbot_lancedb"))
        .execute()
        .await
        .map_err(|e| format!("连接 LanceDB 失败: {e}"))?;

    let mut guard = db_cell().lock().map_err(|e| format!("Lock error: {e}"))?;
    if guard.is_none() {
        *guard = Some(Arc::new(conn));
    }
    Ok(Arc::clone(guard.as_ref().ok_or("数据库未连接")?))
}

#[derive(Serialize)]
pub struct TableInfo {
    pub name: String,
}

#[derive(Serialize)]
pub struct SearchResult {
    pub id: String,
    pub distance: f32,
}

// ─── Validation ───────────────────────────────────────────────

fn validate_table_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 64 {
        return Err("表名长度必须为 1-64 个字符".into());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err("表名只能包含字母、数字、下划线和连字符".into());
    }
    Ok(())
}

// ─── Tauri Commands ───────────────────────────────────────────

#[tauri::command]
pub async fn lancedb_list_tables() -> Result<Vec<TableInfo>, String> {
    let db = get_conn().await?;
    let names = db
        .table_names()
        .execute()
        .await
        .map_err(|e| format!("{e}"))?;
    Ok(names.into_iter().map(|n| TableInfo { name: n }).collect())
}

#[tauri::command]
pub async fn lancedb_create_table(name: String, dimension: u32) -> Result<(), String> {
    validate_table_name(&name)?;
    let db = get_conn().await?;

    use arrow_array::{FixedSizeListArray, RecordBatch, StringArray};
    use arrow_schema::{DataType, Field, Schema};

    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                dimension as i32,
            ),
            true,
        ),
    ]));

    let empty_batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(StringArray::from(Vec::<&str>::new())),
            Arc::new(FixedSizeListArray::from_iter_primitive::<
                arrow_array::types::Float32Type,
                _,
                _,
            >(
                Vec::<Option<Vec<Option<f32>>>>::new(), dimension as i32
            )),
        ],
    )
    .map_err(|e| format!("创建空 RecordBatch 失败: {e}"))?;

    db.create_table(
        &name,
        Box::new(arrow_array::RecordBatchIterator::new(
            vec![Ok(empty_batch)].into_iter(),
            schema,
        )),
    )
    .execute()
    .await
    .map_err(|e| format!("创建表失败: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn lancedb_insert(table_name: String, id: String, vector: Vec<f32>) -> Result<(), String> {
    validate_table_name(&table_name)?;
    let db = get_conn().await?;

    let table = db
        .open_table(&table_name)
        .execute()
        .await
        .map_err(|e| format!("打开表失败: {e}"))?;

    use arrow_array::types::Float32Type;
    use arrow_array::{FixedSizeListArray, RecordBatch, StringArray};

    let dim = vector.len() as i32;
    let ids = StringArray::from(vec![id.as_str()]);
    let cell: Vec<Option<Vec<Option<f32>>>> = vec![Some(vector.into_iter().map(Some).collect())];
    let vectors = FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(cell, dim);

    let schema = table.schema().await.map_err(|e| format!("{e}"))?;
    let batch = RecordBatch::try_new(schema.clone(), vec![Arc::new(ids), Arc::new(vectors)])
        .map_err(|e| format!("构建 RecordBatch 失败: {e}"))?;

    table
        .add(Box::new(arrow_array::RecordBatchIterator::new(
            vec![Ok(batch)].into_iter(),
            schema,
        )))
        .execute()
        .await
        .map_err(|e| format!("插入失败: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn lancedb_search(
    table_name: String,
    query_vector: Vec<f32>,
    top_k: usize,
) -> Result<Vec<SearchResult>, String> {
    validate_table_name(&table_name)?;
    let db = get_conn().await?;

    let table = db
        .open_table(&table_name)
        .execute()
        .await
        .map_err(|e| format!("打开表失败: {e}"))?;

    let mut stream = table
        .query()
        .nearest_to(query_vector)
        .map_err(|e| format!("构建搜索失败: {e}"))?
        .limit(top_k)
        .execute()
        .await
        .map_err(|e| format!("搜索失败: {e}"))?;

    use arrow_array::Float32Array;

    let mut out = Vec::new();
    while let Some(batch_result) = stream.next().await {
        let batch = batch_result.map_err(|e| format!("读取批次失败: {e}"))?;
        for i in 0..batch.num_rows() {
            let id_val = batch
                .column_by_name("id")
                .and_then(|c| {
                    c.as_any()
                        .downcast_ref::<arrow_array::StringArray>()
                        .map(|a| a.value(i).to_string())
                })
                .ok_or("读取 id 列失败")?;
            let dist_val = batch
                .column_by_name("_distance")
                .and_then(|c| {
                    c.as_any()
                        .downcast_ref::<Float32Array>()
                        .map(|a| a.value(i))
                })
                .ok_or("读取 distance 列失败")?;
            out.push(SearchResult {
                id: id_val,
                distance: dist_val,
            });
        }
    }

    Ok(out)
}
