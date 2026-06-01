# BUG-001: Tokio `Handle::current()` Panic in SessionManager

<!--
  Bug Report Template — 所有缺陷均按此结构记录
  Status: open | in_progress | fixed | verified | wont_fix
  Severity: critical | high | medium | low
-->

| Field       | Value                                        |
|-------------|----------------------------------------------|
| **Bug ID**  | BUG-001                                      |
| **Title**   | `tokio::runtime::Handle::current()` panic on main thread in SessionManager |
| **Status**  | open                                         |
| **Severity**| critical (进程崩溃)                            |
| **Found**   | 2026-05-31 21:17 CST                         |
| **Module**  | `src-tauri/src/agent_runtime/session_manager.rs` |
| **Author**  | system                                       |

---

## 1. 现象 (Symptom)

ArchBot 进程从 RustRover IDE 启动约 2 小时后，前端调用 `agent_list_sessions` 命令时，进程突然崩溃退出。

macOS Crash Reporter 输出：

```
Process:               archbot [62711]
Exception Type:        EXC_CRASH (SIGABRT)
Termination Reason:    Namespace SIGNAL, Code 6 Abort trap: 6
Application Specific Information: abort() called
```

## 2. 环境 (Environment)

| 项目          | 值                                         |
|---------------|--------------------------------------------|
| OS            | macOS 15.6.1 (24G90)                       |
| 架构          | ARM-64 (Apple M1 Max)                      |
| 父进程         | node (RustRover 插件)                       |
| ArchBot 版本   | 0.1.0                                      |
| Rust 工具链    | (需确认)                                     |
| 系统完整性保护  | disabled                                   |

## 3. 错误日志 (Crash Stack Trace — 关键帧)

```
Thread 0 Crashed:: main  Dispatch queue: com.apple.main-thread

12  archbot  tokio::runtime::scheduler::Handle::current        + 72   ← PANIC HERE
13  archbot  tokio::runtime::handle::Handle::current           + 12
14  archbot  SessionManager::list_all                          + 32
15  archbot  agent_list_sessions                              + 32
16  archbot  archbot_lib::run::closure                         + 128
17  archbot  archbot_lib::run::closure                         + 12140
...
26  archbot  wry::wkwebview::class::url_scheme_handler::start_task + 4524
27  WebKit   WebURLSchemeHandlerCocoa::platformStartTask        + 112
...
49  AppKit   -[NSApplication run]                              + 480
```

## 4. 根因分析 (Root Cause Analysis)

### 4.1 直接原因

`SessionManager::list_all()` (L170) 调用了 `tokio::runtime::Handle::current()`，该函数在当前 OS 线程没有进入 Tokio 运行时上下文时 **panic**。

```rust
// session_manager.rs:169-170
pub fn list_all(&self) -> Result<Vec<AgentSession>, String> {
    let rt = tokio::runtime::Handle::current();  // ← PANIC on main thread
    rt.block_on(async { /* DB query */ })
}
```

### 4.2 触发条件

Tauri 2 的 IPC 命令在主线程（macOS CFRunLoop）上执行，而 **主线程没有进入 Tokio 运行时上下文**。当 WebKit 发起自定义协议请求（wry URL scheme handler → Tauri IPC）时，调用链完全在主线程上：

```
用户点击前端按钮
  → WebKit custom protocol request
    → wry url_scheme_handler::start_task (主线程 frame 26)
      → Tauri IPC protocol handler (主线程 frame 21)
        → tauri::AppManager::run_invoke_handler (主线程 frame 19)
          → agent_list_sessions 命令 (主线程 frame 15)
            → SessionManager::list_all() (主线程 frame 14)
              → Handle::current() → PANIC (主线程 frame 12)
```

### 4.3 影响范围

`SessionManager` 中**所有**使用 `Handle::current()` 的方法均有同样的问题（共 10 处）：

| 方法                   | 行号  | 影响                              |
|------------------------|------|-----------------------------------|
| `list_all()`           | L170 | 列出 session 崩溃                  |
| `get()`                | L106 | 获取单个 session 崩溃              |
| `update_status()`      | L254 | 更新状态崩溃                       |
| `list_turns()`         | L342 | 列出 turns 崩溃                    |
| `get_turn()`           | L426 | 获取单个 turn 崩溃                 |
| `get_file_changes()`   | L509 | 查询文件变更崩溃                   |
| `get_audit_log()`      | L545 | 审计日志查询崩溃                   |
| `save_turn_start()`    | L606 | 保存 turn 开始记录崩溃             |
| `save_turn_finish()`   | L640 | 保存 turn 完成记录崩溃             |
| `save_file_change()`   | L661 | 保存文件变更记录崩溃               |

### 4.4 为什么之前没有触发？

1. 这些 Tauri 命令可能在此次会话中是**首次被实际调用**（之前主要在开发/测试，未被前端触发）
2. 崩溃需要前端在应用运行期间触发 `agent_list_sessions` 命令

## 5. 修复方案 (Fix Plan)

### 方案：`OnceLock` 静态捕获 Tokio Handle

在启动时（Tokio 运行时已初始化且有上下文）将 Handle 捕获到全局静态变量中，所有 `SessionManager` 方法使用该静态 Handle 替代 `Handle::current()`。

**修改文件**:

1. **`src-tauri/src/agent_runtime/session_manager.rs`**
   - 新增全局 `TOKIO_HANDLE: OnceLock<tokio::runtime::Handle>`
   - 新增 `pub fn init_tokio_handle()` 初始化函数
   - 新增 `fn rt_handle()` 私有辅助函数，优先返回静态 Handle，回退到 `Handle::current()`
   - 将所有 10 处 `tokio::runtime::Handle::current()` 替换为 `rt_handle()`

2. **`src-tauri/src/lib.rs`**
   - 在 `run()` 函数中、`tauri::Builder` 构建之前调用 `agent_runtime::session_manager::init_tokio_handle()`

示例代码：

```rust
use std::sync::OnceLock;

static TOKIO_HANDLE: OnceLock<tokio::runtime::Handle> = OnceLock::new();

/// 在启动时调用一次，捕获当前 Tokio 运行时 Handle。
/// 必须在 Tokio 上下文中调用（例如在 tauri::Builder::run 之前）。
pub fn init_tokio_handle() {
    match tokio::runtime::Handle::try_current() {
        Ok(h) => { TOKIO_HANDLE.set(h).ok(); }
        Err(e) => {
            tracing::warn!("无法捕获 Tokio handle: {e}，将退回到 Handle::current()");
        }
    }
}

fn rt_handle() -> tokio::runtime::Handle {
    TOKIO_HANDLE.get()
        .cloned()
        .unwrap_or_else(|| tokio::runtime::Handle::current())
}
```

## 6. 修复历史 (Fix History)

| Date       | Action                                  | By     |
|------------|-----------------------------------------|--------|
| 2026-05-31 | Bug 发现并记录                           | system |
| -          | 待修复                                   | -      |

---

*本报告按照 `bugs/README.md` 模板规范编写。*
