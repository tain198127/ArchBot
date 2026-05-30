use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::Child;
use std::time::Instant;

use crate::agent_runtime::audit::AuditManager;
use crate::agent_runtime::home_setup::setup_isolated_home;
use crate::agent_runtime::launcher::launch_isolated_runtime;
use crate::agent_runtime::runtime_config::{
    build_home_config, build_launch_config, load_runtimes_config,
};
use crate::agent_runtime::turn_config::{FileChange, TurnConfig, TurnResult};
use crate::secret::SecretManager;

/// Tauri command: 前端调用的入口
#[tauri::command]
pub fn agent_execute_turn(
    runtime: String,
    workspace_root: String,
    user_message: String,
    context_files: Vec<String>,
) -> Result<TurnResult, String> {
    execute_turn(TurnConfig {
        runtime,
        workspace_root,
        user_message,
        context_files,
        git_user_name: None,
        git_user_email: None,
    })
}

/// 执行一次完整的 Agent Turn——串联 launcher / home_setup / audit / secret。
pub fn execute_turn(config: TurnConfig) -> Result<TurnResult, String> {
    let start = Instant::now();
    let turn_id = uuid::Uuid::new_v4().to_string();

    // 1. 加载 Runtime 配置
    let rt_config = load_runtimes_config()?;
    let entry = rt_config
        .runtimes
        .get(&config.runtime)
        .ok_or_else(|| format!("Runtime not found in config: {}", config.runtime))?;

    if !entry.enabled {
        return Err(format!("Runtime disabled: {}", config.runtime));
    }

    // 2. 获取 API token + 构造 launch config
    let mut launch_config = build_launch_config(
        &config.runtime,
        entry,
        &config.workspace_root,
        config.git_user_name.as_deref(),
        config.git_user_email.as_deref(),
    )?;

    // 注入 API token（从 Secret Manager）
    if let Ok(sm) = SecretManager::new(&get_machine_id()) {
        let token_refs = [
            ("claude_code", "ANTHROPIC_AUTH_TOKEN", "secret://claude_code/api_token"),
            ("hermes", "HERMES_API_KEY", "secret://hermes/api_token"),
            ("opencode", "OPENAI_API_KEY", "secret://opencode/api_token"),
            ("openclaw", "OPENAI_API_KEY", "secret://openclaw/api_token"),
        ];
        for (rt, env_key, secret_ref) in &token_refs {
            if config.runtime == *rt {
                if let Ok(token) = sm.resolve(secret_ref) {
                    launch_config.allowed_env.insert(env_key.to_string(), token);
                }
            }
        }
    }

    // 3. 初始化隔离 HOME
    let home_config = build_home_config(entry)?;
    setup_isolated_home(&home_config)?;

    // 4. 生成输入文件
    let turn_dir = turn_directory(&config.workspace_root, &turn_id)?;
    fs::create_dir_all(&turn_dir)
        .map_err(|e| format!("Failed to create turn dir {:?}: {}", turn_dir, e))?;

    let input_yml_path = turn_dir.join("input.yml");
    let prompt_txt_path = turn_dir.join("prompt.txt");

    let input_yml = generate_input_yml(&config, &turn_id);
    let prompt_txt = generate_prompt_txt(&turn_dir);

    fs::write(&input_yml_path, &input_yml)
        .map_err(|e| format!("Failed to write input.yml: {}", e))?;
    fs::write(&prompt_txt_path, &prompt_txt)
        .map_err(|e| format!("Failed to write prompt.txt: {}", e))?;

    // 5. 构建 Runtime 启动参数
    let mut args = launch_config.args.clone();
    // 插入输入/输出路径参数
    args.push("--input-file".into());
    args.push(input_yml_path.to_string_lossy().to_string());
    args.push("--prompt-file".into());
    args.push(prompt_txt_path.to_string_lossy().to_string());
    args.push("--output-dir".into());
    args.push(turn_dir.to_string_lossy().to_string());
    launch_config.args = args;

    // 6. 启动 Runtime 子进程
    let mut child = launch_isolated_runtime(&launch_config)?;

    // 7. 等待进程退出（带超时）
    let timeout = std::time::Duration::from_secs(launch_config.timeout_seconds);
    let (stdout, stderr, status) = wait_with_timeout(&mut child, timeout)?;

    // 写入 stdout/stderr 日志
    let _ = fs::write(turn_dir.join("stdout.log"), &stdout);
    let _ = fs::write(turn_dir.join("stderr.log"), &stderr);

    // 8. 解析结果
    let result_md_path = turn_dir.join("result.md");
    let result_md = if result_md_path.exists() {
        Some(result_md_path.to_string_lossy().to_string())
    } else {
        None
    };

    let file_changes = parse_file_changes(&turn_dir.join("file_changes.json"));

    let stdout_tail = if stdout.len() > 2000 {
        format!("...(truncated)\n{}", &stdout[stdout.len() - 2000..])
    } else {
        stdout.clone()
    };

    // 9. 文件访问审计
    let audit_manager = AuditManager::new();
    let accessed = collect_accessed_paths(&turn_dir);
    let violations = audit_manager.audit(&accessed);
    let audit_violations: Vec<String> = violations
        .iter()
        .map(|v| format!("{:?}: {}", v.rule.severity, v.accessed_path))
        .collect();

    let status_str = if status.success() {
        "completed".to_string()
    } else {
        format!("failed: exit code {:?}", status.code())
    };

    Ok(TurnResult {
        turn_id,
        runtime: config.runtime,
        status: status_str,
        stdout_tail,
        result_md_path: result_md,
        file_changes,
        audit_violations,
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

/// 带超时的进程等待
fn wait_with_timeout(child: &mut Child, timeout: std::time::Duration) -> Result<(String, String, std::process::ExitStatus), String> {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut stdout = String::new();
                let mut stderr = String::new();
                if let Some(mut out) = child.stdout.take() {
                    let _ = out.read_to_string(&mut stdout);
                }
                if let Some(mut err) = child.stderr.take() {
                    let _ = err.read_to_string(&mut stderr);
                }
                return Ok((stdout, stderr, status));
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    return Err(format!("Turn timed out after {:?}", timeout));
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            Err(e) => return Err(format!("Failed to wait on child process: {}", e)),
        }
    }
}

fn turn_directory(workspace_root: &str, turn_id: &str) -> Result<PathBuf, String> {
    let root = PathBuf::from(workspace_root);
    let canonical = root
        .canonicalize()
        .map_err(|_| format!("Invalid workspace_root (not found or inaccessible): {}", workspace_root))?;
    // 额外校验：不包含 .. 或符号链接逃逸
    if canonical.to_string_lossy().contains("..") {
        return Err(format!("workspace_root contains path traversal: {}", workspace_root));
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
    // 第一版：扫描 stdout.log 中所有文件路径模式
    // 后续可通过 strace/dtruss 包装获得更精确的列表
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
