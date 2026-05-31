use crate::trace_fmt;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::Instant;

use crate::agent_runtime::audit::AuditManager;
use crate::agent_runtime::event_stream::{EventBus, StandardEvent};
use crate::agent_runtime::file_control::{self, PreTurnSnapshot};
use crate::agent_runtime::home_setup::setup_isolated_home;
use crate::agent_runtime::launcher::launch_isolated_runtime;
use crate::agent_runtime::runtime_config::{
    build_home_config, build_launch_config, load_runtimes_config,
};
use crate::agent_runtime::turn_config::{FileChange, TurnConfig, TurnResult};
use crate::secret::SecretManager;

/// Tauri command: 前端调用的入口 — async to avoid blocking the IPC thread.
#[tauri::command]
pub async fn agent_execute_turn(
    runtime: String,
    workspace_root: String,
    user_message: String,
    context_files: Vec<String>,
    model_override: Option<String>,
) -> Result<TurnResult, String> {
    let config = TurnConfig {
        runtime,
        workspace_root,
        user_message,
        context_files,
        git_user_name: None,
        git_user_email: None,
        session_id: None,
        model_override,
    };
    tokio::task::spawn_blocking(move || execute_turn(config))
        .await
        .map_err(|e| format!("spawn_blocking error: {}", e))?
}

