# ArchBot 全架构审计报告

审计日期: 2026-06-07
审计标准: ADR-001 (Rust/TS 边界) + Tauri 2 最佳实践 + Rust 领域模型惯例

---

# PART 1: Tauri 2 模式合规审计

## 执行概要

| 严重程度 | 数量 | 说明 |
|---------|------|------|
| BLOCKER | 2 | CSP 完全禁用; `db_execute_raw` 暴露为 Tauri command |
| HIGH | 3 | 全局 OnceLock 替代 Tauri State; 硬件指纹通过 shell 获取; HTTP API 绕过 IPC 权限模型 |
| MEDIUM | 5 | 缺少 capabilities 配置; withGlobalTauri 风险; 无 CSP; 缺少 fs scope; 错误消息暴露路径 |
| LOW | 4 | 模块命名不一致; 重复代码; crate-type 冗余; HTTP 默认启用 |

---

## BLOCKER 级别

### 1. `tauri.conf.json` 中 CSP 完全禁用
```json
"security": { "csp": null }
```
`"csp": null` 意味着任何注入脚本可执行任意系统命令（通过 `window.__TAURI__`）。
**修复**: 配置最小权限 CSP。

### 2. `db_execute_raw` 暴露为 Tauri command
`src-tauri/src/lib.rs:138` — 前端可传任意 SQL，与 HTTP API 安全等级相同。
**修复**: 移除该 command，迁移改为 `setup()` 自动执行。

---

## HIGH 级别

### 3. 全局 `OnceLock<Mutex<T>>` 替代 Tauri State
6 处全局变量（`fs/mod.rs:107`, `db/mod.rs:93`, `vector/mod.rs:77`, `license.rs:5`）替代 `tauri::State<T>` + `app.manage()`。
**后果**: 单元测试无法替换实现，无 DI。
**修复**: 迁移到 Tauri State 管理。

### 4. 硬件指纹通过 shell 命令获取
`license.rs:73-121` — `ifconfig en0` / `ip link show` / `getmac`
**后果**: 不可靠 + PATH 劫持风险。
**修复**: 使用 `mac_address` crate。

### 5. HTTP API (Axum) 绕过 Tauri IPC 权限模型
`localhost:1421` 注册了几乎所有 handler 路由，无认证。
**修复**: 添加 token 认证；审查暴露范围。

---

## MEDIUM 级别

6. 缺少 `src-tauri/capabilities/` 目录 — Tauri 2 的 permissions 模型未启用
7. `withGlobalTauri: true` — 任何第三方脚本可通过 `window.__TAURI__` 调用后端
8. `read_local_file` 可读任意文件 — 无路径白名单
9. `load_http_config` 静默吞没错误 — 格式错误仍启用 HTTP server
10. 部分 struct 缺少 `rename_all = "camelCase"` — 可能与前端不匹配

---

## 合规项 (做得好的)

- `#[tauri::command]` 正确注册 + `generate_handler!` 集中管理 ✅
- Plugin 初始化正确（opener, dialog, fs）✅
- `setup()` hook 用于 trace 初始化 ✅
- `canonicalize()` + `starts_with` 防路径穿越 ✅
- SSRF 防护（RemoteFs 校验 host）✅
- SQL 标识符校验（`validate_identifier`）✅
- Secret 加密（AES-256-GCM, 机器 ID 密钥, 600 权限）✅
- 乐观锁（version-based）✅

---

# PART 2: Rust 领域模型纯度审计

## 执行概要

| 严重程度 | 数量 | 说明 |
|---------|------|------|
| BLOCKER | 1 | 21 个 SeaORM Entity 定义后完全未使用，全部 CRUD 用 raw SQL |
| HIGH | 3 | Result<String> 全局使用无类型化错误; SeaORM vs Raw SQL 并存; 贫血模型 |
| MEDIUM | 9 | String 代替 enum; 测试严重不足; 死文件; unwrap() 隐患; bool 解析重复; 全局单例; 潜在死锁; DB 逻辑混入模型 |
| LOW | 4 | 缺少 newtype; 类型不同步; 中英文混用 |

---

## BLOCKER 级别

### 1. 21 个 SeaORM Entity 完全未使用
`src-tauri/src/db/entities/` 定义了完整的 `DeriveEntityModel`、`DeriveRelation`、`ActiveModelBehavior`，但 `local_sqlite.rs` 全程用 raw SQL。类型安全、编译期检查、迁移工具全部浪费。

**修复**: 二选一 — 删除 entities 目录，或重构 `local_sqlite.rs` 使用 SeaORM Entity。

---

## HIGH 级别

### 2. 错误处理: `Result<T, String>` 全局使用
违反项目 Rust 规则（要求 `thiserror` 或 `anyhow`）。无法匹配具体错误类型，100+ 处重复 `format!("查询失败: {e}")` 模式。
**修复**: 定义 `DbError` enum (thiserror)，内部使用类型化错误。

### 3. SeaORM Entity 与 Raw SQL 并存
`local_sqlite.rs` 全程 raw SQL，但 21 个 Entity 文件存在且完整。`validate_identifier()` 只能缓解不能消除 SQL 注入风险。
**修复**: 与 #1 联动。

