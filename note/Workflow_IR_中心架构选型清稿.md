# Workflow IR 中心架构选型清稿

## 1. 核心结论

不要寻找一个“万能工作流库”解决所有问题。

你的三个场景本质不同：

1. 页面按钮、页面逻辑、页面状态控制
2. 多个 Agent 协作，并支持 Vue 拖拽式定义流程
3. 不同进程之间的长流程，例如 Agent 训练、评估、发布流水线

这三类问题不能用同一个工作流工具硬扛。正确方式是建立一个以 **Workflow IR** 为中心的分层架构：

```txt
Vue Flow 做可视化定义
Zod 做合法性校验
Workflow IR 做统一中间模型
XState 执行页面状态
LangGraph / Mastra 执行 Agent 协作
Temporal / Kestra 执行跨进程长流程
```

一句话定型：

> XState 管前端状态，Vue Flow 管拖拽画布，Zod 管配置校验，Workflow IR 管统一模型，LangGraph/Mastra 管 Agent 协作，Temporal/Kestra 管跨进程可靠执行。

---

## 2. 它们之间是什么关系

这些工具不是天然一套东西，而是分属不同层。

推荐关系如下：

```txt
┌──────────────────────────────────────────────┐
│ Vue 页面 / 拖拽画布                           │
│ Vue Flow / React Flow / 自研 Canvas           │
└──────────────────────────────────────────────┘
                    ↓
┌──────────────────────────────────────────────┐
│ 外部配置表达层                                │
│ workflow.yml / workflow.json / UI graph JSON  │
└──────────────────────────────────────────────┘
                    ↓
┌──────────────────────────────────────────────┐
│ 校验层                                       │
│ Zod / JSON Schema / AJV                      │
└──────────────────────────────────────────────┘
                    ↓
┌──────────────────────────────────────────────┐
│ 内部统一模型层                                │
│ Workflow IR                                  │
└──────────────────────────────────────────────┘
       ↓                  ↓                  ↓
┌──────────────┐   ┌──────────────┐   ┌──────────────┐
│ 页面状态执行  │   │ Agent 编排执行 │   │ 跨进程可靠执行 │
│ XState       │   │ LangGraph     │   │ Temporal     │
│              │   │ Mastra        │   │ Kestra       │
└──────────────┘   └──────────────┘   └──────────────┘
```

核心原则：

> 不要让 Vue Flow 直接生成 XState / LangGraph / Temporal。  
> 先生成自己的 Workflow IR，再由 IR 转成不同执行器。

否则前端画布会被某个后端执行框架绑死，后期替换成本很高。

---

## 3. 分工矩阵

| 工具 | 本质 | 在系统里的位置 | 是否建议使用 |
|---|---|---|---|
| Zod | Schema 校验库 | 检查拖拽生成的配置、AI 生成的流程是否合法 | 必用 |
| Vue Flow | Vue 拖拽节点画布 | 做流程设计器、节点拖拉拽、连线 | 必用 |
| XState | 状态机 / 状态图运行库 | 控制页面状态、按钮状态、表单状态、局部流程 | 必用 |
| Workflow IR | 内部统一流程模型 | 所有拖拽、AI、配置最终都转成它 | 必须自定义 |
| LangGraph | Agent 状态图编排 runtime | 多 Agent 协作、循环、人工介入、状态持久 | 建议使用 |
| Mastra | TypeScript AI Agent 应用框架 | Agent、Tool、Memory、RAG、Workflow 应用层 | 可选 |
| Temporal | 分布式可靠执行引擎 | 长流程、跨进程、失败恢复、重试、补偿 | 重要流程使用 |
| Kestra | YAML 工作流平台 | 任务流、脚本流、数据流、训练流水线、平台化执行 | 可选，偏平台 |
| n8n | 低代码自动化平台 | 可参考交互形态，不建议作为核心内核 | 不建议核心依赖 |

关键判断：

```txt
Zod      = 校验器
Vue Flow = 可视化流程画布
XState   = 页面状态机
LangGraph= Agent 状态图运行时
Mastra   = AI Agent 应用框架
Temporal = 可靠执行引擎
Kestra   = 声明式任务编排平台
Workflow IR = 内部统一语义模型
```

---

## 4. 场景一：页面按钮、页面逻辑、状态控制

### 4.1 结论

使用：

```txt
Vue + XState + Zod + PageFlow IR
```

不建议使用：

```txt
Temporal
Kestra
n8n
LangGraph
Mastra
```

这些工具对页面按钮逻辑过重。

### 4.2 本质判断

