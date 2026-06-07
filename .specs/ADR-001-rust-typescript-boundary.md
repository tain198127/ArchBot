# ADR-001: Rust/TypeScript 职责边界

**状态**: 已确认  
**日期**: 2026-06-07  
**决策人**: tain198127  
**严重程度**: 🔴 架构级 — 影响全部后续开发

---

## 背景

ArchBot 的 Business Flow Designer 实现了以下分工：

```
当前实现（❌ 错误）:
  TypeScript（前端）
    ├── Vue Flow 画布
    ├── flowStore (Pinia 状态管理)
    └── 事件监听 + UI 展示

  Rust（后端）
    ├── DAG 解析（build_dag）
    ├── 拓扑排序（topological_sort / Kahn BFS）
    ├── AND/XOR 网关分支策略
    ├── 降级链策略（degradation chain）
    ├── Agent 执行调度
    └── 系统操作（文件 I/O、DB、LLM API）
```

682 行的 `conductor.rs` 中，绝大部分是业务逻辑（DAG 算法、网关策略、降级链），只有少量是系统操作（文件路径、事件发射、SQLite 更新）。

## 决策

**Rust 不碰业务逻辑。TypeScript 能做的事，不让 Rust 做。**

```
正确架构:
  TypeScript（业务逻辑层）
    ├── Vue Flow 画布（可视化定义）
    ├── Zod 校验（图合法性）
    ├── AgentFlow IR（统一图中间模型）
    ├── LangGraph.js（工作流执行引擎）
    │   ├── DAG/图解析
    │   ├── 拓扑排序 / 就绪集调度
    │   ├── 网关分支策略（AND/XOR/OR）
    │   ├── 降级链策略
    │   ├── 循环/回退/checkpoint
    │   ├── 人工审批暂停-恢复
    │   └── Agent 协作编排
    └── Tauri IPC 调用

  Rust（系统能力层 — 薄层，无业务逻辑）
    ├── 文件 I/O（读/写/遍历）
    ├── 数据库操作（SQLite CRUD）
    ├── LLM API 调用（AI Provider 集成）
    ├── 进程管理
    └── Tauri 事件发射
```

## 理由

1. **Tauri 的设计哲学是 Rust 作为 webview 的系统资源桥梁**，不是业务逻辑的容器。Rust 的存在是为了给 TypeScript 提供浏览器沙箱内无法做到的能力——文件系统、数据库、网络、进程。

2. **业务逻辑应该在靠近用户的一侧迭代。** 工作流引擎的策略（网关怎么分叉、失败怎么降级、Agent 怎么协作）是高频变更的业务决策，应该在 TypeScript 中快速迭代，不应该每次都要过 Rust 编译。

3. **LangGraph.js 是正确的工作流引擎选择。** 它原生支持 Agent 协作需要的循环、回退、checkpoint、人工介入——这些能力如果在 Rust 中从头实现，本质是在复刻 LangGraph。

4. **TypeScript 可以运行图算法。** DAG 解析、拓扑排序、就绪集调度都是纯计算逻辑，不依赖系统资源。放在 TypeScript 里完全可行，而且更容易测试和调试。

## 后果

### 对现有代码的影响

| 文件 | 处理方式 |
|------|---------|
| `conductor.rs` (682 行) | **大幅缩减** — 移除 DAG 解析、拓扑排序、网关策略、降级链；保留仅系统操作部分（事件发射、SQLite 更新、文件路径）|
| `validation.rs` (298 行) | **迁移到 TypeScript** — 循环检测、可达性检查、Start/End 节点校验 → Zod schema + 自定义 validator |
| `model.rs` (216 行) | **保留** — Rust 侧仍需数据类型用于 DB 读取/写入 |
| `handler.rs` (389 行) | **保留部分** — CRUD 命令保留；移除与执行逻辑耦合的部分 |
| `flowStore.ts` (310 行) | **重构为 LangGraph workflow** — 不再是简单的 Pinia store，而是 StateGraph 定义 |
| `businessFlow.ts` (288 行) | **扩展为 AgentFlow IR** — 补充 backtrack 边、coordinator 策略、checkpoint 语义 |

### 正向后果
- 工作流引擎迭代速度大幅提升（TypeScript 热更新 vs Rust 编译）
- 工作流策略可配置化（YAML/JSON 驱动，不需要改 Rust 代码）
- LangGraph.js 的开箱能力（checkpoint、interrupt、StateGraph）立即可用
- 前端状态和执行状态统一（单一真相来源在 TypeScript 侧）

### 负面后果
- `conductor.rs` 的 9 个测试需要重写（从 Rust → TypeScript）
- 需要新增 Tauri IPC 接口（LangGraph 调用 Rust 系统能力）
- 执行状态持久化策略变化（从 Rust 直接写 SQLite → TypeScript 通过 IPC 写）
- 当前的三位专家评审结论部分失效——他们假定的是 Rust 持有执行状态

### 风险
- 如果 webview 崩溃，正在执行的 LangGraph 状态会丢失 —— 需要 checkpoint 持久化策略
- 长时间执行的 Agent 流程需要处理前端页面关闭/刷新场景

## 参考资料

- `note/Workflow_IR_中心架构选型清稿.md` — 场景二：Vue Flow + AgentFlow IR + Zod + LangGraph
- Tauri 2 官方文档 — Rust 作为 backend，不承载业务逻辑
- LangGraph.js 文档 — StateGraph, checkpoint, interrupt/resume

## 三位专家评审的修正说明

三位专家的评审（记录在 `.specs/architecture-review-2026-06-07.md`）是在 Rust Conductor 作为执行态的假设下进行的。本 ADR 确立的边界原则**推翻了该假设**：

- 图运算专家的「移除 LangGraph.js」→ **无效**，LangGraph.js 现在是核心
- 工作流专家的「Conductor 互补 LangGraph」→ **部分保留**，Rust 作为能力后端，但不是对等角色
- Agent 编排专家的「先定义 IR，再决定执行器」→ **仍然有效**，AgentFlow IR 应先定义

评审报告中关于代码 Bug（拓扑排序循环处理、索引跳跃、AND-join 缺失）的发现**仍然有效**，但这些 Bug 将在 TypeScript 侧修复，而非 Rust 侧。
