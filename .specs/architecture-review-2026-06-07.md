# 三位专家架构评审汇总

> ⚠️ **注意**: 此评审的部分结论已被 [ADR-001](./ADR-001-rust-typescript-boundary.md) 推翻。
> ADR-001 确立了 Rust 不碰业务逻辑、TypeScript 全权负责工作流引擎的原则。
> 评审中的代码 Bug 发现仍然有效，但修复将在 TypeScript 侧进行。

评审日期: 2026-06-07
评审对象: ArchBot Business Flow Designer — LangGraph.js + Rust Conductor 架构方案

---

## 工作流引擎专家评审

### 总体结论
**当前 Conductor 是一个合格的 DAG scheduler 原型，但不是工作流执行引擎。** 提议的架构方向（IR 分层 + 多执行器）是正确的，但职责划分需要修正。

### Conductor 成熟度
| 能力 | 状态 |
|------|------|
| DAG 解析 + 拓扑排序 | ✅ 完整 |
| AND 网关并行 (tokio::spawn + Semaphore) | ✅ 完整 |
| XOR 网关条件分支 | ⚠️ MVP（只支持 "true"/"false"） |
| 降级链 | ❌ Stub（只实现 skip/terminate） |
| 取消机制 | ⚠️ 半完整（缺全局注册表） |
| Tauri 事件流 | ✅ 完整 |
| Agent 执行 | ❌ Stub（100ms sleep） |
| 节点级状态持久化 | ❌ 缺失（仅 run 级别） |
| 重试策略引擎 | ❌ Stub |
| 人工暂停/恢复 | ❌ 缺失 |

### 三个致命缺口（P0）
1. **节点级持久化** — `HashMap<String, NodeResult>` 是内存结构，崩溃即丢失
2. **人工暂停/恢复** — `human_review` 节点类型定义了但无执行逻辑
3. **重试策略引擎** — `RetryPolicy` 类型定义了但 `apply_degradation` 只读配置不执行

### 关键修正：Conductor 和 LangGraph 是互补关系，不是替代

```
AgentFlow IR
    ↓
LangGraph.js (状态图协调器: 循环、回退、条件路由、checkpoint)
    ↓ 每个 node 执行时调用
Rust Conductor (系统能力提供者: LLM 调用、文件 I/O、事件发射)
```

### 建议路线图
- **短期**: 补齐持久化 + 重试 + 人工暂停，Conductor 成为"能用的"轻量工作流引擎
- **中期**: 引入 AgentFlow IR + LangGraph.js 协调器，Conductor 退化为节点执行器
- **远期**: JobFlow IR + Temporal/Kestra

---

## Agent 编排专家评审

### 总体结论
**需要调整 — 不是致命缺陷。** 方案的方向正确（IR 中心架构），但核心存在未解决的矛盾。

### 核心矛盾：LangGraph.js vs Rust Conductor 的分工
提议方案让 LangGraph 做「前端状态管理」，Rust 做「执行工作」。但 **LangGraph 的核心价值恰恰是图执行运行时**（循环、回退、checkpoint、人工介入）—— 这正是 Agent 协作需要的。目前的划分让双方都无法拥有图执行的完整所有权。

### 两种可行路径

**路径 A — 扩展 Rust Conductor（推荐当前阶段）**
- 扩展 Conductor 支持循环图（backtrack 边、max-iteration 循环检测）
- 基于 `flow_runs` 表添加 checkpoint/resume
- AgentFlow IR 定义为 TypeScript 类型
- 增量工作，不重写

**路径 B — LangGraph.js 做主执行器（Agent 复杂度显著增长后）**
- LangGraph.js 拥有图拓扑和执行流
- Rust Conductor 退化为"工具节点"，通过 Tauri IPC 被 LangGraph 调用
- 仅在需要 LangSmith/Studio 调试、动态图生成、或 checkpoint 序列化复杂时考虑

### AgentFlow IR 缺失的关键语义

| 缺失概念 | 为什么重要 | 建议 |
|---------|-----------|------|
| **Backtrack 边** | "Agent B 失败，回退到 Agent A 带新上下文" - Agent 协作的第一模式 | `EdgeKind: 'forward' \| 'backtrack' \| 'conditional'` |
| **Coordinator 策略** | 3 个 Agent 产生冲突输出时如何裁决 | `CoordinatorPolicy { strategy: 'majority' \| 'weighted' \| 'llm_judge' }` |
| **Checkpoint 语义** | 人工确认：展示什么、等多久、默认操作 | `CheckpointConfig { prompt, timeoutMs, defaultAction }` |
| **循环边界** | 没有 max_iterations，backtrack 会无限循环 | `LoopConfig { maxIterations, convergenceCondition }` |
| **上下文 Schema** | 目前 `unknown` 太模糊 | `ContextSchema: z.ZodSchema` |
| **子图 I/O 映射** | 上下文如何进入子流程、结果如何返回 | `SubFlowMapping { input, output }` |

### 核心场景覆盖

| 场景 | 现状 | 差距 |
|------|------|------|
| Agent 失败回退重跑 | ❌ 不支持 | 无 backtrack 边，拓扑排序假设 DAG |
| Coordinator 仲裁冲突 | ⚠️ 部分 | 节点类型存在但无 policy schema |
| 人工审批后继续 | ❌ 未实现 | Schema 有但 Conductor 无执行逻辑；LangGraph 原生支持 |

### 推荐：分阶段路线

**阶段 1（现在）: 先完整定义 AgentFlow IR**
- 扩展 `businessFlow.ts` 覆盖上述缺失语义
- 纯类型定义 — 低风险，高清晰度