/// 执行一次完整的 Agent Turn——串联 launcher / home_setup / audit / secret。
pub fn execute_turn(config: TurnConfig) -> Result<TurnResult, String> {
    let start = Instant::now();
    let turn_id = uuid::Uuid::new_v4().to_string();
    trace_fmt!("turn", "START turn_id={} runtime={} workspace={}", turn_id, config.runtime, config.workspace_root);

    // 1. 加载 Runtime 配置
    let rt_config = load_runtimes_config()?;
    let entry = rt_config
        .runtimes
        .get(&config.runtime)
        .ok_or_else(|| format!("Runtime not found in config: {}", config.runtime))?;

    if !entry.enabled {
        trace_fmt!("turn", "FAIL turn_id={} — runtime disabled: {}", turn_id, config.runtime);
        return Err(format!("Runtime disabled: {}", config.runtime));
    }
    trace_fmt!("turn", "Runtime config loaded — executable={}", entry.executable);

    // 2. 获取 API token + 构造 launch config
    let mut launch_config = build_launch_config(
        &config.runtime,
        entry,
        &config.workspace_root,
        config.git_user_name.as_deref(),
        config.git_user_email.as_deref(),
    )?;

    // ── Token resolution ──
    // Look up API token from the configured AI provider via SecretManager.
    // Falls back to env vars from runtimes.yml (already in launch_config.allowed_env
    // from build_launch_config) if SecretManager doesn't have the token.
    if let Ok(sm) = SecretManager::new(&get_machine_id()) {
        let provider_id = entry.provider_id.as_deref().unwrap_or(&config.runtime);
        let token_key = match config.runtime.as_str() {
            "claude_code" => "ANTHROPIC_AUTH_TOKEN",
            "hermes" => "HERMES_API_KEY",
            _ => "OPENAI_API_KEY",
        };
        trace_fmt!("turn", "Token lookup: provider_id={} runtime={} token_key={}", provider_id, config.runtime, token_key);

        // Try provider_id first (e.g. "deepseek"), then runtime name (e.g. "claude_code")
        for lookup in &[provider_id, config.runtime.as_str()] {
            if launch_config.allowed_env.contains_key(token_key) && !launch_config.allowed_env[token_key].is_empty() {
                break; // Already have a token from runtimes.yml env vars
            }
            match sm.get(lookup, "api_token") {
                Ok(token) if !token.is_empty() => {
                    trace_fmt!("turn", "Token resolved via provider={}", lookup);
                    launch_config.allowed_env.insert(token_key.to_string(), token);
                    break;
                }
                Ok(_) => { trace_fmt!("turn", "Token for provider={} is empty", lookup); }
                Err(e) => { trace_fmt!("turn", "No token for provider={}: {}", lookup, e); }
            }
        }
    }

    // Fallback: parent process environment
    for key in &["ANTHROPIC_AUTH_TOKEN", "ANTHROPIC_BASE_URL", "ANTHROPIC_MODEL",
                  "OPENAI_API_KEY", "OPENAI_BASE_URL", "OPENAI_MODEL"] {
        if !launch_config.allowed_env.contains_key(*key) {
            if let Ok(val) = std::env::var(key) {
                trace_fmt!("turn", "Injecting {} from parent env", key);
                launch_config.allowed_env.insert(key.to_string(), val);
            }
        }
    }

    // Derive ANTHROPIC_BASE_URL from AI provider config — always override the default.
    // Claude Code needs Anthropic-format URL. For DeepSeek:
    //   provider base_url = https://api.deepseek.com/v1 (OpenAI format)
    //   → ANTHROPIC_BASE_URL = https://api.deepseek.com/anthropic
    if let Ok(providers) = crate::ai_config::load_providers_raw() {
        let provider_id = entry.provider_id.as_deref().unwrap_or(&config.runtime);
        if let Some(p) = providers.iter().find(|p| p.id == provider_id) {
            let anthropic_url = if p.base_url.ends_with("/v1") {
                format!("{}/anthropic", p.base_url.trim_end_matches("/v1"))
            } else if p.base_url.ends_with("/anthropic") {
                p.base_url.clone()
            } else {
                format!("{}/anthropic", p.base_url.trim_end_matches('/'))
            };
            let old = launch_config.allowed_env.insert("ANTHROPIC_BASE_URL".to_string(), anthropic_url.clone());
            if old.as_deref() != Some(&anthropic_url) {
                trace_fmt!("turn", "ANTHROPIC_BASE_URL {}→ {} (from provider {})",
                    old.as_deref().unwrap_or("(not set)"), anthropic_url, provider_id);
            }
        }
    }

    // 模型覆盖：用户在 UI 中选择的模型优先于所有默认值
    if let Some(ref model) = config.model_override {
        let model_key = match config.runtime.as_str() {
            "claude_code" => "ANTHROPIC_MODEL",
            "hermes" => "HERMES_MODEL_NAME",
            _ => "OPENAI_MODEL",
        };
        trace_fmt!("turn", "Model override: {}={}", model_key, model);
        launch_config.allowed_env.insert(model_key.to_string(), model.clone());
    }

    // 3. 初始化隔离 HOME
    let home_config = build_home_config(entry)?;
    setup_isolated_home(&home_config)?;

    // 4. 创建 turn 目录 + 写入 input.yml（审计用，不传给 Runtime CLI）
    let turn_dir = turn_directory(&config.workspace_root, &turn_id)?;
    fs::create_dir_all(&turn_dir)
        .map_err(|e| format!("Failed to create turn dir {:?}: {}", turn_dir, e))?;

    let input_yml = generate_input_yml(&config, &turn_id);
    fs::write(turn_dir.join("input.yml"), &input_yml)
        .map_err(|e| format!("Failed to write input.yml: {}", e))?;
    fs::write(turn_dir.join("prompt.txt"), generate_prompt_txt(&turn_dir))
        .map_err(|e| format!("Failed to write prompt.txt: {}", e))?;

    // 5. 构建自包含 prompt，写入 stdin（不传文件路径 CLI 参数）
    let stdin_prompt = build_stdin_prompt(&config, &input_yml, &turn_dir);
    launch_config.stdin_content = Some(stdin_prompt);

    // 6. 捕获执行前快照
    let project_root = Path::new(&config.workspace_root);
    let snapshot = PreTurnSnapshot::capture(&turn_id, project_root).ok();

    // 7. 发射事件 + 启动子进程
    trace_fmt!("turn", "Launching runtime — executable={} cwd={} stdin_bytes={}", launch_config.executable, launch_config.workspace_root, launch_config.stdin_content.as_ref().map_or(0, |c| c.len()));
    let bus = EventBus::global();
    let session_anchor = config.session_id.clone().unwrap_or_else(|| turn_id.clone());
    bus.publish(StandardEvent::session_created(&session_anchor, &config.runtime));
    bus.publish(StandardEvent::turn_started(&session_anchor, &turn_id, &config.runtime));

    let mut child = launch_isolated_runtime(&launch_config)?;
    trace_fmt!("turn", "Child process spawned — pid={:?}", child.id());

    // 8. 等待进程退出（带超时），同时解析 NDJSON 流并实时发射 SSE 事件
    let timeout = std::time::Duration::from_secs(launch_config.timeout_seconds);
    let (stdout, stderr, status) = wait_with_timeout(
        &mut child,
        timeout,
        &session_anchor,
        &turn_id,
        &config.runtime,
    )?;

    fs::write(turn_dir.join("stdout.log"), &stdout).ok();
    fs::write(turn_dir.join("stderr.log"), &stderr).ok();

    // 9. 解析结果 — 优先读 Runtime 写入的 result.md，否则回退到 stdout
    let result_md_path = turn_dir.join("result.md");
    if !result_md_path.exists() && !stdout.trim().is_empty() {
        fs::write(&result_md_path, &stdout).ok();
    }
    // Read actual result content from result.md
    let result_content = if result_md_path.exists() {
        fs::read_to_string(&result_md_path).unwrap_or_default()
    } else if !stdout.trim().is_empty() {
        stdout.clone()
    } else {
        String::new()
    };
    let result_md = if result_md_path.exists() {
        Some(result_md_path.to_string_lossy().to_string())
    } else {
        None
    };

    // 10. 扫描文件变更（与快照对比）
    let file_changes = match &snapshot {
        Some(snap) => {
            match file_control::scan_file_changes(project_root, snap) {
                Ok(diffs) => diffs
                    .into_iter()
                    .map(|d| FileChange {
                        path: d.path,
                        change_type: d.change_type,
                    })
                    .collect(),
                Err(_) => parse_file_changes(&turn_dir.join("file_changes.json")),
            }
        }
        None => parse_file_changes(&turn_dir.join("file_changes.json")),
    };

    // 对每个文件变更发射事件
    for fc in &file_changes {
        bus.publish(StandardEvent::turn_file_changed(
            &session_anchor, &turn_id, &fc.path, &fc.change_type, &config.runtime,
        ));
    }

    let stdout_tail = if stdout.len() > 2000 {
        format!("...(truncated)\n{}", &stdout[stdout.len() - 2000..])
    } else {
        stdout.clone()
    };

    // 11. 审计
    let audit_manager = AuditManager::new();
    let accessed = collect_accessed_paths(&turn_dir);
    let violations = audit_manager.audit(&accessed);
    let audit_violations: Vec<String> = violations
        .iter()
        .map(|v| format!("{:?}: {}", v.rule.severity, v.accessed_path))
        .collect();

    let duration_ms = start.elapsed().as_millis() as u64;
    let status_str = if status.success() {
        trace_fmt!("turn", "COMPLETED turn_id={} duration={}ms file_changes={}", turn_id, duration_ms, file_changes.len());
        bus.publish(StandardEvent::turn_completed(&session_anchor, &turn_id));
        "completed".to_string()
    } else {
        let err = format!("failed: exit code {:?}", status.code());
        trace_fmt!("turn", "FAILED turn_id={} error={}", turn_id, err);
        bus.publish(StandardEvent::turn_failed(&session_anchor, &turn_id, &err));
        err
    };

    Ok(TurnResult {
        turn_id,
        runtime: config.runtime,
        status: status_str,
        stdout_tail,
        result_md_path: result_md,
        result_content,
        file_changes,
        audit_violations,
        duration_ms,
    })
}

