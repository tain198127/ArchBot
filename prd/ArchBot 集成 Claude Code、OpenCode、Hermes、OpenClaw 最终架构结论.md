# ArchBot 集成 Claude Code、OpenCode、Hermes、OpenClaw 最终架构结论

## 一、总体结论

ArchBot 应定位为：

**本地 Agent 控制平面。**

Claude Code、OpenCode、Hermes、OpenClaw 不应硬嵌入 Tauri 主程序，也不应直接成为 ArchBot 业务代码的一部分。

最终架构结论如下：

```text
ArchBot Tauri 主程序
  ├── Vue 3 前端
  ├── Rust 后端
  ├── Axum HTTP Server
  ├── Agent Runtime Manager
  ├── Agent Session Manager
  ├── Agent Adapter Manager
  ├── Agent Event Stream Manager
  ├── Runtime Version Manager
  ├── Context Manager
  ├── Audit Log Manager
  └── SQLite / LanceDB / 项目级 .archbot 目录
```

外部 Agent Runtime 的定位如下：

| Runtime     | 定位                           |
| ----------- | ------------------------------ |
| Claude Code | 长期会话型代码 Agent Runtime   |
| OpenCode    | 长期会话型代码 Agent Runtime   |
| Hermes      | 长期会话型泛任务 Agent Runtime |
| OpenClaw    | 长期会话型泛任务 Agent Runtime |

ArchBot 负责：

1. 会话主控。
2. 上下文组织。
3. 任务轮次管理。
4. Runtime 管理。
5. Adapter 管理。
6. 模型配置管理。
7. 流式事件接收。
8. 文件变更记录。
9. 日志审计。
10. 版本锁定。
11. 升级回滚。

外部 Runtime 负责：

1. 每一轮 Agent 执行。
2. 代码分析。
3. 代码修改。
4. 任务推理。
5. 工具调用。
6. 流式输出。
7. 生成中间结果和最终结果。

------

## 二、核心设计原则

最终采用：

```text
ArchBot 会话主控
Runtime 每轮执行
Adapter 协议适配
HTTP + SSE 结构化通信
input.yml 作为真实输入
prompt.txt 作为启动指令
```

核心原则：

```text
Session First
Turn Second
Runtime Per Turn Execution
```

即：

```text
ArchBot 管 Session
Runtime 执行 Turn
Adapter 负责协议转换
```

禁止模式：

```text
Vue 前端直接调用 Claude Code
Vue 前端直接调用 OpenCode
Vue 前端直接调用 Hermes
Vue 前端直接调用 OpenClaw
ArchBot 业务代码散落调用各 Runtime
Runtime 反向调用 ArchBot 内部接口
Runtime 直接访问 ArchBot 数据库
Runtime 直接访问 ArchBot LanceDB
Runtime 直接读取 ArchBot 全局配置
```

推荐模式：

```text
Vue 前端
  -> ArchBot Rust 后端
    -> Agent Runtime Manager
      -> Agent Adapter Service
        -> Claude Code / OpenCode / Hermes / OpenClaw
```

------

## 三、ArchBot 与 Runtime 的关系

ArchBot 不直接面对 Claude Code、OpenCode、Hermes、OpenClaw 的内部细节。

ArchBot 统一面对：

```text
Agent Adapter Service
```

每类 Runtime 对应一个 Adapter：

```text
archbot-adapter-claude-code
archbot-adapter-opencode
archbot-adapter-hermes
archbot-adapter-openclaw
```

Adapter 的职责：

1. 封装 Runtime SDK。
2. 封装 Runtime API。
3. 必要时封装 CLI fallback。
4. 把 Runtime 事件转换为 ArchBot 标准事件。
5. 把 Runtime 错误转换为 ArchBot 标准错误。
6. 把 Runtime 能力转换为 ArchBot 标准 capabilities。
7. 屏蔽 Runtime 版本差异。
8. 屏蔽不同 Runtime 的模型配置差异。
9. 屏蔽不同 Runtime 的会话模型差异。

------

## 四、通信机制最终结论

ArchBot 与 Agent Adapter 之间，主通信机制采用：

```text
HTTP + SSE
```

其中：

```text
HTTP：命令通道
SSE：流式事件通道
```

不采用 JSON-RPC over stdio 作为主通信机制。

JSON-RPC over stdio 仅可作为极简本地 fallback，不作为主架构。

最终通信结构：

```text
ArchBot
  |
  | HTTP Command API
  | SSE Event Stream
  v
Agent Adapter Service
  |
  | SDK / API / CLI fallback
  v
Claude Code / OpenCode / Hermes / OpenClaw
```

------

## 五、选择 HTTP + SSE 的结论

选择 HTTP + SSE 的根本原因：

