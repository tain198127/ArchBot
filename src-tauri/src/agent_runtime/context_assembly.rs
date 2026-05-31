use serde::{Deserialize, Serialize};
use std::fs;

// ─── Context Assembler ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub session_id: String,
    pub title: String,
    pub goal: String,
    pub current_state: String,
    pub recent_messages: Vec<ContextMessage>,
    pub compressed_history: Option<CompressedHistory>,
    pub decision_log: Vec<DecisionEntry>,
    pub working_context: WorkingContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedHistory {
    pub summary: String,
    pub important_context: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionEntry {
    pub id: String,
    pub decision: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingContext {
    pub project_root: String,
    pub current_focus: Vec<String>,
    pub relevant_files: Vec<RelevantFileRef>,
    pub relevant_artifacts: Vec<ArtifactRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevantFileRef {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRef {
    pub r#type: String,
    pub path: String,
}

/// ContextAssembler builds the complete context for each turn from session state.
pub struct ContextAssembler {
    pub sliding_window_size: usize,
}

impl Default for ContextAssembler {
    fn default() -> Self {
        Self {
            sliding_window_size: 10,
        }
    }
}

impl ContextAssembler {
    pub fn new(sliding_window_size: usize) -> Self {
        Self {
            sliding_window_size,
        }
    }

    /// Select messages within the sliding window.
    pub fn select_window(&self, messages: &[ContextMessage]) -> Vec<ContextMessage> {
        if messages.len() <= self.sliding_window_size {
            messages.to_vec()
        } else {
            messages[messages.len() - self.sliding_window_size..].to_vec()
        }
    }

    /// Generate a compressed summary from messages outside the window.
    pub fn compress_history(&self, messages: &[ContextMessage]) -> CompressedHistory {
        let overflow = if messages.len() > self.sliding_window_size {
            messages.len() - self.sliding_window_size
        } else {
            0
        };
        let truncated = if overflow > 0 {
            &messages[..overflow]
        } else {
            &[]
        };

        let summary = if truncated.is_empty() {
            "No earlier messages to compress.".into()
        } else {
            format!("{} earlier messages compressed.", truncated.len())
        };

        CompressedHistory {
            summary,
            important_context: vec![],
        }
    }

    /// Build the full input.yml content as a string for this turn.
    pub fn assemble_turn_input(
        &self,
        context: &SessionContext,
        turn_id: &str,
        user_message: &str,
        runtime_type: &str,
        project_root: &str,
        context_files: &[String],
    ) -> String {
        let recent = self.select_window(&context.recent_messages);
        let compressed = if context.compressed_history.is_some() {
            context.compressed_history.clone()
        } else {
            Some(self.compress_history(&context.recent_messages))
        };

        let mut yml = String::new();
        yml.push_str(&format!("schema_version: \"agent_turn_input.v1\"\n"));
        yml.push_str(&format!("\nsession_header:\n"));
        yml.push_str(&format!("  session_id: \"{}\"\n", context.session_id));
        yml.push_str(&format!("  title: \"{}\"\n", context.title));
        yml.push_str(&format!("  goal: \"{}\"\n", context.goal));
        yml.push_str(&format!("  current_state: \"{}\"\n", context.current_state));

        yml.push_str(&format!("\ncurrent_turn:\n"));
        yml.push_str(&format!("  turn_id: \"{}\"\n", turn_id));
        yml.push_str(&format!("  user_message: \"{}\"\n", user_message));
        yml.push_str(&format!(
            "  expected_output: \"根据用户指令分析代码并输出结果\"\n"
        ));

        yml.push_str(&format!("\nrecent_messages:\n"));
        for msg in &recent {
            yml.push_str(&format!(
                "  - role: {}\n    content: {}\n",
                msg.role, msg.content
            ));
        }

        if let Some(ref ch) = compressed {
            yml.push_str(&format!("\ncompressed_history:\n"));
            yml.push_str(&format!("  summary: >\n    {}\n", ch.summary));
            yml.push_str(&format!("  important_context:\n"));
            for ctx in &ch.important_context {
                yml.push_str(&format!("    - \"{}\"\n", ctx));
            }
        }

        yml.push_str(&format!("\ndecision_log:\n"));
        for dec in &context.decision_log {
            yml.push_str(&format!(
                "  - id: \"{}\"\n    decision: \"{}\"\n    status: \"{}\"\n",
                dec.id, dec.decision, dec.status
            ));
        }

        yml.push_str(&format!("\nworking_context:\n"));
        yml.push_str(&format!("  project_root: \"{}\"\n", project_root));
        yml.push_str(&format!("  relevant_files:\n"));
        for f in context_files {
            yml.push_str(&format!("    - path: \"{}\"\n", f));
        }

        yml.push_str(&format!("\nexecution_policy:\n"));
        yml.push_str(&format!("  runtime: \"{}\"\n", runtime_type));
        yml.push_str(&format!("  mode: \"interactive_turn\"\n"));
        yml.push_str(&format!("  allow_file_write: true\n"));
        yml.push_str(&format!("  allow_shell: false\n"));
        yml.push_str(&format!("  timeout_seconds: 1800\n"));

        yml.push_str(&format!("\noutput_contract:\n"));
        yml.push_str(&format!("  stream:\n    enabled: true\n"));
        yml.push_str(&format!(
            "  final_result:\n    markdown_file: \"result.md\"\n    json_file: \"result.json\"\n"
        ));
        yml.push_str(&format!(
            "  file_changes:\n    enabled: true\n    output_file: \"file_changes.json\"\n"
        ));
        yml.push_str(&format!(
            "  decisions:\n    enabled: true\n    output_file: \"decision_candidates.yml\"\n"
        ));
        yml.push_str(&format!(
            "  context_update:\n    enabled: true\n    output_file: \"context_update.yml\"\n"
        ));

        yml
    }

    /// Parse context_update.yml from turn output and update session working context.
    pub fn update_context_from_turn(
        &self,
        context: &mut SessionContext,
        turn_dir: &std::path::Path,
    ) -> Result<(), String> {
        let update_path = turn_dir.join("context_update.yml");
        if !update_path.exists() {
            return Ok(());
        }
        let content = fs::read_to_string(&update_path)
            .map_err(|e| format!("failed to read context_update.yml: {}", e))?;

        // Simple YAML parsing for context updates
        if let Ok(parsed) = serde_yml::from_str::<serde_yml::Value>(&content) {
            if let Some(state) = parsed.get("current_state").and_then(|v| v.as_str()) {
                context.current_state = state.to_string();
            }
            if let Some(focus) = parsed.get("current_focus").and_then(|v| v.as_sequence()) {
                context.working_context.current_focus = focus
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
            }
        }

        Ok(())
    }

    /// Persist a context snapshot for the session.
    pub fn snapshot(&self, context: &SessionContext, turn_id: Option<&str>) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "session_id": context.session_id,
            "turn_id": turn_id,
            "snapshot_type": if turn_id.is_some() { "pre_turn" } else { "session_start" },
            "recent_messages_count": context.recent_messages.len(),
            "decision_count": context.decision_log.len(),
            "compressed_history_present": context.compressed_history.is_some(),
            "current_state": context.current_state,
        }))
        .unwrap_or_default()
    }
}

