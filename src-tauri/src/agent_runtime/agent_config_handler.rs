use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::agent_runtime::runtime_config::load_runtimes_config;
use crate::agent_runtime::version_manager;
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
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub provider_id: String,
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
    /// All environment variables to inject into the runtime subprocess
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env_vars: HashMap<String, String>,
}

#[tauri::command]
pub fn agent_get_status(runtime: String) -> Result<AgentStatus, String> {
    let rt_config = load_runtimes_config()?;
    let entry = rt_config
        .runtimes
        .get(&runtime)
        .ok_or_else(|| format!("Runtime not found: {}", runtime))?;

    // 1. Try version_manager managed install first (checks ~/.archbot/runtimes/{name}/current)
    let vm_version = version_manager::current_version(&runtime).unwrap_or_default();
    let mut available_versions = version_manager::detect_versions(&runtime).unwrap_or_default();

    // 2. Always include the config's current_version as a known version candidate
    let config_version = &entry.current_version;
    if !config_version.is_empty() && !available_versions.contains(config_version) {
        available_versions.push(config_version.clone());
    }

    let (installed, installed_version) = if !vm_version.is_empty() && vm_version != "not installed" {
        (true, vm_version)
    } else {
        // 3. Fallback: check if the executable from runtimes.yml exists and run --version
        let exe_path = expand_home(&entry.executable);
        let exe_path = std::path::Path::new(&exe_path);
        if exe_path.exists() {
            match std::process::Command::new(exe_path).arg("--version").output() {
                Ok(output) if output.status.success() => {
                    let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !ver.is_empty() && !available_versions.contains(&ver) {
                        available_versions.push(ver.clone());
                    }
                    (true, ver)
                }
                _ => {
                    let ver = config_version.clone();
                    if !ver.is_empty() && !available_versions.contains(&ver) {
                        available_versions.push(ver.clone());
                    }
                    (true, ver)
                }
            }
        } else {
            // Not installed — show config version so user knows what to install
            (false, config_version.clone())
        }
    };

    let config = entry.env.as_ref().map(|env| {
        let is_anthropic = env.get("ANTHROPIC_BASE_URL").is_some()
            || entry
                .args
                .as_ref()
                .map_or(false, |a| a.default.iter().any(|x| x.contains("claude")));
        // Build env_vars: include all env entries except the ones extracted as typed fields
        let env_vars: HashMap<String, String> = env.iter()
            .filter(|(k, _)| {
                !matches!(k.as_str(),
                    "ANTHROPIC_BASE_URL" | "ANTHROPIC_MODEL" |
                    "ANTHROPIC_SMALL_MODEL" | "ANTHROPIC_LARGE_MODEL" |
                    "OPENAI_BASE_URL" | "OPENAI_MODEL"
                )
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        AgentConfigInfo {
            provider_id: entry.provider_id.clone().unwrap_or_default(),
            protocol: if is_anthropic {
                "anthropic".into()
            } else {
                "openai".into()
            },
            base_url: env
                .get("ANTHROPIC_BASE_URL")
                .or_else(|| env.get("OPENAI_BASE_URL"))
                .cloned()
                .unwrap_or_default(),
            model_default: env.get("ANTHROPIC_MODEL").cloned().unwrap_or_default(),
            model_small: env
                .get("ANTHROPIC_SMALL_MODEL")
                .cloned()
                .unwrap_or_default(),
            model_large: env
                .get("ANTHROPIC_LARGE_MODEL")
                .cloned()
                .unwrap_or_default(),
            model_name: env.get("OPENAI_MODEL").cloned().unwrap_or_default(),
            extra_args: String::new(),
            env_vars,
        }
    });

    Ok(AgentStatus {
        installed,
        installed_version,
        available_versions,
        config,
    })
}

// ── agent_install (stub — real impl in version_manager) ──

#[allow(dead_code)]
pub fn agent_install_legacy(runtime: String, version: Option<String>) -> Result<String, String> {
    let version = version.unwrap_or_default();
    Err(format!(
        "Install not yet implemented for {} v{}. Please manually install to ~/.archbot/runtimes/{}/",
        runtime, version, runtime
    ))
}

// ── agent_update (stub — real impl in version_manager) ──

#[allow(dead_code)]
pub fn agent_update_legacy(runtime: String) -> Result<String, String> {
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

    // 持久化 provider 选择
    if config.provider_id.is_empty() {
        entry.provider_id = None;
    } else {
        entry.provider_id = Some(config.provider_id.clone());
    }

    // 确保 env section 存在
    if entry.env.is_none() {
        entry.env = Some(HashMap::new());
    }
    let env = entry.env.as_mut().unwrap();

    // Base URL — claude_code always needs Anthropic format.
    // For providers using OpenAI protocol (e.g. DeepSeek), convert the URL:
    //   https://api.deepseek.com/v1 → https://api.deepseek.com/anthropic
    let needs_anthropic = runtime == "claude_code";
    if needs_anthropic {
        let anthropic_url = if config.base_url.ends_with("/v1") {
            format!("{}/anthropic", config.base_url.trim_end_matches("/v1"))
        } else {
            config.base_url.clone()
        };
        env.insert("ANTHROPIC_BASE_URL".into(), anthropic_url);
    } else {
        env.insert("OPENAI_BASE_URL".into(), config.base_url.clone());
    }

    // Model — claude_code always uses Anthropic env vars regardless of provider protocol
    if needs_anthropic {
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
        env.insert("OPENAI_MODEL".into(), config.model_name.clone());
    }

    // Merge all custom env vars from the UI editor
    for (key, value) in &config.env_vars {
        if value.is_empty() {
            env.remove(key);
        } else {
            env.insert(key.clone(), value.clone());
        }
    }

    // Auto-whitelist: any env key the user explicitly configured must be allowed
    if let Some(exec) = &mut entry.execution {
        if let Some(ref mut isolation) = exec.isolation {
            for key in env.keys() {
                if !isolation.allowed_env_keys.contains(key) {
                    isolation.allowed_env_keys.push(key.clone());
                }
            }
        }
    }

    let yml = serde_yml::to_string(&rt_config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    let path = dirs::home_dir()
        .unwrap_or_default()
        .join(".archbot")
        .join("config")
        .join("runtimes.yml");

    std::fs::write(&path, yml).map_err(|e| format!("Failed to write runtimes.yml: {}", e))?;

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
