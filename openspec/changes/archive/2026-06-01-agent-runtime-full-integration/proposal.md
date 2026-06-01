## Why

ArchBot 定义为本地 Agent 控制平面，统一管理 Claude Code、OpenCode、Hermes、OpenClaw 四个外部 Agent Runtime。当前已完成约 35% 的基础设施（Runtime 配置、启动器、隔离 HOME、密钥管理、Turn Executor），但缺少核心控制平面组件：会话管理、Adapter 协议适配、SSE 事件流、数据库持久化、版本管理、前端面板和文件变更控制。这导致 Turn Executor 是可验证的独立单元，但无法支撑多 Turn 会话、无法实时流式推送到前端、无法审计和回滚。

## What Changes

- **Session Manager** — AgentSession + AgentTurn 生命周期管理，会话目录结构，状态机
- **Adapter Manager** — Adapter trait + 4 个 Runtime Adapter 实现，HTTP + SSE 通信协议
- **SSE Event Stream** — Axum SSE 端点，16 种标准事件广播，前端 EventSource 消费
- **Database Layer** — 10 张 agent 表的 SeaORM 实体 + 数据库迁移
- **Version Manager** — Runtime 安装/检测/升级/回滚（当前为 stub）
- **File Change Control** — 任务前快照 + 任务后 diff + 按 Turn/文件回滚
- **Frontend Panels** — 5 个核心 Agent 面板（Session、Turn、EventStream、DiffReview、AuditLog）
- **Context Assembly** — 滑动窗口 + 历史摘要 + 关键决策 + 上下文快照
- **Shell Allowlisting** — 命令白名单/黑名单，工作目录限定

## Capabilities

### New Capabilities
- `agent-session-manager`: 会话生命周期——创建、查询、暂停、归档 AgentSession，管理 AgentTurn 状态机
- `agent-adapter-manager`: Adapter 管理——Adapter trait 定义，ClaudeCode/OpenCode/Hermes/OpenClaw adapter 实现，HTTP+SSE 协议适配
- `agent-sse-stream`: SSE 事件流——16 种标准事件的 Axum SSE 端点，前端 EventSource 实时消费
- `agent-database`: Agent 数据库——10 张表的 SeaORM 实体定义、迁移脚本、CRUD 操作
- `agent-version-manager`: Runtime 版本管理——安装、检测、升级、回滚（替代当前 agent_config_handler 中的 stub）
- `agent-file-control`: 文件变更控制——任务前 Git/文件快照，任务后 diff 扫描，按 Turn/文件回滚
- `agent-frontend-panels`: 前端 Agent 面板——SessionPanel、TurnPanel、EventStreamPanel、DiffReviewPanel、AuditLogPanel
- `agent-context-assembly`: 上下文组装——滑动窗口选择、历史摘要、关键决策提取、上下文快照持久化
- `agent-shell-control`: Shell 权限控制——命令白名单/黑名单，工作目录限定，超时与审计

### Modified Capabilities
None — all existing capabilities remain unchanged.

## Impact

- `src-tauri/src/agent_runtime/` — 新增 session_manager, adapter_manager, sse_stream, version_manager, file_control, shell_control 子模块
- `src-tauri/src/agent_runtime/turn_executor.rs` — 接入 Session Manager 和 Adapter Manager
- `src-tauri/src/db/` — 新增 10 张 agent 表的实体和迁移
- `src-tauri/src/server.rs` — 新增 SSE 端点
- `src/components/domain/` — 新增 5 个 Agent 前端面板
- `src-tauri/config/runtimes.default.yml` — 无需修改（已完整）
- `src/i18n/zh-CN.ts`, `src/i18n/en-US.ts` — 新增 agent 面板相关文案
- `prd.yml` — 记录 agent 控制平面完整集成
- `function-map.yml` — 更新模块映射