/// 带超时的进程等待 — 边等边读 stdout/stderr，解析 NDJSON 流并实时发射 SSE 事件。
fn wait_with_timeout(
    child: &mut Child,
    timeout: std::time::Duration,
    session_id: &str,
    turn_id: &str,
    runtime: &str,
) -> Result<(String, String, std::process::ExitStatus), String> {
    let mut stdout_buf = Vec::new();
    let mut stderr_buf = Vec::new();
    let mut line_buf = Vec::new();
    let start = Instant::now();
    let bus = EventBus::global();

    let mut out_reader = child.stdout.take();
    let mut err_reader = child.stderr.take();

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                // 进程已退出，读完剩余数据
                if let Some(ref mut r) = out_reader {
                    let _ = r.read_to_end(&mut stdout_buf);
                }
                if let Some(ref mut r) = err_reader {
                    let _ = r.read_to_end(&mut stderr_buf);
                }
                // Parse any remaining lines
                parse_ndjson_events(&stdout_buf, &mut line_buf, session_id, turn_id, runtime, bus);
                let stdout = String::from_utf8_lossy(&stdout_buf).to_string();
                let stderr = String::from_utf8_lossy(&stderr_buf).to_string();
                return Ok((stdout, stderr, status));
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let stdout = String::from_utf8_lossy(&stdout_buf).to_string();
                    bus.publish(StandardEvent::turn_error(
                        session_id, turn_id, "Turn timed out", runtime,
                    ));
                    return Err(format!(
                        "Turn timed out after {:?}. stdout: {}...",
                        timeout,
                        &stdout[..stdout.len().min(200)]
                    ));
                }
                // 增量读取避免管道满，同时解析已到达的 NDJSON 行
                let mut buf = [0u8; 8192];
                let prev_len = stdout_buf.len();
                if let Some(ref mut r) = out_reader {
                    match r.read(&mut buf) {
                        Ok(0) => {} // EOF
                        Ok(n) => stdout_buf.extend_from_slice(&buf[..n]),
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                        Err(_) => {}
                    }
                }
                if let Some(ref mut r) = err_reader {
                    match r.read(&mut buf) {
                        Ok(0) => {}
                        Ok(n) => stderr_buf.extend_from_slice(&buf[..n]),
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                        Err(_) => {}
                    }
                }
                // 仅当有新数据时才解析，减少重复工作
                if stdout_buf.len() > prev_len {
                    parse_ndjson_events(&stdout_buf, &mut line_buf, session_id, turn_id, runtime, bus);
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => return Err(format!("Failed to wait on child process: {}", e)),
        }
    }
}

