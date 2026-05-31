use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::agent_runtime::config::{IsolatedHomeConfig, RuntimeLaunchConfig};

/// runtimes.yml 顶层结构
#[derive(Debug, Deserialize, Serialize)]
pub struct RuntimesConfig {
    pub runtimes: HashMap<String, RuntimeEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RuntimeEntry {
    pub enabled: bool,
    pub current_version: String,
    pub executable: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub args: Option<RuntimeArgs>,
    pub execution: Option<ExecutionConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RuntimeArgs {
    pub default: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExecutionConfig {
    pub default_timeout_seconds: u64,
    pub working_dir_policy: Option<String>,
    pub isolation: Option<IsolationConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IsolationConfig {
    pub mode: String,
    pub isolated_home: String,
    pub env_policy: String,
    pub allowed_env_keys: Vec<String>,
    pub bridge: Option<BridgeConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BridgeConfig {
    pub git: bool,
    pub ssh: bool,
}

/// 加载 runtimes.yml。
///
/// 优先读取 `~/.archbot/config/runtimes.yml`，
/// 不存在时回退到应用内嵌的 `runtimes.default.yml`。
pub fn load_runtimes_config() -> Result<RuntimesConfig, String> {
    let user_path = user_runtimes_path();

    let content = if user_path.exists() {
        std::fs::read_to_string(&user_path)
            .map_err(|e| format!("Failed to read {:?}: {}", user_path, e))?
    } else {
        // 回退到内嵌默认配置
        let default_path = default_runtimes_path()?;
        std::fs::read_to_string(&default_path)
            .map_err(|e| format!("Failed to read default config {:?}: {}", default_path, e))?
    };

    serde_yml::from_str::<RuntimesConfig>(&content)
        .map_err(|e| format!("Failed to parse runtimes.yml: {}", e))
}

/// 从 RuntimeEntry 构造 RuntimeLaunchConfig
pub fn build_launch_config(
    runtime_type: &str,
    entry: &RuntimeEntry,
    workspace_root: &str,
    git_user_name: Option<&str>,
    git_user_email: Option<&str>,
) -> Result<RuntimeLaunchConfig, String> {
    let exec = entry
        .execution
        .as_ref()
        .ok_or_else(|| format!("[config] Missing execution section for {}", runtime_type))?;
    let isolation = exec
        .isolation
        .as_ref()
        .ok_or_else(|| format!("[config] Missing isolation section for {}", runtime_type))?;

    // 从 env 中筛选出白名单允许的 key
    let mut allowed_env = HashMap::new();
    if let Some(env) = &entry.env {
        for key in &isolation.allowed_env_keys {
            if let Some(value) = env.get(key) {
                allowed_env.insert(key.clone(), value.clone());
            }
        }
    }

    let args = entry
        .args
        .as_ref()
        .map(|a| a.default.clone())
        .unwrap_or_default();

    Ok(RuntimeLaunchConfig {
        runtime_type: runtime_type.to_string(),
        executable: expand_home(&entry.executable),
        workspace_root: workspace_root.to_string(),
        isolated_home: expand_home(&isolation.isolated_home),
        allowed_env,
        args,
        timeout_seconds: exec.default_timeout_seconds,
        stdin_content: None,
    })
}

/// 从 RuntimeEntry 构造 IsolatedHomeConfig
pub fn build_home_config(entry: &RuntimeEntry) -> Result<IsolatedHomeConfig, String> {
    let isolation = entry
        .execution
        .as_ref()
        .and_then(|e| e.isolation.as_ref())
        .ok_or_else(|| "[config] Missing isolation section".to_string())?;

    let bridge = isolation.bridge.as_ref();

    Ok(IsolatedHomeConfig {
        home_path: PathBuf::from(expand_home(&isolation.isolated_home)),
        needs_git: bridge.map(|b| b.git).unwrap_or(false),
        git_user_name: None,
        git_user_email: None,
        needs_ssh: bridge.map(|b| b.ssh).unwrap_or(false),
        ssh_key_path: None,
    })
}

fn user_runtimes_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".archbot")
        .join("config")
        .join("runtimes.yml")
}

fn default_runtimes_path() -> Result<PathBuf, String> {
    // Tauri v2 resource: 打包时 config/ 目录被复制到 App 的 resource 目录
    // 开发时: 相对于 src-tauri/ 目录
    let dev_path = PathBuf::from("config/runtimes.default.yml");
    if dev_path.exists() {
        return Ok(dev_path);
    }

    // 生产时: 从 Tauri resource 目录读取
    if let Ok(resource_dir) = std::env::var("TAURI_RESOURCE_DIR") {
        let prod_path = PathBuf::from(&resource_dir).join("config/runtimes.default.yml");
        if prod_path.exists() {
            return Ok(prod_path);
        }
    }

    Err("[config] Cannot find runtimes.default.yml".to_string())
}

fn expand_home(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest).to_string_lossy().to_string();
        }
    }
    path.to_string()
}
