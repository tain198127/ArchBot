## Why

ArchBot 作为 AI 驱动的开发管理工具，需要本地向量数据库来存储代码嵌入、文档语义检索、知识库管理等。LanceDB 是 Rust 原生的嵌入式向量数据库——零外部依赖、毫秒级查询、支持本地文件系统存储，完美契合 ArchBot 的离线优先架构。

## What Changes

- 新增 `src-tauri/src/lancedb_store.rs` 模块：封装 LanceDB 连接、表管理、向量 CRUD
- 新增 Tauri commands：`db_create_table`, `db_insert`, `db_search`, `db_list_tables`
- 数据存储在项目目录下的 `.archbot-data/` 中

## Capabilities

### New Capabilities
- `lancedb-integration`: 嵌入式向量数据库——连接、建表、写入向量、ANN 搜索、表管理

## Impact

- `src-tauri/Cargo.toml`：新增 `lancedb` 依赖
- `src-tauri/src/lancedb_store.rs`：新模块
- `src-tauri/src/lib.rs`：注册 module 和 commands