/// Parse NDJSON lines from the buffered stdout and emit SSE events for each complete line.
///
/// Claude Code `--output-format stream-json` outputs one JSON object per line.
/// We extract text content and tool use events, mapping them to ArchBot standard events.
fn parse_ndjson_events(
    stdout_buf: &[u8],
    line_buf: &mut Vec<u8>,
    session_id: &str,
    turn_id: &str,
    runtime: &str,
    bus: &EventBus,
) {
    let bus_ref = bus; // borrow checker helper
    // Process every byte in the buffer, treating line_buf as carry-over from last call
    for &byte in stdout_buf {
        line_buf.push(byte);
        if byte == b'\n' {
            let line = std::str::from_utf8(line_buf).unwrap_or("");
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(line.trim()) {
                emit_ndjson_event(&parsed, session_id, turn_id, runtime, bus_ref);
            }
            line_buf.clear();
        }
    }
}

fn emit_ndjson_event(
    line: &serde_json::Value,
    session_id: &str,
    turn_id: &str,
    runtime: &str,
    bus: &EventBus,
) {
    let event_type = line.get("type").and_then(|v| v.as_str()).unwrap_or("");

    match event_type {
        // stream_event with text delta → turn.delta
        "stream_event" => {
            if let Some(event) = line.get("event") {
                // Content block delta (text increment)
                if let Some(delta) = event.get("delta") {
                    if let Some(text) = delta.get("text").and_then(|v| v.as_str()) {
                        if !text.is_empty() {
                            bus.publish(StandardEvent::turn_delta(session_id, turn_id, text, runtime));
                        }
                    }
                }
                // Tool use block start
                if let Some(content_block) = event.get("content_block") {
                    if content_block.get("type").and_then(|v| v.as_str()) == Some("tool_use") {
                        let tool_name = content_block
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        bus.publish(StandardEvent::turn_tool_started(
                            session_id, turn_id, tool_name, runtime,
                        ));
                    }
                }
            }
        }
        // assistant message — extract text as turn.delta
        "assistant" => {
            if let Some(message) = line.get("message") {
                if let Some(content) = message.get("content").and_then(|v| v.as_array()) {
                    for block in content {
                        if block.get("type").and_then(|v| v.as_str()) == Some("text") {
                            if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
                                if !text.is_empty() {
                                    bus.publish(StandardEvent::turn_delta(
                                        session_id, turn_id, text, runtime,
                                    ));
                                }
                            }
                        }
                        // Tool use in assistant message
                        if block.get("type").and_then(|v| v.as_str()) == Some("tool_use") {
                            let tool_name = block
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown");
                            bus.publish(StandardEvent::turn_tool_started(
                                session_id, turn_id, tool_name, runtime,
                            ));
                        }
                    }
                }
            }
        }
        // user message with tool result → turn.tool_finished
        "user" => {
            if let Some(message) = line.get("message") {
                if let Some(content) = message.get("content").and_then(|v| v.as_array()) {
                    for block in content {
                        if block.get("type").and_then(|v| v.as_str()) == Some("tool_result") {
                            let tool_id = block
                                .get("tool_use_id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown");
                            bus.publish(StandardEvent::turn_tool_finished(
                                session_id, turn_id, tool_id, runtime,
                            ));
                        }
                    }
                }
            }
        }
        // error event
        "error" => {
            let msg = line
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error");
            bus.publish(StandardEvent::turn_error(session_id, turn_id, msg, runtime));
        }
        _ => {
            // Unknown event types are silently ignored (heartbeat, ping, etc.)
        }
    }
}