ArchBot 不是简单的本地 CLI Wrapper，而是长期会话型、多 Runtime、可观测、可扩展的 Agent 控制平面。

HTTP + SSE 更适合：

1. 长期会话。
2. 多客户端。
3. Browser Mode。
4. 本地服务化。
5. 外部客户端扩展。
6. 调试观测。
7. Runtime 健康检查。
8. Runtime capabilities 查询。
9. 事件流。
10. 后续远程化。

JSON-RPC over stdio 更适合：

1. 单一子进程插件。
2. 本地一次性调用。
3. 简单 sidecar。
4. 不占端口的极简模式。

但 ArchBot 的目标不是极简插件模式，而是 Agent 控制平面，因此主方案选择 HTTP + SSE。

------

## 六、Adapter 服务形态

每个 Adapter 是一个本地 Agent Adapter Service。

默认监听：

```text
127.0.0.1
```

端口由 ArchBot 启动时分配。

Adapter 不应默认监听公网地址。

Adapter 启动时由 ArchBot 注入：

```text
adapter_id
runtime_type
runtime_version
listen_host
listen_port
auth_token
config_path
workspace_root
log_dir
```

Adapter 对外暴露 HTTP API 和 SSE 事件流。

Adapter 不直接暴露给普通用户。

Adapter 默认只接受 ArchBot 注入的认证 Token。

------

## 七、HTTP 命令接口

Agent Adapter Service 至少提供以下接口：

```text
GET  /v1/runtime/health
GET  /v1/runtime/capabilities
GET  /v1/runtime/version

POST /v1/sessions
GET  /v1/sessions/{session_id}
POST /v1/sessions/{session_id}/turns
GET  /v1/sessions/{session_id}/turns/{turn_id}
POST /v1/sessions/{session_id}/turns/{turn_id}/cancel
```

核心请求：

```text
POST /v1/sessions/{session_id}/turns
```

示例请求：

```json
{
  "session_id": "sess_001",
  "turn_id": "turn_008",
  "input_file": ".archbot/agent/sessions/sess_001/turns/turn_008/input.yml",
  "prompt_file": ".archbot/agent/sessions/sess_001/turns/turn_008/prompt.txt",
  "stream": true
}
```

------

## 八、SSE 流式事件接口

每一轮 Turn 的事件流接口：

```text
GET /v1/sessions/{session_id}/turns/{turn_id}/events
```

事件格式：

```text
event: turn.delta
data: {"session_id":"sess_001","turn_id":"turn_008","text":"..."}

event: turn.tool_started
data: {"session_id":"sess_001","turn_id":"turn_008","tool":"..."}

event: turn.tool_finished
data: {"session_id":"sess_001","turn_id":"turn_008","tool":"..."}

event: turn.completed
data: {"session_id":"sess_001","turn_id":"turn_008"}

event: turn.failed
data: {"session_id":"sess_001","turn_id":"turn_008","error":"..."}
```

SSE 用于：

1. 模型输出增量。
2. 工具调用事件。
3. 状态变化。
4. 中间结果。
5. 错误事件。
6. 完成事件。
7. 文件变更事件。
8. 心跳事件。

------

## 九、会话主控结论

ArchBot 保存会话主状态。

Runtime 不作为会话主数据库。

Claude Code、OpenCode、Hermes、OpenClaw 都只是每一轮的执行 Runtime。

核心模型：

```text
AgentSession
  -> AgentTurn
      -> RuntimeInvocation
          -> AgentEvent
          -> AgentArtifact
```

ArchBot 负责保存：

1. 会话目标。
2. 会话历史。
3. 最近消息窗口。
4. 历史摘要。
5. 关键决策。
6. 上下文快照。
7. 文件变更。
8. 执行结果。
9. 事件流。
10. 产物。

Runtime 负责执行：

1. 当前 Turn。
2. 当前推理。
3. 当前代码修改。
4. 当前任务推进。

------

## 十、长期会话上下文策略

长期会话采用：

```text
滑动窗口 + 历史摘要 + 关键决策 + 当前相关上下文
```

不采用全量历史一直传递。

不采用只发当前问题。

每一轮调用 Runtime 时，ArchBot 重新组装上下文。

每一轮输入包括：

```text
用户当前输入
ArchBot 解释后的意图
最近 N 轮原文
历史摘要
关键决策
当前相关文件
当前相关产物
当前执行策略
```

这样可以保证：

1. 长期会话不会丢失主线。
2. Runtime 不依赖自身记忆。
3. 会话可以在不同 Runtime 间迁移。
4. 会话可以审计。
5. 会话可以压缩。
6. 会话可以恢复。

------

## 十一、每轮输入机制

每一轮 Runtime 调用采用双输入机制：

```text
input.yml 为主
prompt.txt 为辅
```

其中：

```text
input.yml：真实结构化输入
prompt.txt：启动指令
```

