use std::collections::HashMap;

/// Runtime 隔离启动配置
pub struct RuntimeLaunchConfig {
    /// Runtime 类型标识：claude_code / hermes / opencode / openclaw
    pub runtime_type: String,
    /// 可执行文件路径（ArchBot 托管版本）
    pub executable: String,
    /// 工作目录（项目根目录）
    pub workspace_root: String,
    /// ArchBot 管理的隔离 HOME
    pub isolated_home: String,
    /// 显式允许注入的环境变量（白名单）
    pub allowed_env: HashMap<String, String>,
    /// 启动参数
    pub args: Vec<String>,
    /// 超时秒数
    pub timeout_seconds: u64,
}

/// 隔离 HOME 初始化配置
pub struct IsolatedHomeConfig {
    pub home_path: std::path::PathBuf,
    pub needs_git: bool,
    pub git_user_name: Option<String>,
    pub git_user_email: Option<String>,
    pub needs_ssh: bool,
    pub ssh_key_path: Option<std::path::PathBuf>,
}
