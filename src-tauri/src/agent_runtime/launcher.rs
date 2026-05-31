use std::io::Write;
use std::process::{Child, Command, Stdio};

use crate::agent_runtime::config::RuntimeLaunchConfig;

pub type LaunchResult = Result<Child, String>;

/// 启动一个被完全隔离的 Runtime 子进程。
///
/// - 清空所有继承的环境变量
/// - 将 `HOME` 重写为 ArchBot 管理的隔离目录
/// - 只注入白名单中的环境变量
/// - 注入最小系统环境（PATH/LANG/TZ）
/// - Windows 上额外注入 SystemRoot/TEMP/TMP
/// - 如果 config.stdin_content 非空，通过管道写入子进程 stdin
pub fn launch_isolated_runtime(config: &RuntimeLaunchConfig) -> LaunchResult {
    if !std::path::Path::new(&config.executable).exists() {
        return Err(format!(
            "[launcher] Executable not found: {}",
            config.executable
        ));
    }

    let mut cmd = Command::new(&config.executable);

    // 1. 清空所有继承的环境变量
    cmd.env_clear();

    // 2. 注入隔离 HOME — Runtime 内所有 ~/ 路径解析到这里
    cmd.env("HOME", &config.isolated_home);

    // 3. 只注入白名单环境变量
    for (key, value) in &config.allowed_env {
        cmd.env(key, value);
    }

    // 4. 最小系统环境（不含用户 shell profile）
    #[cfg(target_os = "macos")]
    cmd.env("PATH", "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin");
    #[cfg(not(target_os = "macos"))]
    cmd.env("PATH", "/usr/local/bin:/usr/bin:/bin");
    cmd.env("LANG", "en_US.UTF-8");
    cmd.env("TZ", "UTC");

    // 5. Windows 关键系统变量
    #[cfg(target_os = "windows")]
    {
        cmd.env("SystemRoot", "C:\\Windows");
        cmd.env("TEMP", &config.isolated_home);
        cmd.env("TMP", &config.isolated_home);
    }

    // 6. 设置工作目录
    cmd.current_dir(&config.workspace_root);

    // 7. 传递启动参数
    cmd.args(&config.args);

    // 8. 如果需要写入 stdin，配置管道
    if config.stdin_content.is_some() {
        cmd.stdin(Stdio::piped());
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("[launcher] Failed to spawn {}: {}", config.runtime_type, e))?;

    // 写入 stdin 内容后关闭管道（EOF 信号）
    if let Some(ref content) = config.stdin_content {
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(content.as_bytes())
                .map_err(|e| format!("[launcher] Failed to write to stdin: {}", e))?;
            // stdin 在此处 drop，管道关闭，子进程收到 EOF
        }
    }

    Ok(child)
}