页面按钮、页面逻辑关系，本质不是传统工作流，而是：

```txt
UI State Machine
```

例如：

```txt
草稿状态：
  - 保存按钮可点
  - 提交按钮可点
  - 审核按钮不可见

审核中：
  - 保存按钮不可点
  - 提交按钮不可点
  - 撤回按钮可点

已发布：
  - 所有编辑按钮不可点
  - 查看按钮可点
```

这不是任务编排，也不是 Agent 编排，而是状态约束。

### 4.3 推荐结构

```txt
page-flow.yml
    ↓
Zod 校验
    ↓
PageFlow IR
    ↓
XState machine
    ↓
Vue 页面绑定
```

页面配置示例：

```yaml
id: bond_edit_page
initial: draft

states:
  draft:
    buttons:
      save:
        visible: true
        enabled: true
      submit:
        visible: true
        enabled: true
      approve:
        visible: false
        enabled: false
    on:
      SUBMIT: reviewing

  reviewing:
    buttons:
      save:
        visible: true
        enabled: false
      submit:
        visible: false
        enabled: false
      withdraw:
        visible: true
        enabled: true
    on:
      WITHDRAW: draft
      APPROVE: approved

  approved:
    buttons:
      save:
        visible: true
        enabled: false
      view:
        visible: true
        enabled: true
```

Vue 页面中只做绑定：

```txt
按钮是否显示 = 当前状态配置
按钮是否可点 = 当前状态配置
按钮点击 = send(event)
```

### 4.4 技术判断

| 问题 | 选择 |
|---|---|
| 页面状态流转 | XState |
| 拖拽式编辑页面状态 | Vue Flow |
| 配置校验 | Zod |
| 页面逻辑统一模型 | PageFlow IR |
| 是否需要 Temporal | 不需要 |
| 是否需要 LangGraph | 不需要 |
| 是否需要 Mastra | 不需要 |

---

## 5. 场景二：多个 Agent 拖拽式协作编排

### 5.1 结论

使用：

```txt
Vue Flow + Workflow IR + Zod + LangGraph
```

Mastra 可作为 Agent 应用框架，但不要让 Mastra 成为唯一流程模型。

### 5.2 本质判断

这个场景不是普通 workflow，而是：

```txt
Agent Graph
```

它需要表达：

```txt
Agent A 分析需求
Agent B 分析代码
Agent C 生成方案
Agent D 审查风险
Coordinator 汇总冲突
人工确认
回到某个 Agent 重跑
```

Agent 流程通常具备：

```txt
循环
回退
人工介入
状态传递
上下文裁剪
工具调用
子图
失败重试
多 Agent 冲突仲裁
```

因此，它不能简单等同于普通 DAG。

### 5.3 推荐结构

```txt
Vue Flow 画布
    ↓
graph.json
    ↓
Zod 校验
    ↓
AgentWorkflow IR
    ↓
LangGraph 编译器
    ↓
LangGraph runtime 执行
    ↓
执行轨迹 / 日志 / 状态回写 UI
```

错误做法：

```txt
Vue Flow Node = LangGraph Node
```

正确做法：

```txt
Vue Flow Node
    ↓
AgentWorkflow IR Node
    ↓
LangGraph Node / Mastra Workflow Step
```

### 5.4 AgentWorkflow IR 示例

```ts
type AgentWorkflowIR = {
  id: string
  version: string
  nodes: AgentNodeIR[]
  edges: AgentEdgeIR[]
  contextSchema: unknown
  policies: WorkflowPolicy[]
}

type AgentNodeIR = {
  id: string
  kind:
    | "agent"
    | "tool"
    | "human_review"
    | "router"
    | "coordinator"
    | "subworkflow"
    | "start"
    | "end"

  agentRef?: string
  toolRef?: string
  inputMapping?: Record<string, string>
  outputMapping?: Record<string, string>
  timeout?: string
  retry?: RetryPolicy
}

type AgentEdgeIR = {
  from: string
  to: string
  condition?: unknown
}
```

### 5.5 Mastra 和 LangGraph 的关系

| 判断点 | 选 LangGraph | 选 Mastra |
|---|---|---|
| 多 Agent 状态图、循环、回退、人工介入 | 更适合 | 可以，但不是最强项 |
| 快速开发 AI app、agent、tool、memory、RAG | 可以 | 更适合 |
| 底层可控的 Agent Graph runtime | 更适合 | 中 |
| TypeScript AI 应用框架 | 中 | 更适合 |
| Vue 拖拽画布自定义执行 | 更适合 | 可作为 agent/tool 层 |