流程：

```text
ArchBot 生成 input.yml
ArchBot 生成 prompt.txt
ArchBot 调用 Adapter HTTP API
Adapter 调用 Runtime SDK / API / CLI
Runtime 读取 input.yml
Runtime 根据 prompt.txt 启动执行
Runtime 输出事件流
Adapter 转换事件
ArchBot 接收 SSE
ArchBot 落库和展示
```

------

## 十二、会话目录结构

每个会话一个目录：

```text
项目目录/.archbot/agent/sessions/{session_id}/
```

每一轮一个目录：

```text
项目目录/.archbot/agent/sessions/{session_id}/turns/{turn_id}/
```

每轮目录结构：

```text
turn_008/
  input.yml
  prompt.txt
  stdout.log
  stderr.log
  events.jsonl
  result.md
  result.json
  file_changes.json
  decision_candidates.yml
  context_update.yml
```

文件说明：

| 文件                    | 作用                     |
| ----------------------- | ------------------------ |
| input.yml               | 本轮真实结构化输入       |
| prompt.txt              | 本轮启动指令             |
| stdout.log              | Runtime 标准输出日志     |
| stderr.log              | Runtime 错误输出日志     |
| events.jsonl            | ArchBot 标准事件流       |
| result.md               | 最终自然语言结果         |
| result.json             | 最终结构化结果           |
| file_changes.json       | 文件变更记录             |
| decision_candidates.yml | Runtime 建议沉淀的决策   |
| context_update.yml      | Runtime 建议更新的上下文 |

------

## 十三、input.yml 标准结构

```yaml
schema_version: "agent_turn_input.v1"

session_header:
  session_id: "sess_001"
  title: "ArchBot 与外部 Agent Runtime 的通信机制设计"
  goal: "设计 ArchBot 与 Claude Code、OpenCode、Hermes、OpenClaw 的长期会话通信机制"
  current_state: "已确认采用 ArchBot 会话主控、滑动窗口、结构化文件输入、HTTP + SSE 通信"

current_turn:
  turn_id: "turn_008"
  user_message: "本轮用户输入原文"
  interpreted_intent: "ArchBot 对用户本轮意图的解释"
  expected_output: "本轮期望输出"

recent_messages:
  - role: "user"
    content: "最近用户消息"
  - role: "assistant"
    content: "最近助手消息"

compressed_history:
  summary: >
    对更早历史的压缩摘要。
  important_context:
    - "重要背景 1"
    - "重要背景 2"

decision_log:
  - id: "D001"
    decision: "ArchBot 是会话主控，不依赖 Runtime 保存主会话状态。"
    status: "accepted"
  - id: "D002"
    decision: "Claude Code、OpenCode、Hermes、OpenClaw 都按长期会话型 Runtime 处理。"
    status: "accepted"
  - id: "D003"
    decision: "每一轮调用 Runtime 时，由 ArchBot 重新组装上下文。"
    status: "accepted"
  - id: "D004"
    decision: "采用滑动窗口 + 历史摘要 + 关键决策 + 当前相关上下文。"
    status: "accepted"
  - id: "D005"
    decision: "结构化 input.yml 作为真实输入，prompt.txt 作为启动指令。"
    status: "accepted"
  - id: "D006"
    decision: "ArchBot 与 Adapter 主通信机制采用 HTTP + SSE。"
    status: "accepted"

working_context:
  project_root: "/path/to/project"
  current_focus:
    - "当前关注点 1"
    - "当前关注点 2"
  relevant_files:
    - path: "src-tauri/src/agent_runtime/mod.rs"
      reason: "Agent Runtime 总入口"
  relevant_artifacts:
    - type: "config"
      path: ".archbot/agent.yml"

execution_policy:
  runtime: "claude_code"
  mode: "interactive_turn"
  input_mode: "structured_file_primary"
  output_mode: "sse_stream"
  allow_file_write: true
  allow_shell: false
  timeout_seconds: 1800
  model:
    provider: "custom"
    base_url: "https://your-model-gateway.example.com"
    name: "your-code-model"

output_contract:
  stream:
    enabled: true
    protocol: "sse"
  final_result:
    markdown_file: "result.md"
    json_file: "result.json"
  file_changes:
    enabled: true
    output_file: "file_changes.json"
  decisions:
    enabled: true
    output_file: "decision_candidates.yml"
  context_update:
    enabled: true
    output_file: "context_update.yml"
```

------

## 十四、prompt.txt 标准结构

