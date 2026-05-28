## ADDED Requirements

### Requirement: LanceDB 数据库连接

系统 SHALL 支持在指定路径上创建或打开 LanceDB 嵌入式数据库。

#### Scenario: 本地路径创建数据库

- **WHEN** 调用 `db_connect` 命令并传入本地目录路径
- **THEN** 系统在该路径下创建或打开 LanceDB 数据库
- **AND** 返回连接成功状态

### Requirement: 向量表管理

系统 SHALL 支持创建、列出和删除向量表，表需指定向量维度和索引类型。

#### Scenario: 创建向量表

- **WHEN** 调用 `db_create_table` 命令传入表名和向量维度
- **THEN** 系统创建带有 id（字符串）和 vector（指定维度 float32 数组）的表
- **AND** 返回创建成功

#### Scenario: 列出所有表

- **WHEN** 调用 `db_list_tables` 命令
- **THEN** 返回当前数据库中所有表的名称列表

### Requirement: 向量写入与搜索

系统 SHALL 支持向表中写入向量数据，并支持基于余弦相似度的 ANN 搜索。

#### Scenario: 写入向量

- **WHEN** 调用 `db_insert` 命令传入表名、id 和 vector
- **THEN** 向量数据持久化到表中
- **AND** 返回成功

#### Scenario: ANN 搜索

- **WHEN** 调用 `db_search` 命令传入表名、查询向量和 top_k
- **THEN** 返回相似度最高的 k 条记录的 id 和 distance 值