**阶段 2（短期）: 扩展 Rust Conductor**
- 添加 `execute_human_approval` + checkpoint 持久化
- 添加 backtrack 边 + max-iteration 循环边界
- 添加 coordinator 节点 + 冲突仲裁策略

**阶段 3（评估）: 决定是否引入 LangGraph.js**
- 阶段 2 后评估 Conductor 是否覆盖全部需求
- 如果 Agent 图变动态（LLM 生成而非手绘）→ LangGraph 更有吸引力
- 如果 checkpoint 序列化变复杂 → LangGraph 内置 checkpointer 有价值
- 如果不需要 → 延迟引入，避免增加第二运行时

---

## 图运算专家评审

### 总体结论
**方案需要调整。** 方向正确，但有三类问题需要修复。

### 图算法缺陷分析

#### Bug #1：拓扑排序循环处理是隐患，不是策略
```rust
// conductor.rs:452-456 — 循环节点被追加到末尾
for node in &graph.nodes {
    if !order.contains(&node.id) {
        order.push(node.id.clone());
    }
}
```
A→B→A 循环中，A 和 B 都会在缺少合法输入的情况下执行，产生静默错误。**正确做法：** 运行时显式拒绝循环图。

#### Bug #2：执行循环索引跳跃假设错误
`i += 1 + skip_count` 假设下游节点在拓扑序中紧邻网关。Kahn 算法只保证前驱-后继顺序，不保证空间邻接。跳过的可能是无关节点。

**正确做法：** 用就绪集（frontier）替代索引递增——出队节点后检查所有后继的依赖是否满足。

#### Bug #3：`rev_adj` 被构建但从未在执行阶段使用
AND-join（多入度节点需等待所有上游完成）需要用它——但被完全忽略。

### 关键结论：移除 LangGraph.js

```
┌─ 提议方案（不合理）─┐          ┌─ 正确方案 ──────────────────────┐
│                      │          │                                 │
│ AgentFlow IR         │          │ AgentFlow IR (JSON Schema)      │
│  ↓          ↓        │          │    ↓                            │
│ LangGraph   Conductor│          │ Rust Conductor (唯一图运行时)    │
│ (前端)      (后端)    │          │   ├─ DAG + 就绪集遍历            │
│                      │          │   ├─ AND/OR/XOR 网关            │
│ 同一张图两个运行时     │          │   ├─ 降级链 + 重试               │
│ 不同语义理解           │          │   ├─ mpsc 双向协议 (审批)       │
│ 职责重叠              │          │   └─ Tauri 事件发射             │
│                      │          │ SQLite 持久化                    │
│                      │          │                                 │
│                      │          │ 前端：Vue Flow + flowStore       │
│                      │          │ 不需要第二个图运行时               │
└──────────────────────┘          └─────────────────────────────────┘
```

### 必须修复（阻塞级）

| # | 问题 | 修复 |
|---|------|------|
| 1 | 拓扑排序循环节点追加到末尾 | `order.len() != graph.nodes.len()` → 返回错误 |
| 2 | 执行循环索引跳跃 | 用就绪集替代：出队 → 检查后继 → 就绪则入队 |
| 3 | LangGraph.js 冗余 | 移除；flowStore + Tauri Event Listener 足够 |

### 应该修复（正确性级别）

| # | 问题 | 修复 |
|---|------|------|
| 4 | AND-join 缺失 | execute 前用 rev_adj 检查所有前驱已完成 |
| 5 | OR 网关未实现 | 实现"任一条件满足即触发" |
| 6 | human_approval 无双向协议 | `tokio::sync::mpsc` 通道实现暂停-响应 |
| 7 | 子图递归是空桩 | 实现递归加载、执行、合并结果 |

### 关键原则
1. **Rust Conductor 是图执行态的唯一持有者**（单一真相来源）
2. **前端是图定义态的编辑器 + 执行态的只读视图**
3. **AgentFlow IR 是被共享的序列化格式**，不是被共享的运行时

---

## 三位专家共识总结

### 一致同意的结论
1. **Workflow IR 中心架构方向正确** — 三位专家一致认可
2. **LangGraph.js 不应作为前端状态管理器** — 图运算专家建议直接移除；Agent 编排专家建议最多在阶段 3 评估；工作流专家建议退化为节点执行器池
3. **Rust Conductor 是执行态的唯一持有者** — 单一真相来源原则
4. **当前 Conductor 必须补齐三个核心能力：节点级持久化、人工暂停/恢复、重试策略引擎**

### 分歧点
| 议题 | 工作流专家 | Agent 编排专家 | 图运算专家 |
|------|----------|--------------|----------|
| LangGraph 未来角色 | 中期引入做状态图协调器 | 阶段3评估，视需要决定 | 不需要，直接移除 |
| Conductor 定位 | 从全栈引擎→节点执行器池 | 短期扩展，中期评估 | 唯一图运行时，持续扩展 |
| 循环图支持 | LangGraph 更适合 | 两条路都可，取决于 IR | Conductor 应自己实现 |

### 推荐路线图（综合三位意见）

**阶段 1 — 立即（不改架构）**
- 修复图算法的 3 个 bug（拓扑排序循环处理、索引跳跃、AND-join）
- 定义 AgentFlow IR（扩展 TypeScript 类型）
- **不做 LangGraph.js 迁移** — 三位专家一致认为当前不需要

**阶段 2 — 短期（补齐核心能力）**
- 节点级执行状态持久化到 SQLite
- 重试策略引擎（指数退避 + maxRetries）
- 人工暂停/恢复（mpsc 双向协议）
- Agent stub → agent_runtime 集成

**阶段 3 — 中期（评估 LangGraph）**
- 当 Agent 图变动态生成（LLM 产出）、或需要复杂循环/回退语义、或 checkpoint 序列化足够复杂时，再评估引入 LangGraph.js

