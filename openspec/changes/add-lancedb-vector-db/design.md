## Context

ArchBot 为 Tauri 2 桌面应用，Rust 后端运行在用户本机。LanceDB 嵌入式模式无需外部服务，数据文件直接存储在本地目录。

## Goals / Non-Goals

**Goals:**
- 嵌入 LanceDB，零外部依赖
- 提供建表、写入、搜索、列出表 4 个核心操作
- 通过 Tauri command 暴露给前端

**Non-Goals:**
- 不实现全文搜索（BM25）
- 不实现增量索引更新
- 不暴露原始 `lance` 格式操作

## Decisions

### 使用全局 Arc<Mutex<Connection>>

数据库连接为全局单例，多线程安全访问。Tauri command 均为 async，但 LanceDB 的 Connection 不是 Send + Sync，需要 `Arc<Mutex<>>` 包装。

### 默认索引：L2 距离

LanceDB 默认使用 L2 欧氏距离做 ANN 搜索。后续可升级为余弦相似度通过 `metric_type` 参数配置。

### 表 Schema：id + vector + embedding 文本

```rust
let schema = arrow_schema::Schema::new(vec![
    arrow_schema::Field::new("id", DataType::Utf8, false),
    arrow_schema::Field::new("vector", DataType::FixedSizeList(
        Arc::new(Field::new("item", DataType::Float32, true)),
        dim,
    ), true),
    arrow_schema::Field::new("text", DataType::Utf8, true),
]);
```

### 存储路径

默认 `~/.ArchBot/lancedb/`，也可通过参数指定。与 license.dat 和 settings.json 同目录。