fn turn_directory(workspace_root: &str, turn_id: &str) -> Result<PathBuf, String> {
    let root = PathBuf::from(workspace_root);
    // Auto-create workspace if it doesn't exist (e.g. chat mode uses /tmp/archbot-chat)
    if !root.exists() {
        fs::create_dir_all(&root)
            .map_err(|e| format!("Failed to create workspace {}: {}", workspace_root, e))?;
    }
    let canonical = root.canonicalize().map_err(|_| {
        format!(
            "Invalid workspace_root (not found or inaccessible): {}",
            workspace_root
        )
    })?;
    // 额外校验：不包含 .. 或符号链接逃逸
    if canonical.to_string_lossy().contains("..") {
        return Err(format!(
            "workspace_root contains path traversal: {}",
            workspace_root
        ));
    }
    Ok(canonical
        .join(".archbot")
        .join("agent")
        .join("turns")
        .join(turn_id))
}

fn get_machine_id() -> String {
    // 与 license 模块的 get_machine_id_cmd 一致
    if let Ok(host) = hostname::get() {
        return host.to_string_lossy().to_string();
    }
    "unknown-machine".to_string()
}

fn generate_input_yml(config: &TurnConfig, turn_id: &str) -> String {
    use serde::Serialize;

    #[derive(Serialize)]
    struct InputYml<'a> {
        schema_version: &'a str,
        session_header: SessionHeader<'a>,
        current_turn: CurrentTurn<'a>,
        recent_messages: Vec<Message<'a>>,
        working_context: WorkingContext<'a>,
        execution_policy: ExecutionPolicy<'a>,
        output_contract: OutputContract<'a>,
    }

    #[derive(Serialize)]
    struct SessionHeader<'a> {
        session_id: &'a str,
        title: &'a str,
        goal: &'a str,
    }

    #[derive(Serialize)]
    struct CurrentTurn<'a> {
        turn_id: &'a str,
        user_message: &'a str,
        expected_output: &'a str,
    }

    #[derive(Serialize)]
    struct Message<'a> {
        role: &'a str,
        content: &'a str,
    }

    #[derive(Serialize)]
    struct WorkingContext<'a> {
        project_root: &'a str,
        relevant_files: Vec<RelevantFile<'a>>,
    }

    #[derive(Serialize)]
    struct RelevantFile<'a> {
        path: &'a str,
    }

    #[derive(Serialize)]
    struct ExecutionPolicy<'a> {
        runtime: &'a str,
        mode: &'a str,
        allow_file_write: bool,
        allow_shell: bool,
        timeout_seconds: u32,
    }

    #[derive(Serialize)]
    struct OutputContract<'a> {
        stream: StreamConfig,
        final_result: FinalResultConfig<'a>,
        file_changes: FileChangesConfig<'a>,
    }

    #[derive(Serialize)]
    struct StreamConfig {
        enabled: bool,
    }

    #[derive(Serialize)]
    struct FinalResultConfig<'a> {
        markdown_file: &'a str,
        json_file: &'a str,
    }

    #[derive(Serialize)]
    struct FileChangesConfig<'a> {
        enabled: bool,
        output_file: &'a str,
    }

    let input = InputYml {
        schema_version: "agent_turn_input.v1",
        session_header: SessionHeader {
            session_id: turn_id,
            title: "ArchBot Agent Turn",
            goal: config.user_message.as_str(),
        },
        current_turn: CurrentTurn {
            turn_id,
            user_message: config.user_message.as_str(),
            expected_output: "根据用户指令分析代码并输出结果",
        },
        recent_messages: vec![Message {
            role: "user",
            content: config.user_message.as_str(),
        }],
        working_context: WorkingContext {
            project_root: config.workspace_root.as_str(),
            relevant_files: config
                .context_files
                .iter()
                .map(|f| RelevantFile { path: f.as_str() })
                .collect(),
        },
        execution_policy: ExecutionPolicy {
            runtime: config.runtime.as_str(),
            mode: "interactive_turn",
            allow_file_write: true,
            allow_shell: false,
            timeout_seconds: 1800,
        },
        output_contract: OutputContract {
            stream: StreamConfig { enabled: true },
            final_result: FinalResultConfig {
                markdown_file: "result.md",
                json_file: "result.json",
            },
            file_changes: FileChangesConfig {
                enabled: true,
                output_file: "file_changes.json",
            },
        },
    };

    serde_yml::to_string(&input).unwrap_or_else(|e| format!("# YAML serialize error: {}", e))
}

