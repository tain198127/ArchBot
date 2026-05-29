## Context

当前 ArchBot 已有 `read_local_file` 和 `fetch_remote` 两个独立的 Tauri command，分别处理本地和远程。新设计将它们统一到一个 trait 下，前端只需切换 `fs_type` 参数。

## Goals / Non-Goals

**Goals:**
- 统一的文件操作 trait `FsBackend`，6 个方法
- `LocalFs` 封装 `std::fs`
- `RemoteFs` 封装 `reqwest`，通过 REST API 操作远程文件
- 前端 `FsClient` 抽象类 + 两个具象类

**Non-Goals:**
- 不实现文件流式传输（chunked upload/download）
- 不实现文件锁定或并发控制
- Remote 不实现 WebSocket 实时同步

## Decisions

### trait 使用 `async_trait`

Rust 原生 trait 不支持 async fn，使用 `async_trait` crate 提供兼容性。

### RemoteFs REST API 约定

| 操作 | HTTP Method | URL | Body |
|------|-------------|-----|------|
| read | GET | `{base}/fs/read?path=...` | — |
| write | POST | `{base}/fs/write` | `{path, content}` |
| list | GET | `{base}/fs/list?path=...` | — |
| delete | DELETE | `{base}/fs/delete?path=...` | — |
| exists | GET | `{base}/fs/exists?path=...` | — |
| mkdir | POST | `{base}/fs/mkdir` | `{path}` |

### 全局 Backend 管理

与 LanceDB 相同的 `OnceLock<Mutex<>>` 模式：
- `LOCAL_FS` — 初始化一次，全局复用
- `REMOTE_FS` — 需要先配置 `base_url`，初始化一次

### Remote 需要前端配置 base_url

增加 `fs_configure_remote(base_url, token)` Tauri command，在调用 remote 操作前配置连接参数。