```text
请读取以下结构化输入文件，并严格根据其中内容执行本轮任务：

{input_file_path}

执行要求：

1. 以 input.yml 作为本轮真实输入。
2. 不要只根据本 prompt 执行。
3. 遵守 input.yml 中的 session_header、current_turn、recent_messages、compressed_history、decision_log、working_context 和 execution_policy。
4. 执行过程通过 Runtime 原生流式机制输出。
5. 最终结果写入 result.md 和 result.json。
6. 如果产生文件变更，请输出 file_changes.json。
7. 如果产生新的设计决策建议，请输出 decision_candidates.yml。
8. 如果需要更新长期上下文，请输出 context_update.yml。
```

------

## 十五、输出机制结论

Runtime 输出分为两类：

```text
流式事件输出
结构化结果文件
```

流式事件用于前端展示和实时反馈。

结构化结果文件用于 ArchBot 落库、审计、回放和长期会话状态更新。

最终输出不能只依赖 stdout/stderr。

stdout/stderr 可以保留为原始日志，但不能作为唯一协议。

Adapter 必须将 Runtime 输出转换为 ArchBot 标准事件。

------

## 十六、标准事件模型

ArchBot 标准事件包括：

```text
session.created
session.closed

turn.started
turn.delta
turn.reasoning_delta
turn.tool_started
turn.tool_delta
turn.tool_finished
turn.file_changed
turn.artifact_generated
turn.warning
turn.error
turn.completed
turn.failed
turn.cancelled
turn.timeout

runtime.started
runtime.health_changed
runtime.capability_changed
runtime.exited
```

每个事件至少包含：

```json
{
  "event_id": "evt_001",
  "session_id": "sess_001",
  "turn_id": "turn_008",
  "runtime": "claude_code",
  "event_type": "turn.delta",
  "timestamp": "2026-05-30T00:00:00Z",
  "payload": {}
}
```

------

## 十七、Claude Code 部署结论

Claude Code 采用唯一部署方式：

```text
ArchBot 托管安装与版本管理
```

Claude Code 不要求用户手工安装。

Claude Code 不直接打进 Tauri 主二进制。

Claude Code 由 ArchBot Runtime Manager 检测、安装、配置、启动、升级和回滚。

推荐目录：

```text
~/.archbot/runtimes/claude-code/
  versions/
    2.1.100/
    2.1.128/
  current -> versions/2.1.128
  config/
  logs/
```

ArchBot 启动时检测：

```text
~/.archbot/runtimes/claude-code/current/claude --version
```

Claude Code 不使用系统全局版本作为默认 Runtime。

Claude Code 执行不依赖用户全局环境变量。

Claude Code Runtime 与 ArchBot 主程序解耦。

------

## 十八、OpenCode 部署结论

OpenCode 采用：

```text
ArchBot 托管安装与版本管理
```

推荐目录：

```text
~/.archbot/runtimes/opencode/
  versions/
    0.x.x/
  current -> versions/0.x.x
  config/
  logs/
```

OpenCode 由对应 Adapter 统一接入。

OpenCode 优先通过 SDK 或 API 接入。

如 SDK/API 不满足要求，再降级 CLI fallback。

OpenCode 不直接暴露给 Vue 前端。

------

## 十九、Hermes 部署结论

Hermes 采用：

```text
ArchBot 托管隔离运行时
```

Hermes 不打进 Tauri 主二进制。

Hermes 不要求用户手工安装。

Hermes 由 ArchBot Runtime Manager 安装、初始化、启动、停止、升级和回滚。

推荐目录：

```text
~/.archbot/runtimes/hermes/
  versions/
    0.8.0/
    0.8.1/
  current -> versions/0.8.0
  venv/
  config/
  plugins/
  memory/
  logs/
```

Hermes 如依赖 Python、uv、插件、模型配置或外部工具，必须安装在 ArchBot 管理的隔离目录中。

禁止污染用户全局环境：

```text
不修改用户全局 Python
不修改用户全局 pip
不修改用户全局 uv
不写入系统级配置
不依赖用户 shell profile
不依赖用户手工配置 PATH
```

------

## 二十、OpenClaw 部署结论

OpenClaw 采用：

```text
ArchBot 托管隔离运行时
```

推荐目录：

```text
~/.archbot/runtimes/openclaw/
  versions/
    0.x.x/
  current -> versions/0.x.x
  config/
  plugins/
  memory/
  logs/
```

OpenClaw 由对应 Adapter 统一接入。

OpenClaw 优先通过 SDK 或 API 接入。

如 SDK/API 不满足要求，再降级 CLI fallback。

OpenClaw 不直接访问 ArchBot 内部数据。

------

## 二十一、打包部署最终结论

ArchBot 安装包只包含：

1. Tauri 主程序。
2. Vue 前端静态资源。
3. Rust 后端。
4. Axum HTTP 服务。
5. Agent Runtime Manager。
6. Agent Adapter Manager。
7. Runtime 安装器。
8. Runtime 版本管理器。
9. Runtime 检测器。
10. 权限与审计模块。

Claude Code、OpenCode、Hermes、OpenClaw 不直接打进主二进制。

