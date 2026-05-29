## ADDED Requirements

### Requirement: 统一文件操作接口

系统 SHALL 提供 6 个 Tauri commands，所有命令通过 `fs_type: "local" | "remote"` 参数选择后端实现。

#### Scenario: 本地读取文件

- **WHEN** 调用 `fs_read` 命令且 `fs_type` 为 `"local"`
- **THEN** 系统通过 `std::fs` 读取本地文件
- **AND** 返回文件内容（UTF-8 字符串）

#### Scenario: 远程读取文件

- **WHEN** 调用 `fs_read` 命令且 `fs_type` 为 `"remote"`
- **THEN** 系统通过 `reqwest` 向远程 REST API 发起 GET 请求
- **AND** 返回响应体文本

### Requirement: Local 后端能力完整性

Local backend SHALL 支持：读文件、写文件、列出目录、删除、检查存在、创建目录。

#### Scenario: 列出本地目录

- **WHEN** 调用 `fs_list` 命令且 `fs_type` 为 `"local"`
- **THEN** 返回指定路径下的 `FileEntry` 列表（含 name、path、is_dir、size、modified）

### Requirement: Remote 后端能力完整性

Remote backend SHALL 支持与 Local 相同的 6 个操作，通过已配置的 base_url + REST path 拼接请求 URL。

#### Scenario: 远程写文件

- **WHEN** 调用 `fs_write` 命令且 `fs_type` 为 `"remote"`
- **THEN** 系统向 `{base_url}/fs/write` 发送 POST 请求
- **AND** 请求体包含 path 和 content
