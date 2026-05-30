# launcher.rs — 组件设计

## 职责

启动一个被完全隔离的 Runtime 子进程。调用方（Tauri command handler）只需传入 `RuntimeLaunchConfig`，得到一个 `std::process::Child`。

## 接口

```rust
pub fn launch_isolated_runtime(config: &RuntimeLaunchConfig) -> Result<Child, LaunchError>
```

## 输入

```rust
pub struct RuntimeLaunchConfig {
    pub runtime_type: String,         // "claude_code" | "hermes" | "opencode" | "openclaw"
    pub executable: String,           // ~/.archbot/runtimes/claude-code/current/claude
    pub workspace_root: String,       // /path/to/project
    pub isolated_home: String,        // ~/.archbot/runtimes/claude-code/home
    pub allowed_env: HashMap<String, String>,  // 白名单环境变量
    pub args: Vec<String>,            // ["--input-file", "input.yml", "--json-output"]
    pub timeout_seconds: u64,
}
```

## 执行流程

```
launch_isolated_runtime(config)
  │
  ├─ 1. 构造 std::process::Command
  │
  ├─ 2. cmd.env_clear()
  │    清空所有从 ArchBot 进程继承的环境变量
  │
  ├─ 3. cmd.env("HOME", config.isolated_home)
  │    重写 HOME 为隔离目录（最关键的一步）
  │
  ├─ 4. 注入白名单环境变量
  │    for (k, v) in config.allowed_env { cmd.env(k, v); }
  │
  ├─ 5. 注入最小系统环境
  │    PATH, LANG, TZ
  │    不注入: USER, LOGNAME, SHELL, SSH_AUTH_SOCK, TMPDIR
  │
  ├─ 6. [cfg(windows)] 注入 Windows 关键变量
  │    SystemRoot, TEMP, TMP
  │
  ├─ 7. cmd.current_dir(&config.workspace_root)
  │
  ├─ 8. cmd.args(&config.args)
  │
  └─ 9. cmd.spawn() → Result<Child>
```

## 错误类型

```rust
#[derive(Debug, thiserror::Error)]
pub enum LaunchError {
    #[error("executable not found: {0}")]
    ExecutableNotFound(String),
    #[error("failed to spawn process: {0}")]
    SpawnFailed(#[from] std::io::Error),
    #[error("isolated home not initialized: {0}")]
    HomeNotReady(String),
}
```

## 平台差异

| 操作 | macOS/Linux | Windows |
|------|------------|---------|
| env_clear | ✅ | ✅ |
| HOME | `$HOME` | `%HOME%`（作用相同） |
| PATH | `/usr/local/bin:/usr/bin:/bin` | `C:\Windows\System32;C:\Windows;C:\Windows\System32\Wbem` |
| 额外变量 | — | `SystemRoot`, `TEMP`, `TMP` |
| executable 路径分隔符 | `/` | `\`（Rust `Command` 自动处理） |

## 安全约束

- `executable` 必须指向 `~/.archbot/runtimes/` 下的路径（调用方校验）
- 白名单外的环境变量绝不注入
- `SSH_AUTH_SOCK`、`GPG_AGENT_INFO` 等凭据类变量绝不注入
- 用户真实 HOME 绝不暴露
