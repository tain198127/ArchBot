## Context

ArchBot 定位为本地 Agent 控制平面，管理 Claude Code、OpenCode、Hermes、OpenClaw 四个外部 Agent Runtime。当前已完成基础设施层：Runtime 配置解析（runtime_config.rs）、进程隔离启动（launcher.rs）、隔离 HOME 目录（home_setup.rs）、密钥管理（secret/）、审计框架（audit.rs）和 Turn Executor（turn_executor.rs）。Turn Executor 可独立运行一次完整的 Agent 调用，但只能通过 stdout 捕获结果，缺少会话管理、实时事件流、Adapter 协议适配和数据库持久化。

本设计覆盖从"单次可验证调用"到"完整控制平面"的架构演进。

## Goals / Non-Goals

**Goals:**
- Session Manager: AgentSession + AgentTurn CRUD，状态机，会话目录结构
- Adapter Manager: 统一 Adapter trait + 4 个 Runtime adapter，HTTP + SSE 协议
- SSE Event Stream: Axum SSE 端点，16 种标准事件广播，前端 EventSource
- Database Layer: 10 张 agent 表 SeaORM 实体 + SQLite 迁移
- Version Manager: Runtime 安装/检测/升级/回滚（替代当前 stub）
- File Change Control: 任务前快照 + 任务后 diff + 回滚
- 5 个前端面板: SessionPanel, TurnPanel, EventStreamPanel, DiffReviewPanel, AuditLogPanel
- Context Assembly: 滑动窗口 + 摘要 + 决策 + 快照
- Shell Allowlisting: 命令白名单/黑名单，工作目录限定

**Non-Goals:**
- Hermes/OpenCode/OpenClaw 的实际 CLI 适配（共用接口，第一版只验证 Claude Code）
- 多 Runtime 并发 Turn（第一版单 Turn 串行）
- 远程 Adapter 部署（第一版 Adapter 与 ArchBot 同进程）
- LanceDB 向量化上下文检索（后续版本）
- BPMN 流程编排（独立模块，不在此 change 范围）

## Decisions

### 1. Adapter 与 ArchBot 同进程运行（第一版）

架构文档定义了 Adapter 作为独立进程通过 HTTP+SSE 通信。第一版将 Adapter 实现为 Rust trait 同进程调用，HTTP+SSE 通信层作为 facade 预留。

**理由**: 当前 ArchBot 是单机桌面应用，Adapter 独立进程增加了部署复杂度和调试难度。同进程 trait 实现让 Claude Code CLI 调用路径更短、错误处理更直接。HTTP+SSE facade 接口保持不变，后续可零改动切换到独立进程。

### 2. 数据库：SeaORM + SQLite

与现有项目一致。10 张 agent 表与现有 9 张 digital employee 表共存于同一 SQLite 文件。

### 3. SSE 流式方案：Axum SSE + 广播 channel

使用 `tokio::sync::broadcast` 实现事件广播。Turn Executor 发送事件到 channel，Axum SSE handler 订阅 channel 并转发给前端 EventSource。`GET /api/agent/sessions/{session_id}/turns/{turn_id}/events` 端点。

### 4. 版本管理：下载 + 校验 + 符号链接切换

Runtime 安装流程：下载 tarball → SHA256 校验 → 解压到 `~/.archbot/runtimes/{name}/versions/{ver}/` → 创建 `current` 符号链接。回滚即切换符号链接到旧版本。

### 5. 文件变更控制：Git 优先，文件 hash 回退

Git 仓库项目使用 `git diff --name-status` 做任务后 diff，`git checkout` 做回滚。非 Git 项目使用任务前文件 hash 快照 + 手动恢复。

### 6. 前端面板架构

5 个面板独立组件，挂载在现有 BottomPanel 的 tab 系统下。每个面板通过 `useAgentStore` (Pinia) 获取状态，通过 `agentExecuteTurn` Tauri command 触发操作。

## Risks / Trade-offs

- [Risk] Adapter 同进程运行 → 某个 Runtime 崩溃可能影响 ArchBot 稳定性 → Mitigation: Runtime 仍在子进程运行，Adapter trait 只负责协议转换和参数构造
- [Risk] SQLite 并发写入冲突 → Mitigation: SeaORM 连接池 + WAL 模式
- [Risk] SSE 连接中断导致事件丢失 → Mitigation: 事件持久化到 agent_event 表，前端重连后从 last_event_id 恢复
- [Risk] 10 张新表 + 9 张现有表 → 迁移脚本复杂度 → Mitigation: 分步迁移，每张表独立 migration 文件

## Open Questions

- Hermes/OpenClaw 的实际安装方式（PyPI/npm/cargo）待各 Runtime 发布稳定版后确定
- 上下文滑窗的窗口大小 N 的默认值——建议从 10 开始，通过项目级 agent.yml 可配