// ─── Tauri Commands ───

#[tauri::command]
pub fn agent_assemble_context(
    session_id: String,
    turn_id: String,
    user_message: String,
    runtime_type: String,
    project_root: String,
    context_files: Vec<String>,
) -> Result<String, String> {
    let assembler = ContextAssembler::default();
    let context = SessionContext {
        session_id,
        title: "ArchBot Turn".into(),
        goal: user_message.clone(),
        current_state: String::new(),
        recent_messages: vec![ContextMessage {
            role: "user".into(),
            content: user_message,
        }],
        compressed_history: None,
        decision_log: vec![],
        working_context: WorkingContext {
            project_root: project_root.clone(),
            current_focus: vec![],
            relevant_files: vec![],
            relevant_artifacts: vec![],
        },
    };
    Ok(assembler.assemble_turn_input(
        &context,
        &turn_id,
        "",
        &runtime_type,
        &project_root,
        &context_files,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sliding_window_within_limit() {
        let assembler = ContextAssembler::new(10);
        let messages: Vec<ContextMessage> = (0..5)
            .map(|i| ContextMessage {
                role: "user".into(),
                content: format!("msg {}", i),
            })
            .collect();
        let window = assembler.select_window(&messages);
        assert_eq!(window.len(), 5);
    }

    #[test]
    fn test_sliding_window_over_limit() {
        let assembler = ContextAssembler::new(3);
        let messages: Vec<ContextMessage> = (0..10)
            .map(|i| ContextMessage {
                role: "user".into(),
                content: format!("msg {}", i),
            })
            .collect();
        let window = assembler.select_window(&messages);
        assert_eq!(window.len(), 3);
        assert_eq!(window[0].content, "msg 7");
        assert_eq!(window[2].content, "msg 9");
    }

    #[test]
    fn test_assemble_turn_input_has_all_sections() {
        let assembler = ContextAssembler::new(3);
        let context = SessionContext {
            session_id: "sess_001".into(),
            title: "Test".into(),
            goal: "Test goal".into(),
            current_state: "initial".into(),
            recent_messages: vec![],
            compressed_history: None,
            decision_log: vec![],
            working_context: WorkingContext {
                project_root: "/test".into(),
                current_focus: vec![],
                relevant_files: vec![],
                relevant_artifacts: vec![],
            },
        };
        let yml = assembler.assemble_turn_input(
            &context,
            "turn_001",
            "hello",
            "claude_code",
            "/test",
            &["src/main.rs".into()],
        );
        assert!(yml.contains("schema_version"));
        assert!(yml.contains("session_header"));
        assert!(yml.contains("current_turn"));
        assert!(yml.contains("execution_policy"));
        assert!(yml.contains("output_contract"));
    }
}
