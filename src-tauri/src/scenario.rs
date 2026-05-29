//! 项目场景管理模块
//!
//! 提供三种项目场景（从0到1 / 老项目改造 / 产品化二开）的配置读写。
//! 场景配置影响数字员工、业务流程、Skill 集、协作模式和目录结构。
//! 配置保存在 `{project_dir}/.archbot/scenario.yml`。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// 场景的五个影响维度
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ScenarioDimension {
    pub digital_employees: Vec<String>,
    pub business_flow: Vec<String>,
    pub skills: Vec<String>,
    pub collaboration_mode: String,
    pub dir_structure: Vec<String>,
}

/// 项目场景配置
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectScenario {
    #[serde(rename = "type")]
    pub scenario_type: String,
    #[serde(default)]
    pub overrides: HashMap<String, serde_json::Value>,
    pub applied_at: String,
}

impl Default for ProjectScenario {
    fn default() -> Self {
        Self {
            scenario_type: String::new(),
            overrides: HashMap::new(),
            applied_at: String::new(),
        }
    }
}

/// 获取 `.archbot/scenario.yml` 的完整路径
fn scenario_path(project_path: &str) -> Result<std::path::PathBuf, String> {
    let ab_path = Path::new(project_path);
    let project_dir = ab_path
        .parent()
        .ok_or("无法获取项目目录")?;
    Ok(project_dir.join(".archbot").join("scenario.yml"))
}

/// 读取项目场景配置
///
/// 文件不存在时返回默认值（空场景），不报错。
#[tauri::command]
pub fn get_scenario(project_path: String) -> Result<ProjectScenario, String> {
    let path = scenario_path(&project_path)?;
    if !path.exists() {
        return Ok(ProjectScenario::default());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取场景配置失败: {e}"))?;
    serde_yml::from_str(&content)
        .map_err(|e| format!("解析场景配置失败: {e}"))
}

/// 保存项目场景配置
#[tauri::command]
pub fn save_scenario(project_path: String, scenario: ProjectScenario) -> Result<(), String> {
    let path = scenario_path(&project_path)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建场景配置目录失败: {e}"))?;
    }
    let yaml = serde_yml::to_string(&scenario)
        .map_err(|e| format!("序列化场景配置失败: {e}"))?;
    std::fs::write(&path, yaml)
        .map_err(|e| format!("保存场景配置失败: {e}"))
}