fn generate_prompt_txt(turn_dir: &PathBuf) -> String {
    format!(
        r#"请读取以下结构化输入文件，并严格根据其中内容执行本轮任务：

{}/input.yml

执行要求：
1. 以 input.yml 作为本轮真实输入，不要只根据本 prompt 执行
2. 遵守 input.yml 中的 working_context 和 execution_policy
3. 最终结果写入 result.md 和 result.json
4. 如果产生文件变更，写入 file_changes.json
"#,
        turn_dir.display()
    )
}

/// 构建自包含的 stdin prompt — 把所有上下文内联到一段文本中，
/// 通过管道传给 Runtime 子进程的 stdin（而非依赖虚构的 CLI 参数）。
fn build_stdin_prompt(config: &TurnConfig, input_yml: &str, turn_dir: &PathBuf) -> String {
    let files = if config.context_files.is_empty() {
        String::from("（无额外上下文文件）")
    } else {
        config
            .context_files
            .iter()
            .map(|f| format!("  - {}", f))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"You are executing a turn in an ArchBot agent session.

## Structured Input (YAML)
{input_yml}

## Output Contract
- Write your final result (markdown) to: {turn_dir}/result.md
- If you modified, created, or deleted any files, list them in JSON array format at: {turn_dir}/file_changes.json
  Example: [{{"path": "src/main.rs", "change_type": "modified"}}, ...]

## User Message
{user_message}

## Context Files
{context_files}

## Instructions
1. Read the structured YAML input above — it defines the session header, current turn, working context, and execution policy.
2. Your current working directory is the project root. Use Read/Write/Edit/Bash tools as needed.
3. Follow the execution_policy: timeout={timeout}s, allow_file_write=true, allow_shell=false.
4. Process the user's message and produce a thorough result.
5. Write result.md and file_changes.json as specified above.

Proceed with execution now.
"#,
        input_yml = input_yml,
        turn_dir = turn_dir.display(),
        user_message = config.user_message,
        context_files = files,
        timeout = 1800u32,
    )
}