### 4. 贫血模型 — 数据和行为完全分离
所有领域类型零方法。DB 转换逻辑（`flow_row_from_db`、`bool_field`）混在 model.rs 中。
**修复**: DB helper 移至独立 repository 层。

---

## MEDIUM 级别

5. **String 代替 enum** — `flow_type`、`status`、`severity`、`operator` 均用 String，丧失编译期穷尽检查
6. **死文件** — `local_sqllite.rs` (空文件, 拼写错误), `lancedb_store.rs` (已注释仍保留)
7. **全局单例** — 同样模式在 db/fs/vector/mod.rs 三处重复
8. **unwrap() 隐患** — `conductor.rs:147` 是生产代码，panic 会杀进程
9. **bool 解析在三处重复** — 不同实现，不同边界情况
10. **潜在死锁** — 废弃的 `lancedb_store.rs` 使用 std Mutex 保护 async 操作
11. **持久化逻辑混入模型文件** — `model.rs` 含 DB 转换函数
12. **测试覆盖严重不足** — 远低于 80% 要求

---

## 合规项 (做得好的)

- 模块按领域划分，mod.rs 干净 ✅
- 无循环依赖 ✅
- 所有顶层模块有 `//!` 文档 ✅
- `serde(rename_all = "camelCase")` 统一 ✅
- 零 unsafe 代码 ✅
- immutability by default ✅

---

# PART 3: TS/Rust 边界审计（按 ADR-001）

## 执行概要

| 严重程度 | 数量 | 说明 |
|---------|------|------|
| BLOCKER | 1 | `conductor.rs` 682 行全部是业务逻辑——DAG 引擎、拓扑排序、网关策略、降级链 |
| HIGH | 1 | `validation.rs` 299 行图校验——循环检测、BFS 连通性、Start/End 检查 |
| MEDIUM | 4 | handler 业务规则混入、ai_config 硬编码、session state machine、context 滑动窗口策略 |
| LOW | 5 | data_standard 默认值、turn_executor prompt 模板、shell_control 黑名单等 |

---

## BLOCKER-01: `business_flow/conductor.rs` — 整个 DAG 执行引擎在 Rust 中

**文件:** `src-tauri/src/business_flow/conductor.rs` (682 行)
**违规:** ADR-001 明确规定 DAG 解析、拓扑排序、网关策略、降级链全应在 TypeScript。

| 行号 | 功能 | 应在 |
|------|------|------|
| 37–76 | `FlowGraph`, `FlowNode`, `FlowEdge` 结构体 — 与 `businessFlow.ts` 重复定义 | TypeScript |
| 104–261 | `run_flow()` — DAG 编排核心：解析图、构建邻接表、遍历节点、分派执行 | TypeScript |
| 115 | `build_dag()` — 从边列表构建 DAG | TypeScript |
| 131 | `topological_sort()` — **明确 ADR-001 违规** | TypeScript |
| 149–210 | AND 网关（`tokio::spawn` + `Semaphore` 并行分叉）| TypeScript |
| 211–238 | XOR 网关（条件路径选择）| TypeScript |
| 244 | `apply_degradation()` — **明确 ADR-001 违规** | TypeScript |
| 265–291 | `execute_single_node()` — 节点类型路由 | TypeScript |
| 392–418 | `build_dag()` — 图算法 | TypeScript |
| 420–459 | `topological_sort()` — 图算法 | TypeScript |
| 463–475 | `evaluate_condition()` — 条件求值 | TypeScript |
| 479–502 | `apply_degradation()` — 降级链策略 | TypeScript |

**修复计划:**
1. 将 `build_dag()`、`topological_sort()`、`evaluate_condition()`、`apply_degradation()` 移至 `src/orchestration/DagEngine.ts`
2. 将网关分派（AND/XOR/OR）移至 `DagEngine.ts`
3. Rust 只保留：`start_flow_run()`（IPC 入口）、每个 Agent 节点的进程启动、Tauri 事件发射
4. `bf_run_flow` 和 `bf_abort_run` 变为薄包装，接收 TypeScript 传来的执行计划

---

## HIGH-01: `business_flow/validation.rs` — 图结构校验在 Rust 中

**文件:** `src-tauri/src/business_flow/validation.rs` (299 行)
**违规:** ADR-001 说 "图合法性校验 — Zod"。

| 行号 | 功能 | 应在 |
|------|------|------|
| 41–134 | `validate_flow()` — Start/End 计数、孤立边、不连通节点、循环 | TypeScript |
| 137–165 | `bfs_reachable()` — BFS 连通性 | TypeScript |
| 167–208 | `detect_cycle()` + `dfs_cycle()` — DFS 循环检测 | TypeScript |

**修复计划:** 在 TypeScript 实现 `FlowGraphValidator.ts`（Zod + 自定义 validator）。现有的 Rust 测试（5 个）移植到 Vitest。

---

## MEDIUM 级别

