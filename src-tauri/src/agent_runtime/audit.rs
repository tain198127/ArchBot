use std::path::{Path, PathBuf};

/// An audit log entry for the session_manager API.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub log_id: String,
    pub action: String,
    pub severity: AuditSeverity,
    pub detail: String,
    pub created_at: String,
}

/// 审计违规严重级别
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AuditSeverity {
    /// 读到用户凭据（~/.ssh, ~/.aws）→ 立即终止 Turn
    Critical,
    /// 读到其他 Runtime 配置（~/.claude, ~/.hermes）→ 完成 Turn + 告警
    High,
    /// 读到用户个人数据（~/.config, ~/.gitconfig）→ 仅记录
    Medium,
    /// 告警
    Warning,
    /// 信息性记录
    Info,
    /// 读到无关系统文件 → 忽略
    Low,
}

/// 一条审计规则：禁止访问某个路径前缀
#[derive(Debug, Clone)]
pub struct AuditRule {
    pub path: String,
    pub severity: AuditSeverity,
}

/// 违规记录
#[derive(Debug, Clone)]
pub struct AuditViolation {
    pub rule: AuditRule,
    pub accessed_path: String,
}

/// Turn 审计管理器
pub struct AuditManager {
    rules: Vec<AuditRule>,
}

impl AuditManager {
    /// 创建审计管理器，加载默认规则。
    ///
    /// 规则对齐架构结论 §32（禁止访问范围）。
    pub fn new() -> Self {
        Self {
            rules: Self::default_rules(),
        }
    }

    fn default_rules() -> Vec<AuditRule> {
        vec![
            // Critical — 凭据类
            AuditRule::new("~/.ssh", AuditSeverity::Critical),
            AuditRule::new("~/.aws", AuditSeverity::Critical),
            AuditRule::new("/etc/shadow", AuditSeverity::Critical),
            // High — 其他 Runtime 配置
            AuditRule::new("~/.claude", AuditSeverity::High),
            AuditRule::new("~/.hermes", AuditSeverity::High),
            AuditRule::new("~/.opencode", AuditSeverity::High),
            AuditRule::new("~/.openclaw", AuditSeverity::High),
            // Medium — 用户个人数据
            AuditRule::new("~/.config", AuditSeverity::Medium),
            AuditRule::new("~/.gitconfig", AuditSeverity::Medium),
            AuditRule::new("~/.npmrc", AuditSeverity::Medium),
            AuditRule::new("~/.zshrc", AuditSeverity::Medium),
            AuditRule::new("~/.bashrc", AuditSeverity::Medium),
            AuditRule::new("~/.profile", AuditSeverity::Medium),
            AuditRule::new("/etc/passwd", AuditSeverity::Medium),
        ]
    }

    /// 扫描文件访问列表，返回所有违规。
    ///
    /// `accessed_paths` 是 Turn 期间 Runtime 进程访问过的文件路径列表。
    /// 第一版由调用方通过 proc 扫描或事后 find 收集这些路径。
    pub fn audit(&self, accessed_paths: &[String]) -> Vec<AuditViolation> {
        let mut violations = Vec::new();

        for accessed in accessed_paths {
            // canonicalize 解析符号链接和 .. 组件，防止绕过
            let resolved = canonicalize_or_resolve(accessed);
            for rule in &self.rules {
                let rule_path = canonicalize_or_resolve(&resolve_home(&rule.path));
                if resolved.starts_with(&rule_path) {
                    violations.push(AuditViolation {
                        rule: rule.clone(),
                        accessed_path: accessed.clone(),
                    });
                    break;
                }
            }
        }

        violations
    }
}

impl Default for AuditManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditRule {
    pub fn new(path: impl Into<String>, severity: AuditSeverity) -> Self {
        Self {
            path: path.into(),
            severity,
        }
    }
}

/// 将 ~/ 开头的路径展开到实际 HOME 目录
fn resolve_home(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest).to_string_lossy().to_string();
        }
    }
    path.to_string()
}

/// 对路径做 canonicalize（解析 symlink / ..），失败时降级为 resolve_home
fn canonicalize_or_resolve(path: &str) -> PathBuf {
    let resolved = resolve_home(path);
    Path::new(&resolved)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(resolved))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_critical_violation() {
        let manager = AuditManager::new();
        let home = dirs::home_dir().unwrap();
        let violations = manager.audit(&[home.join(".ssh/id_rsa").to_string_lossy().to_string()]);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule.severity, AuditSeverity::Critical);
    }

    #[test]
    fn detects_high_violation() {
        let manager = AuditManager::new();
        let home = dirs::home_dir().unwrap();
        let violations = manager.audit(&[home
            .join(".claude/settings.json")
            .to_string_lossy()
            .to_string()]);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule.severity, AuditSeverity::High);
    }

    #[test]
    fn allows_safe_path() {
        let manager = AuditManager::new();
        let violations = manager.audit(&["/tmp/archbot_test.txt".into()]);
        assert!(violations.is_empty());
    }
}