它们作为 ArchBot 管理的外部 Runtime 安装到用户目录。

最终部署结构：

```text
ArchBot.app / ArchBot.exe
  └── 主程序

~/.archbot/
  ├── runtimes/
  │     ├── claude-code/
  │     ├── opencode/
  │     ├── hermes/
  │     └── openclaw/
  ├── adapters/
  │     ├── claude-code/
  │     ├── opencode/
  │     ├── hermes/
  │     └── openclaw/
  ├── config/
  ├── cache/
  └── logs/

项目目录/
  └── .archbot/
        ├── db/
        ├── agent/
        ├── context/
        └── logs/
```

------

## 二十二、运行时配置结论

ArchBot 必须支持 Claude Code、OpenCode、Hermes、OpenClaw 的扩展运行时配置。

运行时配置不能只包含可执行文件路径和版本号。

必须包含：

1. Runtime 类型。
2. Runtime 版本。
3. Adapter 配置。
4. 模型供应商。
5. 模型地址。
6. 模型名称。
7. 认证方式。
8. 环境变量。
9. 启动参数。
10. 超时配置。
11. 并发配置。
12. 代理配置。
13. 自定义 CA 证书。
14. 日志配置。
15. 扩展配置。

------

## 二十三、全局运行时配置

全局配置文件：

```text
~/.archbot/config/runtimes.yml
```

推荐结构：

```yaml
runtimes:
  claude_code:
    enabled: true
    mode: managed
    current_version: "2.1.128"
    executable: "~/.archbot/runtimes/claude-code/current/claude"
    adapter:
      enabled: true
      executable: "~/.archbot/adapters/claude-code/current/archbot-adapter-claude-code"
      protocol: "http_sse"
      host: "127.0.0.1"
      port: "auto"
      auth: "startup_token"

    provider:
      type: custom
      name: "company-claude-compatible"
      protocol: anthropic-compatible
      base_url: "https://your-model-gateway.example.com"
      auth:
        type: bearer_token
        token_ref: "secret://claude_code/api_token"

    model:
      default: "your-code-model"
      small: "your-small-model"
      large: "your-large-model"
      reasoning: "your-reasoning-model"

    env:
      ANTHROPIC_BASE_URL: "https://your-model-gateway.example.com"
      ANTHROPIC_AUTH_TOKEN: "${secret:claude_code/api_token}"
      ANTHROPIC_MODEL: "your-code-model"
      CLAUDE_CONFIG_DIR: "~/.archbot/runtimes/claude-code/current/config"
      DISABLE_AUTOUPDATER: "1"

    args:
      default:
        - "--model"
        - "your-code-model"

    execution:
      default_timeout_seconds: 1800
      max_concurrent_turns: 2
      working_dir_policy: project_root
      inherit_system_env: false

    stream:
      protocol: "sse"
      persist_events: true

    upgrade:
      policy: manual
      allow_auto_update: false
      rollback_enabled: true

    extra:
      settings_json_path: "~/.archbot/runtimes/claude-code/current/config/settings.json"
      proxy: null
      ca_cert_path: null


  opencode:
    enabled: true
    mode: managed
    current_version: "0.x.x"
    executable: "~/.archbot/runtimes/opencode/current/opencode"
    adapter:
      enabled: true
      executable: "~/.archbot/adapters/opencode/current/archbot-adapter-opencode"
      protocol: "http_sse"
      host: "127.0.0.1"
      port: "auto"
      auth: "startup_token"

    provider:
      type: custom
      name: "company-openai-compatible"
      protocol: openai-compatible
      base_url: "https://your-model-gateway.example.com/v1"
      auth:
        type: bearer_token
        token_ref: "secret://opencode/api_token"

    model:
      default: "your-code-model"

    execution:
      default_timeout_seconds: 1800
      max_concurrent_turns: 2


  hermes:
    enabled: true
    mode: managed
    current_version: "0.8.0"
    executable: "~/.archbot/runtimes/hermes/current/bin/hermes"
    home: "~/.archbot/runtimes/hermes/current"
    adapter:
      enabled: true
      executable: "~/.archbot/adapters/hermes/current/archbot-adapter-hermes"
      protocol: "http_sse"
      host: "127.0.0.1"
      port: "auto"
      auth: "startup_token"

    provider:
      type: custom
      name: "company-agent-model-gateway"
      protocol: openai-compatible
      base_url: "https://your-model-gateway.example.com/v1"
      auth:
        type: bearer_token
        token_ref: "secret://hermes/api_token"

    model:
      default: "your-agent-model"
      small: "your-small-model"
      large: "your-large-model"
      reasoning: "your-reasoning-model"
      embedding: "your-embedding-model"

    env:
      HERMES_HOME: "~/.archbot/runtimes/hermes/current"
      HERMES_CONFIG_DIR: "~/.archbot/runtimes/hermes/current/config"
      HERMES_MODEL_BASE_URL: "https://your-model-gateway.example.com/v1"
      HERMES_MODEL_NAME: "your-agent-model"
      HERMES_API_KEY: "${secret:hermes/api_token}"

    execution:
      default_timeout_seconds: 1800
      max_concurrent_turns: 2

    extra:
      plugin_enabled: true
      memory_enabled: true
      proxy: null
      ca_cert_path: null


  openclaw:
    enabled: true
    mode: managed
    current_version: "0.x.x"
    executable: "~/.archbot/runtimes/openclaw/current/openclaw"
    adapter:
      enabled: true
      executable: "~/.archbot/adapters/openclaw/current/archbot-adapter-openclaw"
      protocol: "http_sse"
      host: "127.0.0.1"
      port: "auto"
      auth: "startup_token"

    provider:
      type: custom
      protocol: openai-compatible
      base_url: "https://your-model-gateway.example.com/v1"
      auth:
        type: bearer_token
        token_ref: "secret://openclaw/api_token"

    model:
      default: "your-agent-model"

    execution:
      default_timeout_seconds: 1800
      max_concurrent_turns: 2
```

