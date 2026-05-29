//! 上下文工程配置模块
//!
//! 管理 Rules、Memory、Session、Codebase、Git 五个维度的配置。
//! 每个 section 对应 `.archbot/context/` 下的独立文件或子目录。
//! 配置与数字员工角色支持联动（通过 `linked_agents` 字段）。

use serde::{Deserialize, Serialize};
use std::path::Path;

/// 上下文配置条目（用于 rules、memory 等列表型 section）
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContextEntry {
    pub name: String,
    pub description: String,
    pub content: String,
    #[serde(default)]
    pub linked_agents: Vec<String>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

/// 获取 `.archbot/context/` 目录路径
fn context_dir(project_path: &str) -> Result<std::path::PathBuf, String> {
    let ab_path = Path::new(project_path);
    let project_dir = ab_path.parent().ok_or("无法获取项目目录")?;
    Ok(project_dir.join(".archbot").join("context"))
}

/// 获取 section 对应的文件路径或目录路径
///
/// `rules` 和 `memory` 是目录（含多个条目），其余是单文件。
fn section_path(project_path: &str, section: &str) -> Result<std::path::PathBuf, String> {
    let dir = context_dir(project_path)?;
    match section {
        "rules" | "memory" => Ok(dir.join(section)),
        _ => Ok(dir.join(format!("{section}.yml"))),
    }
}

/// 读取上下文配置（单文件型 section）
///
/// 返回 YAML 字符串，文件不存在时返回空字符串。
#[tauri::command]
pub fn get_context_config(project_path: String, section: String) -> Result<String, String> {
    validate_section(&section)?;
    let path = section_path(&project_path, &section)?;
    if !path.exists() {
        return Ok(String::new());
    }
    std::fs::read_to_string(&path).map_err(|e| format!("读取上下文配置失败: {e}"))
}

/// 保存上下文配置（单文件型 section）
///
/// `content` 为 YAML 字符串，自动创建父目录。
#[tauri::command]
pub fn save_context_config(
    project_path: String,
    section: String,
    content: String,
) -> Result<(), String> {
    validate_section(&section)?;
    let path = section_path(&project_path, &section)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建上下文配置目录失败: {e}"))?;
    }
    std::fs::write(&path, content).map_err(|e| format!("保存上下文配置失败: {e}"))
}

/// 列出上下文 section 下的所有条目（目录型 section）
///
/// 仅 `rules` 和 `memory` 支持，返回文件名列表（不含扩展名）。
#[tauri::command]
pub fn list_context_entries(project_path: String, section: String) -> Result<Vec<String>, String> {
    validate_section(&section)?;
    let dir = section_path(&project_path, &section)?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    let read_dir =
        std::fs::read_dir(&dir).map_err(|e| format!("读取目录失败: {e}"))?;
    for entry in read_dir {
        let entry = entry.map_err(|e| format!("读取条目失败: {e}"))?;
        if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            if let Some(name) = entry.path().file_stem() {
                entries.push(name.to_string_lossy().to_string());
            }
        }
    }
    Ok(entries)
}

/// 读取单条上下文条目（目录型 section 下的单个文件）
#[tauri::command]
pub fn get_context_entry(
    project_path: String,
    section: String,
    name: String,
) -> Result<ContextEntry, String> {
    validate_section(&section)?;
    validate_entry_name(&name)?;
    if !matches!(section.as_str(), "rules" | "memory") {
        return Err("仅 rules 和 memory 支持条目级读取".into());
    }
    let dir = section_path(&project_path, &section)?;
    let path = dir.join(format!("{name}.yml"));
    if !path.exists() {
        return Err(format!("条目不存在: {name}"));
    }
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("读取条目失败: {e}"))?;
    serde_yml::from_str(&content).map_err(|e| format!("解析条目失败: {e}"))
}

/// 保存单条上下文条目
#[tauri::command]
pub fn save_context_entry(
    project_path: String,
    section: String,
    entry: ContextEntry,
) -> Result<(), String> {
    validate_section(&section)?;
    validate_entry_name(&entry.name)?;
    if !matches!(section.as_str(), "rules" | "memory") {
        return Err("仅 rules 和 memory 支持条目级保存".into());
    }
    let dir = section_path(&project_path, &section)?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败: {e}"))?;
    let path = dir.join(format!("{}.yml", entry.name));
    let yaml = serde_yml::to_string(&entry).map_err(|e| format!("序列化条目失败: {e}"))?;
    std::fs::write(&path, yaml).map_err(|e| format!("保存条目失败: {e}"))
}

/// 删除单条上下文条目
#[tauri::command]
pub fn delete_context_entry(
    project_path: String,
    section: String,
    name: String,
) -> Result<(), String> {
    validate_section(&section)?;
    validate_entry_name(&name)?;
    if !matches!(section.as_str(), "rules" | "memory") {
        return Err("仅 rules 和 memory 支持条目级删除".into());
    }
    let dir = section_path(&project_path, &section)?;
    let path = dir.join(format!("{name}.yml"));
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("删除条目失败: {e}"))?;
    }
    Ok(())
}

/// 校验 section 名称是否合法
fn validate_section(section: &str) -> Result<(), String> {
    match section {
        "rules" | "memory" | "sessions" | "codebase" | "git" => Ok(()),
        _ => Err(format!("未知的上下文 section: {section}")),
    }
}

/// 校验条目名称，防止路径穿越
fn validate_entry_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 128 {
        return Err("条目名称长度不合法".into());
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err("条目名称包含非法字符".into());
    }
    Ok(())
}