fn parse_file_changes(path: &PathBuf) -> Vec<FileChange> {
    if !path.exists() {
        return vec![];
    }
    match fs::read_to_string(path) {
        Ok(content) => serde_json::from_str::<Vec<FileChange>>(&content).unwrap_or_default(),
        Err(_) => vec![],
    }
}

fn collect_accessed_paths(turn_dir: &PathBuf) -> Vec<String> {
    let stdout_path = turn_dir.join("stdout.log");
    if !stdout_path.exists() {
        return vec![];
    }
    match fs::read_to_string(&stdout_path) {
        Ok(content) => content
            .lines()
            .filter(|line| line.contains('/') || line.contains('\\'))
            .map(|line| line.to_string())
            .collect(),
        Err(_) => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// End-to-end test: 用真实 claude 二进制执行一个简单 turn。
    /// 需要 ANTHROPIC_AUTH_TOKEN 环境变量和 claude 在 /opt/homebrew/bin/claude。
    #[test]
    fn test_execute_turn_with_real_claude() {
        // 跳过条件
        if std::env::var("ANTHROPIC_AUTH_TOKEN").is_err() {
            eprintln!("SKIP: ANTHROPIC_AUTH_TOKEN not set");
            return;
        }
        let claude_path = "/opt/homebrew/bin/claude";
        if !Path::new(claude_path).exists() {
            eprintln!("SKIP: claude not found at {}", claude_path);
            return;
        }

        // 创建临时"项目"目录
        let tmp = std::env::temp_dir().join("archbot_e2e_test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        fs::write(tmp.join("README.md"), "# Test Project\n").unwrap();

        // 确保 runtimes.yml 指向可用的 claude 二进制（测试时覆盖）
        // 注意：测试依赖于用户已正确配置 ~/.archbot/config/runtimes.yml

        let config = TurnConfig {
            runtime: "claude_code".into(),
            workspace_root: tmp.to_string_lossy().to_string(),
            user_message: "List the files in the current directory and write a one-line summary to result.md.".into(),
            context_files: vec!["README.md".into()],
            git_user_name: None,
            git_user_email: None,
            session_id: None,
            model_override: None,
        };

        let result = execute_turn(config);

        match &result {
            Ok(r) => {
                eprintln!(
                    "Turn {} — status: {}, duration: {}ms",
                    r.turn_id, r.status, r.duration_ms
                );

                // 验证基础条件
                assert!(!r.stdout_tail.is_empty(), "stdout should not be empty");
                assert!(r.duration_ms > 0, "duration should be positive");

                // 如果 status 是 completed，验证 result.md 存在
                if r.status == "completed" {
                    if let Some(ref p) = r.result_md_path {
                        let path = Path::new(p);
                        assert!(path.exists(), "result.md should exist at {}", p);
                        let content = fs::read_to_string(path).unwrap_or_default();
                        assert!(!content.trim().is_empty(), "result.md should not be empty");
                        eprintln!("result.md content (first 200 chars): {}", &content[..content.len().min(200)]);
                    }
                }
            }
            Err(ref e) => {
                panic!("Turn execution failed: {}", e);
            }
        }

        // 清理
        let _ = fs::remove_dir_all(&tmp);
    }
}