------

## 二十四、项目级 Agent 配置

项目级配置文件：

```text
项目目录/.archbot/agent.yml
```

推荐结构：

```yaml
agent:
  default_code_runtime: claude_code
  default_task_runtime: hermes

  claude_code:
    provider_override:
      enabled: true
      base_url: "https://project-model-gateway.example.com"
      model: "project-code-model"

    env_override:
      ANTHROPIC_BASE_URL: "https://project-model-gateway.example.com"
      ANTHROPIC_MODEL: "project-code-model"

    args_override:
      - "--model"
      - "project-code-model"

    turn:
      timeout_seconds: 1800
      max_context_files: 200
      max_output_bytes: 10485760

  opencode:
    provider_override:
      enabled: false
      base_url: null
      model: null

    turn:
      timeout_seconds: 1800

  hermes:
    provider_override:
      enabled: true
      base_url: "https://project-agent-gateway.example.com/v1"
      model: "project-agent-model"

    env_override:
      HERMES_MODEL_BASE_URL: "https://project-agent-gateway.example.com/v1"
      HERMES_MODEL_NAME: "project-agent-model"

    turn:
      timeout_seconds: 1800
      max_context_files: 200
      max_output_bytes: 10485760

  openclaw:
    provider_override:
      enabled: false
      base_url: null
      model: null

    turn:
      timeout_seconds: 1800

  sandbox:
    project_root_only: true
    allow_agent_write_in_project: true
    allow_agent_shell_in_project: false

  stream:
    protocol: sse
    persist_events: true

  logs:
    persist_stdout: true
    persist_stderr: true
    persist_events: true
    persist_file_diff: true
```

------

## 二十五、配置优先级

配置优先级从高到低：

```text
任务级参数
项目级 .archbot/agent.yml
全局 ~/.archbot/config/runtimes.yml
ArchBot 默认配置
```

任务级参数优先级最高。

ArchBot 默认配置优先级最低。

------

## 二十六、密钥配置结论

API Key、Token、Auth Token 不应明文写入项目配置。

推荐写法：

```yaml
token_ref: "secret://claude_code/api_token"
```

或：

```yaml
ANTHROPIC_AUTH_TOKEN: "${secret:claude_code/api_token}"
```

密钥应由 ArchBot Secret Manager 管理。

密钥不得写入：

```text
项目目录
Git 仓库
日志文件
stdout
stderr
events.jsonl
result.md
result.json
```

------

## 二十七、写文件权限结论

ArchBot 不对 Claude Code、OpenCode、Hermes、OpenClaw 的内部写文件行为做逐次强制拦截。

Runtime 在执行任务时，可以按照自身能力在 ArchBot 指定的工作目录内读写文件。

ArchBot 不要求 Runtime 在每次写文件前反向调用 ArchBot。

ArchBot 不要求 Runtime 在每次修改文件前等待 ArchBot 审批。

写文件控制采用：

```text
边界控制 + 快照 + diff + 审计 + 回滚
```

而不是：

```text
逐文件审批 + 逐次拦截 + Runtime 反向调用 ArchBot
```

------

## 二十八、写文件边界

Runtime 的默认工作目录必须由 ArchBot 指定。

默认工作目录为：

```text
项目根目录
```

任务级临时目录为：

```text
项目目录/.archbot/agent/sessions/{session_id}/turns/{turn_id}/
```

Runtime 可以在项目工作目录内执行读写操作。

Runtime 不应默认访问项目目录之外的任意路径。

------

## 二十九、任务前快照