推荐组合：

```txt
Agent 图执行：LangGraph
Agent 能力封装：Mastra 可选
画布定义：Vue Flow
统一模型：AgentWorkflow IR
```

更明确地说：

```txt
Vue Flow 负责画
Workflow IR 负责存
LangGraph 负责跑图
Mastra 负责封装 Agent/Tool 能力
```

---

## 6. 场景三：跨进程长流程，例如训练 Agent

### 6.1 结论

这里不要用 XState 当主执行器，也不要只用 LangGraph。

使用：

```txt
Workflow IR + Temporal
```

或者：

```txt
Workflow IR + Kestra
```

选择标准：

```txt
偏平台化、YAML、任务流、脚本流：Kestra
偏代码可靠执行、失败恢复、长事务、强一致执行语义：Temporal
```

### 6.2 本质判断

不同进程之间的流程、Agent 训练流程，本质是：

```txt
Distributed Job Pipeline
```

它可能包含：

```txt
准备数据
生成训练样本
启动 Agent 训练进程
评估 Agent
失败重试
生成报告
人工确认
发布新版本 Agent
回滚
多进程隔离
日志采集
指标采集
```

这类流程具备以下特点：

| 特点 | 意味着什么 |
|---|---|
| 时间长 | 不能只靠内存状态 |
| 多进程 | 要有任务队列 / worker |
| 容易失败 | 要有 retry / resume |
| 需要审计 | 要记录每一步状态 |
| 需要产物 | 要管理 artifacts |
| 可能需要人工确认 | 要支持暂停 / 恢复 |
| 需要资源隔离 | 可能要 Docker / process runner |

这已经不是页面状态机，也不是简单 Agent Graph，而是可靠任务编排。

### 6.3 Temporal vs Kestra

| 维度 | Temporal | Kestra |
|---|---|---|
| 本质 | 可靠执行引擎 | 声明式工作流平台 |
| 表达方式 | TypeScript / Go / Java / Python 代码 | YAML |
| UI | 运行态 UI | 流程定义 + 执行 UI 更完整 |
| 可靠性 | 极强 | 强 |
| 跨进程任务 | 强 | 强 |
| 数据 / 脚本流水线 | 可以做，但不是最顺手 | 很适合 |
| Agent 训练流水线 | 适合关键核心链路 | 适合平台化任务管理 |
| AI 生成友好 | 中 | 高 |
| 工程复杂度 | 高 | 中高 |
| 本地嵌入 | 偏重 | 偏平台服务 |
| 长事务恢复 | 强项 | 有，但不是 Temporal 那种模型 |

### 6.4 推荐判断

产品早期：

```txt
先用 Kestra 思路或自研轻量 runner
```

如果明确要金融级、长期运行、失败恢复：

```txt
用 Temporal
```

如果既要平台 UI，又要强任务流：

```txt
Kestra 做训练流水线平台
Agent 内部执行用 LangGraph/Mastra
关键流程未来再接 Temporal
```

---

## 7. 组合以后适合做什么

### 7.1 组合 A：页面状态控制

```txt
Vue + Vue Flow + XState + Zod + PageFlow IR
```

适合：

```txt
按钮状态
页面切换
表单步骤
权限状态
编辑 / 审核 / 发布状态
复杂页面交互
```

不适合：

```txt
长时间后台训练
多进程任务
复杂 Agent 协作
```

### 7.2 组合 B：Agent 拖拽编排器

```txt
Vue Flow + Zod + AgentWorkflow IR + LangGraph + Mastra
```

适合：

```txt
多 Agent 协同
Agent 路由
Agent 审查
Agent 冲突仲裁
人工介入
循环修正
工具调用
上下文传递
```

不适合单独承担：

```txt
跨进程长任务可靠恢复
训练流水线资源调度
```

### 7.3 组合 C：Agent 训练流水线

```txt
Workflow IR + Kestra / Temporal + LangGraph / Mastra + Artifact Store
```

适合：

```txt
数据准备
样本生成
Agent prompt 版本训练
工具能力测试
多模型评测
Agent benchmark
人工验收
发布 / 回滚
日志 / 指标 / 审计
```

推荐结构：

```txt
训练流程定义
    ↓
Workflow IR
    ↓
Temporal / Kestra 负责流程调度
    ↓
每个步骤调用独立 worker 进程
    ↓
worker 内部调用 LangGraph / Mastra agent
    ↓
产物写入 artifact store
    ↓
指标写入 evaluation store
```

---