**M-01 `handler.rs`**: 已发布/内置流的不可变性检查（行 139–176）、"(Copy)" 命名（行 239）→ 移至 TypeScript
**M-02 `ai_config.rs`**: 110 行硬编码的 9 个 AI Provider 定义（行 35–145）→ 移至 `src/config/aiProviders.ts`
**M-03 `session_manager.rs`**: 状态机验证（行 314–326）、缓存裁剪策略（行 134–137）→ 移至 TypeScript
**M-04 `context_assembly.rs`**: 滑动窗口算法（行 78–83）、历史压缩策略（行 87–109）、硬编码执行策略（行 176–181）→ 策略参数由 TS 传入

## LOW 级别

**L-01** `data_standard.rs`: 默认约定的硬编码值
**L-02** `turn_executor.rs`: URL 推导、硬编码中文 prompt 模板
**L-03** `shell_control.rs`: 默认黑名单命令列表（合理保留）
**L-04** `agent_config_handler.rs`: URL 转换逻辑与 turn_executor 重复
**L-05** `scenario.rs`, `context.rs`: ✅ 无违规，纯文件 I/O

---

## 正确遵循 ADR-001 的文件

**Rust 侧（纯系统能力）:**
- `agent_runtime/version_manager.rs` — 可执行文件检测、版本管理、symlinks
- `agent_runtime/skill_installer.rs` — Git clone
- `agent_runtime/file_control.rs` — 文件快照、git diff、回滚
- `agent_runtime/launcher.rs` — 进程启动
- `agent_runtime/home_setup.rs` — 隔离 HOME 目录
- `agent_runtime/event_stream.rs` — SSE 事件总线
- `agent_runtime/audit.rs` — 文件访问审计日志
- `scenario.rs` — YAML 文件读写
- `context.rs` — YAML 文件读写
- `db/` — SQLite CRUD
- `fs/` — 文件系统操作

**TypeScript 侧（正确作为业务逻辑层）:**
- `src/orchestration/` — ConfigLoader, ConfigValidator, FlowExecutor, ActionRegistry, ContextMenuResolver, ExpressionEvaluator, ParameterResolver, PredicateRegistry, RuntimeContext
- `src/types/businessFlow.ts` — 完整的 TypeScript 类型定义
- `src/stores/flowStore.ts` — 状态管理 + IPC 封装
- `src/config/scenarios.ts` — 场景预设
- `src/config/menu.ts` — 菜单配置

---

## 搬迁优先级

| 优先级 | 内容 | 搬迁量 |
|--------|------|--------|
| **P0 立即** | `conductor.rs` DAG 引擎 → `DagEngine.ts` | ~500 行 Rust → TS |
| **P1 下一迭代** | `validation.rs` 图校验 → `FlowGraphValidator.ts` | ~200 行 Rust → TS |
| **P2 两迭代内** | handler 业务规则、ai_config 硬编码、session state machine | ~250 行 Rust → TS/CONFIG |
| **P3 积压** | data_standard 默认值、prompt 模板提取、URL 逻辑去重 | ~100 行 Rust → TS |

---

# 综合汇总

## 三份审计交叉问题

| 交叉领域 | Tauri 审计 | Rust 领域审计 | 边界审计 |
|---------|-----------|-------------|---------|
| `conductor.rs` DAG 引擎 | — | — | 🔴 BLOCKER |
| `validation.rs` 图校验 | — | — | 🟠 HIGH |
| `entities/` SeaORM 未使用 | — | 🔴 BLOCKER | — |
| CSP null | 🔴 BLOCKER | — | — |
| `db_execute_raw` 暴露 | 🔴 BLOCKER | — | — |
| `Result<String>` 错误处理 | — | 🟠 HIGH | — |
| SeaORM + Raw SQL 并存 | — | 🟠 HIGH | — |
| String 代替 enum | — | 🟡 MEDIUM | — |
| 测试严重不足 | — | 🟡 MEDIUM | — |
| handler 业务规则 | — | — | 🟡 MEDIUM |
| ai_config 硬编码 | — | — | 🟡 MEDIUM |
| State 管理 (OnceLock) | 🟠 HIGH | 🟡 MEDIUM | — |
| HTTP API 无认证 | 🟠 HIGH | — | — |
| 边界违规合计 | — | — | 🔴1 🟠1 🟡4 🟢5 |

## 建议的执行顺序

**第一阶段: 修 BLOCKER（本周）**
1. CSP 配置 + `db_execute_raw` 移除（Tauri 审计 #1, #2）
2. SeaORM Entity 处置决定（Rust 领域审计 #1）
3. `conductor.rs` 搬迁到 `DagEngine.ts` 方案设计（边界审计 #1）

**第二阶段: 修 HIGH（本月）**
4. `validation.rs` 搬迁到 TypeScript（边界审计 HIGH-01）
5. 错误处理统一（Rust 领域审计 #2）
6. State 迁移到 `tauri::State`（Tauri 审计 #3）
7. HTTP API 添加认证（Tauri 审计 #5）

**第三阶段: 修 MEDIUM（下月）**
8. handler 业务规则提取
9. ai_config 硬编码搬迁
10. String → enum 类型化
11. session state machine 提取
12. 测试补充