每次 Turn 启动前，ArchBot 应记录当前项目状态。

快照内容包括：

```text
当前 Git commit
当前 Git branch
工作区 dirty 状态
关键文件 hash
任务启动时间
任务执行目录
Runtime 类型
Runtime 版本
模型配置
```

如果项目不是 Git 仓库，ArchBot 应对关键文件生成 hash 快照。

------

## 三十、任务后 diff

每次 Turn 结束后，ArchBot 应扫描文件变更。

变更内容包括：

```text
新增文件
修改文件
删除文件
重命名文件
文件 diff
变更时间
变更大小
所属 session_id
所属 turn_id
所属 runtime
```

ArchBot 前端应展示本轮产生的文件变更列表。

------

## 三十一、回滚机制

如果项目是 Git 仓库，优先使用 Git diff 和 Git checkout/revert 能力进行回滚。

如果项目不是 Git 仓库，ArchBot 应基于任务前快照和关键文件备份进行回滚。

回滚粒度至少支持：

```text
按 Turn 回滚
按 Session 回滚
按文件回滚
按 diff 块人工回滚
```

------

## 三十二、禁止访问范围

即使不做逐次写入拦截，Runtime 仍不应默认访问以下路径：

```text
用户主目录
~/.ssh
~/.aws
~/.config
~/.claude
~/.hermes
.env
系统目录
其他项目目录
ArchBot 全局配置目录
ArchBot Runtime 目录
ArchBot Adapter 目录
ArchBot 数据库目录
```

------

## 三十三、Shell 执行权限结论

Runtime 默认不允许执行 shell 命令。

如需执行命令，必须由 ArchBot 显式配置。

允许执行命令的前提：

1. 项目级配置允许。
2. 命令在白名单内。
3. 工作目录限定在项目目录。
4. 有超时时间。
5. 有输出日志。
6. 有审计记录。

默认禁止命令：

```text
rm
sudo
chmod
chown
curl | sh
wget | sh
ssh
scp
rsync
docker system prune
修改系统环境变量
修改 shell profile
```

------

## 三十四、任务与会话数据模型

建议核心表：

```text
agent_runtime
agent_runtime_version
agent_adapter
agent_session
agent_turn
agent_event
agent_artifact
agent_file_change
agent_decision
agent_context_snapshot
agent_audit_log
```

用途：

| 表                     | 用途                                                 |
| ---------------------- | ---------------------------------------------------- |
| agent_runtime          | 记录 Claude Code、OpenCode、Hermes、OpenClaw Runtime |
| agent_runtime_version  | 记录 Runtime 版本                                    |
| agent_adapter          | 记录 Adapter 进程与端口                              |
| agent_session          | 记录长期会话                                         |
| agent_turn             | 记录每轮交互                                         |
| agent_event            | 记录标准事件                                         |
| agent_artifact         | 记录输出产物                                         |
| agent_file_change      | 记录文件变更                                         |
| agent_decision         | 记录关键决策                                         |
| agent_context_snapshot | 记录上下文快照                                       |
| agent_audit_log        | 记录审计日志                                         |

------

## 三十五、AgentSession 模型

```text
AgentSession
  session_id
  title
  goal
  project_id
  runtime_type
  default_model
  current_state
  status
  created_at
  updated_at
```

Session 状态：

```text
active
paused
closed
archived
```

------

## 三十六、AgentTurn 模型

```text
AgentTurn
  turn_id
  session_id
  user_message
  interpreted_intent
  input_file
  prompt_file
  status
  runtime_type
  runtime_version
  model
  started_at
  finished_at
  error_message
```

Turn 状态：

```text
pending
running
streaming
completed
failed
cancelled
timeout
```

------

## 三十七、版本管理结论

Claude Code、OpenCode、Hermes、OpenClaw 都必须由 ArchBot 锁定版本。

不允许默认使用系统全局最新版。

不允许静默自动升级。

版本策略：

```text
企业交付：固定版本
个人使用：手动升级
开发环境：可使用最新版本
```

默认策略：

```text
manual
```

即：用户确认后才升级。

------

## 三十八、升级与回滚结论

Runtime 升级必须支持回滚。

升级流程：

```text
下载新版本
校验完整性
安装到新版本目录
执行版本检测
执行最小可用性测试
切换 current 指针
记录升级日志
```

失败处理：

```text
保留旧版本
不切换 current
标记新版本不可用
记录失败原因
提示用户
```

回滚方式：

```text
current -> previous_version
```

------

## 三十九、Adapter 与 Runtime 集成策略

Adapter 内部接入 Runtime 的优先级：

```text
SDK 优先
API 次之
CLI fallback 最后
```

即：

```text
Adapter -> Runtime SDK
Adapter -> Runtime Local API
Adapter -> Runtime CLI
```

不同 Runtime 由不同 Adapter 处理。