## 8. 不要做一个巨型 Workflow IR

建议拆成三类 IR，而不是一套 IR 包打天下。

```txt
1. PageFlow IR
   解决页面状态、按钮、权限、交互

2. AgentFlow IR
   解决多个 Agent 协作、工具调用、人工介入

3. JobFlow IR
   解决跨进程、长任务、训练流水线、失败恢复
```

它们共享基础结构：

```txt
node
edge
condition
input
output
metadata
policy
```

但执行语义不同。

---

## 9. 三类 IR 和工具映射

| IR | 解决什么 | 前端展示 | 校验 | 执行器 | 备注 |
|---|---|---|---|---|---|
| PageFlow IR | 页面状态、按钮、表单、权限 | Vue Flow / 普通配置页 | Zod | XState | 最轻 |
| AgentFlow IR | 多 Agent、工具、路由、人工介入 | Vue Flow | Zod | LangGraph / Mastra | 中等复杂 |
| JobFlow IR | 跨进程任务、训练、评估、发布 | Vue Flow / Kestra UI | Zod / JSON Schema | Temporal / Kestra | 最重 |

---

## 10. 推荐落地顺序

### 第一步：先做 PageFlow

原因：最小闭环，最容易验证。

```txt
Vue 页面
    ↓
page-flow.yml
    ↓
Zod
    ↓
PageFlow IR
    ↓
XState
    ↓
按钮 / 页面状态
```

先证明“配置驱动页面逻辑”可行。

### 第二步：做 AgentFlow

```txt
Vue Flow 拖拽 Agent 节点
    ↓
agent-flow.json
    ↓
Zod
    ↓
AgentFlow IR
    ↓
LangGraph
    ↓
执行 Agent 协作
```

重点设计：

```txt
上下文传什么
每个 Agent 输入输出是什么
失败如何处理
如何防止死循环
人工确认点在哪里
协调者如何裁决冲突
```

### 第三步：做 JobFlow

```txt
job-flow.yml
    ↓
JobFlow IR
    ↓
Temporal / Kestra
    ↓
多进程 worker
    ↓
Agent 训练 / 评估 / 发布
```

这里不要急。一开始可以先用自研轻量 runner。等确实出现长任务恢复、失败重试、审计需求，再接 Temporal 或 Kestra。

---

## 11. 不建议的路线

### 11.1 不建议用 n8n 做核心

n8n 适合自动化和集成，但不适合作为核心流程内核。

你的目标是：

```txt
页面状态控制 + Agent 编排 + 跨进程训练
```

n8n 可以参考交互形态，不建议做核心依赖。

### 11.2 不建议直接让 Vue Flow 生成执行器格式

不要这样：

```txt
Vue Flow JSON -> XState
Vue Flow JSON -> LangGraph
Vue Flow JSON -> Temporal
```

应该这样：

```txt
Vue Flow JSON -> Workflow IR -> XState / LangGraph / Temporal
```

### 11.3 不建议用 XState 做所有流程

XState 很适合页面状态，但不适合长时间训练流程。

错误用法：

```txt
XState 管按钮
XState 管 Agent
XState 管跨进程训练
XState 管任务恢复
```

这会把 XState 用成小型 Temporal，最后会崩。

### 11.4 不建议用 Temporal 做页面状态

Temporal 太重。页面按钮逻辑用 Temporal，是把导弹拿来开罐头。

---

## 12. 最终推荐方案

| 你的需求 | 推荐组合 | 不推荐 |
|---|---|---|
| 页面按钮和页面逻辑，用状态控制 | XState + Zod + PageFlow IR | Temporal / Kestra / LangGraph |
| Vue 拖拽定义多个 Agent 协作流程 | Vue Flow + AgentFlow IR + Zod + LangGraph，Mastra 可选 | 直接用 n8n 当核心 |
| 不同进程之间的 Agent 训练流程 | JobFlow IR + Temporal 或 Kestra + Worker 进程 | 只用 XState / 只用 LangGraph |

最终结构：

```txt
Vue Flow 做可视化定义
Zod 做合法性校验
Workflow IR 做统一中间模型
XState 执行页面状态
LangGraph / Mastra 执行 Agent 协作
Temporal / Kestra 执行跨进程长流程
```

更硬的架构判断：

> 不要寻找“一个工作流库”解决全部问题。你要做的是一个 Workflow IR 中心架构：前端用 Vue Flow 画，配置用 Zod 校验，页面交互交给 XState，Agent 图交给 LangGraph/Mastra，长任务交给 Temporal/Kestra。
