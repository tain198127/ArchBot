use std::fs;
use std::path::Path;

use crate::agent_runtime::config::IsolatedHomeConfig;

/// 初始化隔离 HOME 目录。
///
/// 在 Runtime 首次启动前调用，准备：
/// - 隔离 HOME 目录本身
/// - 按需生成 `.gitconfig`（使用 ArchBot 配置的身份，不桥接用户真实 git 身份）
/// - 按需部署 SSH deploy key（从 ArchBot Secret Manager 读取）
pub fn setup_isolated_home(config: &IsolatedHomeConfig) -> Result<(), String> {
    // 1. 创建隔离 HOME 目录
    fs::create_dir_all(&config.home_path)
        .map_err(|e| format!("Failed to create isolated HOME {:?}: {}", config.home_path, e))?;

    // 2. 如果需要 git，生成 .gitconfig
    if config.needs_git {
        let git_name = config.git_user_name.as_deref().unwrap_or("ArchBot");
        let git_email = config
            .git_user_email
            .as_deref()
            .unwrap_or("archbot@local");

        let gitconfig = format!(
            "[user]\n  name = {}\n  email = {}\n",
            git_name, git_email
        );
        fs::write(config.home_path.join(".gitconfig"), &gitconfig).map_err(|e| {
            format!(
                "Failed to write .gitconfig in {:?}: {}",
                config.home_path, e
            )
        })?;
    }

    // 3. 如果需要 SSH，从 ArchBot Secret Manager 获取 deploy key
    if config.needs_ssh {
        if let Some(key_path) = &config.ssh_key_path {
            if !Path::new(key_path).exists() {
                return Err(format!(
                    "[home_setup] SSH key not found: {:?}",
                    key_path
                ));
            }

            let ssh_dir = config.home_path.join(".ssh");
            fs::create_dir_all(&ssh_dir)
                .map_err(|e| format!("Failed to create .ssh dir {:?}: {}", ssh_dir, e))?;

            let dest = ssh_dir.join("id_ed25519");
            fs::copy(key_path, &dest).map_err(|e| {
                format!(
                    "Failed to copy SSH key from {:?} to {:?}: {}",
                    key_path, dest, e
                )
            })?;

            // 设置权限 600
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&dest, fs::Permissions::from_mode(0o600)).map_err(
                    |e| format!("Failed to set SSH key permissions on {:?}: {}", dest, e),
                )?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn creates_home_directory() {
        let tmp = std::env::temp_dir().join("archbot_test_home_setup");
        let _ = fs::remove_dir_all(&tmp);

        let config = IsolatedHomeConfig {
            home_path: tmp.clone(),
            needs_git: false,
            git_user_name: None,
            git_user_email: None,
            needs_ssh: false,
            ssh_key_path: None,
        };

        let result = setup_isolated_home(&config);
        assert!(result.is_ok());
        assert!(tmp.exists());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn generates_gitconfig_when_needed() {
        let tmp = std::env::temp_dir().join("archbot_test_home_git");
        let _ = fs::remove_dir_all(&tmp);

        let config = IsolatedHomeConfig {
            home_path: tmp.clone(),
            needs_git: true,
            git_user_name: Some("TestUser".into()),
            git_user_email: Some("test@example.com".into()),
            needs_ssh: false,
            ssh_key_path: None,
        };

        setup_isolated_home(&config).unwrap();

        let gitconfig = tmp.join(".gitconfig");
        assert!(gitconfig.exists());
        let content = fs::read_to_string(&gitconfig).unwrap();
        assert!(content.contains("TestUser"));
        assert!(content.contains("test@example.com"));

        let _ = fs::remove_dir_all(&tmp);
    }
}