ArchBot 主进程不直接依赖各 Runtime SDK。

ArchBot 主进程只依赖统一的 Adapter HTTP + SSE 协议。

------

## 四十、前端展示结论

前端只展示 ArchBot 抽象后的信息。

前端不展示 Runtime 内部协议细节。

前端模块包括：

```text
AgentRuntimePanel
AgentAdapterPanel
AgentSessionPanel
AgentTurnPanel
AgentEventStreamPanel
AgentDiffReviewPanel
AgentVersionPanel
AgentAuditLogPanel
AgentContextPanel
AgentDecisionPanel
```

前端能力包括：

1. 查看 Runtime 状态。
2. 查看 Adapter 状态。
3. 查看 Runtime 版本。
4. 创建长期会话。
5. 执行当前 Turn。
6. 查看流式输出。
7. 取消当前 Turn。
8. 查看文件变更。
9. 查看 diff。
10. 查看历史会话。
11. 查看关键决策。
12. 查看上下文摘要。
13. 查看审计日志。
14. 管理模型配置。
15. 管理 Runtime 升级和回滚。

------

## 四十一、最终决策表

| 决策项                            | 结论                                   |
| --------------------------------- | -------------------------------------- |
| ArchBot 定位                      | 本地 Agent 控制平面                    |
| Claude Code 定位                  | 长期会话型代码 Agent Runtime           |
| OpenCode 定位                     | 长期会话型代码 Agent Runtime           |
| Hermes 定位                       | 长期会话型泛任务 Agent Runtime         |
| OpenClaw 定位                     | 长期会话型泛任务 Agent Runtime         |
| 会话主控                          | ArchBot                                |
| Runtime 职责                      | 每轮 Turn 执行                         |
| Adapter 职责                      | 协议转换、SDK/API/CLI 封装、事件标准化 |
| 主通信机制                        | HTTP + SSE                             |
| 命令通道                          | HTTP                                   |
| 流式通道                          | SSE                                    |
| JSON-RPC over stdio               | 不作为主方案，仅作为 fallback 可能性   |
| 输入方式                          | input.yml 为主，prompt.txt 为辅        |
| 输出方式                          | SSE 事件流 + 结构化结果文件            |
| Claude Code 部署                  | ArchBot 托管安装与版本管理             |
| OpenCode 部署                     | ArchBot 托管安装与版本管理             |
| Hermes 部署                       | ArchBot 托管隔离运行时                 |
| OpenClaw 部署                     | ArchBot 托管隔离运行时                 |
| 是否打进 Tauri 主二进制           | 否                                     |
| 是否要求用户手工安装 Runtime      | 否                                     |
| 是否使用系统全局 Runtime          | 否                                     |
| 是否允许 Runtime 反向调用 ArchBot | 否                                     |
| 是否默认允许写文件                | 允许在项目工作目录内写                 |
| 写文件控制方式                    | 边界控制 + 快照 + diff + 审计 + 回滚   |
| 是否默认允许 shell                | 否                                     |
| 是否支持自定义模型地址            | 是                                     |
| 是否支持自定义模型名称            | 是                                     |
| 是否支持企业模型网关              | 是                                     |
| 是否支持版本锁定                  | 是                                     |
| 是否支持升级回滚                  | 是                                     |
| 是否记录审计日志                  | 是                                     |
| 数据沉淀位置                      | ArchBot SQLite / 文件日志              |
| 上下文沉淀位置                    | ArchBot LanceDB / 项目目录             |
| 权限控制位置                      | ArchBot                                |
| 任务控制位置                      | ArchBot                                |
| Runtime 管理位置                  | ArchBot                                |

------

## 四十二、最终一句话结论

ArchBot 应作为长期会话型 Agent 控制平面，统一管理 Claude Code、OpenCode、Hermes、OpenClaw 等外部 Runtime。

ArchBot 保存会话主状态，Runtime 只负责每一轮执行。

ArchBot 与 Adapter 之间采用 HTTP + SSE 作为主通信机制。

每一轮执行采用 input.yml 作为真实结构化输入，prompt.txt 作为启动指令。

Runtime 的输出通过 SSE 转换为 ArchBot 标准事件，同时生成结构化结果文件，用于审计、回放、上下文更新和长期会话沉淀。

Claude Code、OpenCode、Hermes、OpenClaw 都由 ArchBot 托管安装、配置、启动、升级和回滚。

ArchBot 不硬嵌 Runtime，不依赖 Runtime 保存主会话，不允许 Runtime 反向调用 ArchBot 内部接口。

最终形成：

```text
ArchBot 控制会话
Adapter 适配协议
Runtime 执行任务
HTTP 传命令
SSE 传事件
input.yml 传上下文
结构化文件沉淀结果
SQLite/LanceDB 保存长期状态
```