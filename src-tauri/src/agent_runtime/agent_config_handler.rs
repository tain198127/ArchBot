use serde::{Deserialize, Serialize};

use crate::agent_runtime::runtime_config::load_runtimes_config;
use crate::secret::SecretManager;

fn expand_home(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest).to_string_lossy().to_string();
        }
    }
    path.to_string()
}

// ── agent_get_status ──

#[derive(Serialize)]
pub struct AgentStatus {
    pub installed: bool,
    pub installed_version: String,
    pub available_versions: Vec<String>,
    pub config: Option<AgentConfigInfo>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct AgentConfigInfo {
    pub protocol: String,
    pub base_url: String,
    #[serde(default)]
    pub model_default: String,
    #[serde(default)]
    pub model_small: String,
    #[serde(default)]
    pub model_large: String,
    /// Legacy single-model field kept for OpenAI-compatible runtimes
    #[serde(default)]
    pub model_name: String,
    #[serde(default)]
    pub extra_args: String,
}

#[tauri::command]
pub fn agent_get_status(runtime: String) -> Result<AgentStatus, String> {
    let rt_config = load_runtimes_config()?;
    let entry = rt_config
        .runtimes
        .get(&runtime)
        .ok_or_else(|| format!("Runtime not found: {}", runtime))?;

    let executable = expand_home(&entry.executable);
    let installed = std::path::Path::new(&executable).exists();
    let installed_version = if installed {
        entry.current_version.clone()
    } else {
        String::new()
    };

    let config = entry.env.as_ref().map(|env| {
        let is_anthropic = env.get("ANTHROPIC_BASE_URL").is_some()
            || entry
                .args
                .as_ref()
                .map_or(false, |a| a.default.iter().any(|x| x.contains("claude")));
        AgentConfigInfo {
            protocol: if is_anthropic { "anthropic".into() } else { "openai".into() },
            base_url: env
                .get("ANTHROPIC_BASE_URL")
                .or_else(|| env.get("OPENAI_BASE_URL"))
                .cloned()
                .unwrap_or_default(),
            model_default: env.get("ANTHROPIC_MODEL").cloned().unwrap_or_default(),
            model_small: env.get("ANTHROPIC_SMALL_MODEL").cloned().unwrap_or_default(),
            model_large: env.get("ANTHROPIC_LARGE_MODEL").cloned().unwrap_or_default(),
            model_name: env.get("OPENAI_MODEL").cloned().unwrap_or_default(),
            extra_args: String::new(),
        }
    });

    Ok(AgentStatus {
        installed,
        installed_version,
        available_versions: vec![entry.current_version.clone()],
        config,
    })
}

// ── agent_install ──

#[tauri::command]
pub fn agent_install(runtime: String, version: Option<String>) -> Result<String, String> {
    let version = version.unwrap_or_default();
    Err(format!(
        "Install not yet implemented for {} v{}. Please manually install to ~/.archbot/runtimes/{}/",
        runtime, version, runtime
    ))
}

// ── agent_update ──

#[tauri::command]
pub fn agent_update(runtime: String) -> Result<String, String> {
    Err(format!("Update not yet implemented for {}", runtime))
}

// ── agent_save_config ──

#[tauri::command]
pub fn agent_save_config(runtime: String, config: AgentConfigInfo) -> Result<(), String> {
    // 写入 runtimes.yml 的对应 Runtime 段
    let mut rt_config = load_runtimes_config()?;
    let entry = rt_config
        .runtimes
        .get_mut(&runtime)
        .ok_or_else(|| format!("Runtime not found: {}", runtime))?;

    // 更新 env 中的 model 相关变量
    if let Some(env) = &mut entry.env {
        // 通用: 协议对应的 base URL
        if config.protocol == "anthropic" {
            env.insert("ANTHROPIC_BASE_URL".into(), config.base_url.clone());
        } else {
            env.insert("OPENAI_BASE_URL".into(), config.base_url.clone());
        }

        // Anthropic 协议 (Claude Code) 支持三档模型
        if config.protocol == "anthropic" {
            if !config.model_default.is_empty() {
                env.insert("ANTHROPIC_MODEL".into(), config.model_default.clone());
            }
            if !config.model_small.is_empty() {
                env.insert("ANTHROPIC_SMALL_MODEL".into(), config.model_small.clone());
            }
            if !config.model_large.is_empty() {
                env.insert("ANTHROPIC_LARGE_MODEL".into(), config.model_large.clone());
            }
        } else if !config.model_name.is_empty() {
            // OpenAI 兼容协议使用单模型字段
            env.insert("OPENAI_MODEL".into(), config.model_name.clone());
        }
    }

    let yml = serde_yml::to_string(&rt_config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    let path = dirs::home_dir()
        .unwrap_or_default()
        .join(".archbot")
        .join("config")
        .join("runtimes.yml");

    std::fs::write(&path, yml)
        .map_err(|e| format!("Failed to write runtimes.yml: {}", e))?;

    Ok(())
}

// ── agent_save_secret ──

#[tauri::command]
pub fn agent_save_secret(runtime: String, key: String, value: String) -> Result<(), String> {
    let machine_id = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let sm = SecretManager::new(&machine_id)?;
    sm.store(&runtime, &key, &value)
}

// ── agent_validate ──

#[derive(Serialize)]
pub struct ValidateResult {
    pub ok: bool,
    pub error: Option<String>,
}

#[tauri::command]
pub fn agent_validate(runtime: String) -> Result<ValidateResult, String> {
    let rt_config = load_runtimes_config()?;
    let entry = rt_config
        .runtimes
        .get(&runtime)
        .ok_or_else(|| format!("Runtime not found: {}", runtime))?;

    let executable = expand_home(&entry.executable);
    if !std::path::Path::new(&executable).exists() {
        return Ok(ValidateResult {
            ok: false,
            error: Some("Executable not found — please install first".into()),
        });
    }

    match std::process::Command::new(&executable)
        .arg("--version")
        .output()
    {
        Ok(output) if output.status.success() => Ok(ValidateResult {
            ok: true,
            error: None,
        }),
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Ok(ValidateResult {
                ok: false,
                error: Some(stderr),
            })
        }
        Err(e) => Ok(ValidateResult {
            ok: false,
            error: Some(e.to_string()),
        }),
    }
}
