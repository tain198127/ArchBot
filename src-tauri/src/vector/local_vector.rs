//! 本地 LanceDB 向量数据库后端
//!
//! 使用嵌入式 LanceDB 实现 [`VectorBackend`] trait。
//! 数据持久化到本地文件系统。

use async_trait::async_trait;
use lancedb::{connect, connection::Connection, query::ExecutableQuery};
use std::sync::Arc;

use super::{SearchResult, TableInfo, VectorBackend};

/// 本地 LanceDB 后端
///
/// 封装 LanceDB 嵌入式连接，提供本地向量存储和 ANN 搜索。
pub struct LocalVectorDb {
    conn: Connection,
}

impl LocalVectorDb {
    /// 连接到 LanceDB 数据库
    ///
    /// `path` 为存储目录路径。
    pub async fn connect(path: &str) -> Result<Self, String> {
        std::fs::create_dir_all(path).map_err(|e| format!("创建目录失败: {e}"))?;
        let conn = connect(path)
            .execute()
            .await
            .map_err(|e| format!("连接 LanceDB 失败: {e}"))?;
        Ok(Self { conn })
    }
}

#[async_trait]
impl VectorBackend for LocalVectorDb {
    async fn create_table(&self, name: &str, dimension: u32) -> Result<(), String> {
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
                    Vec::<Option<Vec<Option<f32>>>>::new(),
                    dimension as i32,
                )),
            ],
        )
        .map_err(|e| format!("创建空 RecordBatch 失败: {e}"))?;

        self.conn
            .create_table(
                name,
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

    async fn insert(&self, table: &str, id: &str, vector: Vec<f32>) -> Result<(), String> {
        let tbl = self
            .conn
            .open_table(table)
            .execute()
            .await
            .map_err(|e| format!("打开表失败: {e}"))?;

        use arrow_array::types::Float32Type;
        use arrow_array::{FixedSizeListArray, RecordBatch, StringArray};

        let dim = vector.len() as i32;
        let ids = StringArray::from(vec![id]);
        let cell: Vec<Option<Vec<Option<f32>>>> = vec![Some(vector.into_iter().map(Some).collect())];
        let vectors = FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(cell, dim);

        let schema = tbl.schema().await.map_err(|e| format!("{e}"))?;
        let batch = RecordBatch::try_new(schema.clone(), vec![Arc::new(ids), Arc::new(vectors)])
            .map_err(|e| format!("构建 RecordBatch 失败: {e}"))?;

        tbl.add(Box::new(arrow_array::RecordBatchIterator::new(
            vec![Ok(batch)].into_iter(),
            schema,
        )))
        .execute()
        .await
        .map_err(|e| format!("插入失败: {e}"))?;
        Ok(())
    }

    async fn search(
        &self,
        table: &str,
        query: Vec<f32>,
        top_k: usize,
    ) -> Result<Vec<SearchResult>, String> {
        use futures::StreamExt;
        use lancedb::query::QueryBase;

        let tbl = self
            .conn
            .open_table(table)
            .execute()
            .await
            .map_err(|e| format!("打开表失败: {e}"))?;

        let mut stream = tbl
            .query()
            .nearest_to(query)
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
                    metadata: None,
                });
            }
        }
        Ok(out)
    }

    async fn delete(&self, table: &str, id: &str) -> Result<(), String> {
        let tbl = self
            .conn
            .open_table(table)
            .execute()
            .await
            .map_err(|e| format!("打开表失败: {e}"))?;

        tbl.delete(&format!("id = '{}'", id))
            .await
            .map_err(|e| format!("删除失败: {e}"))?;
        Ok(())
    }

    async fn list_tables(&self) -> Result<Vec<String>, String> {
        self.conn
            .table_names()
            .execute()
            .await
            .map_err(|e| format!("{e}"))
    }

    async fn table_info(&self, name: &str) -> Result<TableInfo, String> {
        let tbl = self
            .conn
            .open_table(name)
            .execute()
            .await
            .map_err(|e| format!("打开表失败: {e}"))?;

        let schema = tbl.schema().await.map_err(|e| format!("{e}"))?;
        let dim = schema
            .field_with_name("vector")
            .map(|f| match f.data_type() {
                arrow_schema::DataType::FixedSizeList(_, n) => *n as u32,
                _ => 0,
            })
            .unwrap_or(0);

        Ok(TableInfo {
            name: name.to_string(),
            dimension: dim,
        })
    }
}
