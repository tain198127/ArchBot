use serde::{Deserialize, Serialize};

/// 一次 Agent Turn 的输入配置
#[derive(Debug, Clone)]
pub struct TurnConfig {
    pub runtime: String,
    pub workspace_root: String,
    pub user_message: String,
    pub context_files: Vec<String>,
    pub git_user_name: Option<String>,
    pub git_user_email: Option<String>,
    /// Optional session context — enables SSE event streaming with session_id
    pub session_id: Option<String>,
}

/// Turn 结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnResult {
    pub turn_id: String,
    pub runtime: String,
    pub status: String, // "completed" | "failed" | "timeout"
    pub stdout_tail: String,
    pub result_md_path: Option<String>,
    pub file_changes: Vec<FileChange>,
    pub audit_violations: Vec<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub change_type: String,
}

impl TurnResult {
    pub fn failed(turn_id: String, runtime: String, reason: String, duration_ms: u64) -> Self {
        Self {
            turn_id,
            runtime,
            status: format!("failed: {}", reason),
            stdout_tail: String::new(),
            result_md_path: None,
            file_changes: vec![],
            audit_violations: vec![],
            duration_ms,
        }
    }
}
