use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::turn_config::{TurnConfig, TurnResult};

/// Standardised result from a runtime adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterHealth {
    pub runtime_type: String,
    pub available: bool,
    pub version: String,
    pub capabilities: Vec<String>,
}

/// Unified adapter trait — all runtime adapters implement this.
#[async_trait]
pub trait AgentAdapter: Send + Sync {
    /// Check if the runtime is installed and responsive.
    async fn health(&self) -> Result<AdapterHealth, String>;

    /// Return the capabilities this runtime supports.
    async fn capabilities(&self) -> Vec<String>;

    /// Return the installed version string.
    async fn version(&self) -> Result<String, String>;

    /// Execute a single turn.
    async fn execute_turn(&self, config: &TurnConfig) -> Result<TurnResult, String>;

    /// Cancel a running turn.
    async fn cancel_turn(&self, turn_id: &str) -> Result<(), String>;

    /// Return the runtime type identifier.
    fn runtime_type(&self) -> &str;
}

// ── Claude Code Adapter ──

pub struct ClaudeCodeAdapter;

#[async_trait]
impl AgentAdapter for ClaudeCodeAdapter {
    fn runtime_type(&self) -> &str {
        "claude_code"
    }

    async fn health(&self) -> Result<AdapterHealth, String> {
        let output = std::process::Command::new("claude")
            .arg("--version")
            .output()
            .map_err(|e| format!("claude not found: {}", e))?;
        Ok(AdapterHealth {
            runtime_type: "claude_code".into(),
            available: output.status.success(),
            version: String::from_utf8_lossy(&output.stdout).trim().into(),
            capabilities: vec![
                "code_analysis".into(),
                "code_generation".into(),
                "refactoring".into(),
                "debugging".into(),
                "code_review".into(),
            ],
        })
    }

    async fn capabilities(&self) -> Vec<String> {
        vec![
            "code_analysis",
            "code_generation",
            "refactoring",
            "debugging",
            "code_review",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }

    async fn version(&self) -> Result<String, String> {
        let output = std::process::Command::new("claude")
            .arg("--version")
            .output()
            .map_err(|e| format!("claude not found: {}", e))?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().into())
    }

    async fn execute_turn(&self, config: &TurnConfig) -> Result<TurnResult, String> {
        super::turn_executor::execute_turn(config.clone())
    }

    async fn cancel_turn(&self, _turn_id: &str) -> Result<(), String> {
        Ok(()) // v1: process-level cancellation handled by timeout
    }
}

// ── Adapter Stubs (OpenCode, Hermes, OpenClaw) ──

pub struct OpenCodeAdapter;

#[async_trait]
impl AgentAdapter for OpenCodeAdapter {
    fn runtime_type(&self) -> &str {
        "opencode"
    }
    async fn health(&self) -> Result<AdapterHealth, String> {
        Ok(AdapterHealth {
            runtime_type: "opencode".into(),
            available: false,
            version: "not installed".into(),
            capabilities: vec!["code_analysis".into(), "code_generation".into()],
        })
    }
    async fn capabilities(&self) -> Vec<String> {
        vec!["code_analysis", "code_generation"]
            .into_iter()
            .map(String::from)
            .collect()
    }
    async fn version(&self) -> Result<String, String> {
        Err("OpenCode not yet installed".into())
    }
    async fn execute_turn(&self, config: &TurnConfig) -> Result<TurnResult, String> {
        super::turn_executor::execute_turn(config.clone())
    }
    async fn cancel_turn(&self, _turn_id: &str) -> Result<(), String> {
        Ok(())
    }
}

pub struct HermesAdapter;

#[async_trait]
impl AgentAdapter for HermesAdapter {
    fn runtime_type(&self) -> &str {
        "hermes"
    }
    async fn health(&self) -> Result<AdapterHealth, String> {
        Ok(AdapterHealth {
            runtime_type: "hermes".into(),
            available: false,
            version: "not installed".into(),
            capabilities: vec!["task_execution".into(), "agent_orchestration".into()],
        })
    }
    async fn capabilities(&self) -> Vec<String> {
        vec!["task_execution", "agent_orchestration"]
            .into_iter()
            .map(String::from)
            .collect()
    }
    async fn version(&self) -> Result<String, String> {
        Err("Hermes not yet installed".into())
    }
    async fn execute_turn(&self, config: &TurnConfig) -> Result<TurnResult, String> {
        super::turn_executor::execute_turn(config.clone())
    }
    async fn cancel_turn(&self, _turn_id: &str) -> Result<(), String> {
        Ok(())
    }
}

pub struct OpenClawAdapter;

#[async_trait]
impl AgentAdapter for OpenClawAdapter {
    fn runtime_type(&self) -> &str {
        "openclaw"
    }
    async fn health(&self) -> Result<AdapterHealth, String> {
        Ok(AdapterHealth {
            runtime_type: "openclaw".into(),
            available: false,
            version: "not installed".into(),
            capabilities: vec!["task_execution".into(), "plugin_system".into()],
        })
    }
    async fn capabilities(&self) -> Vec<String> {
        vec!["task_execution", "plugin_system"]
            .into_iter()
            .map(String::from)
            .collect()
    }
    async fn version(&self) -> Result<String, String> {
        Err("OpenClaw not yet installed".into())
    }
    async fn execute_turn(&self, config: &TurnConfig) -> Result<TurnResult, String> {
        super::turn_executor::execute_turn(config.clone())
    }
    async fn cancel_turn(&self, _turn_id: &str) -> Result<(), String> {
        Ok(())
    }
}

// ── Adapter Registry ──

use std::sync::Arc;

pub struct AdapterRegistry {
    adapters: Vec<Arc<dyn AgentAdapter>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self {
            adapters: vec![
                Arc::new(ClaudeCodeAdapter),
                Arc::new(OpenCodeAdapter),
                Arc::new(HermesAdapter),
                Arc::new(OpenClawAdapter),
            ],
        }
    }

    pub fn get(&self, runtime_type: &str) -> Option<Arc<dyn AgentAdapter>> {
        self.adapters
            .iter()
            .find(|a| a.runtime_type() == runtime_type)
            .cloned()
    }

    pub fn list_all(&self) -> Vec<String> {
        self.adapters
            .iter()
            .map(|a| a.runtime_type().to_string())
            .collect()
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tauri Commands ───

#[tauri::command]
pub fn agent_check_runtime_health(runtime: String) -> Result<AdapterHealth, String> {
    let registry = AdapterRegistry::new();
    let adapter = registry
        .get(&runtime)
        .ok_or_else(|| format!("unsupported runtime: {}", runtime))?;
    let rt = tokio::runtime::Handle::current();
    rt.block_on(adapter.health())
}

#[tauri::command]
pub fn agent_get_runtime_capabilities(runtime: String) -> Result<Vec<String>, String> {
    let registry = AdapterRegistry::new();
    let adapter = registry
        .get(&runtime)
        .ok_or_else(|| format!("unsupported runtime: {}", runtime))?;
    let rt = tokio::runtime::Handle::current();
    Ok(rt.block_on(adapter.capabilities()))
}
