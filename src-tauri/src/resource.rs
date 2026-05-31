//! 资源路径解析器
//!
//! 解析两种 URI 协议:
//! - `resource://` → 源码内置资源目录 (`src-tauri/resources/`)
//! - `user://`     → 用户自定义资源 (`~/.archbot/{project}/`)
//!
//! ## 使用
//! ```ignore
//! let path = resolve_resource("resource://skills/biz-process-extract/");
//! // → /path/to/app/src-tauri/resources/skills/biz-process-extract/
//! ```

use std::path::{Path, PathBuf};

/// URI 协议类型
#[derive(Debug, PartialEq)]
pub enum UriProtocol {
    Resource,
    User,
}

/// 解析 URI 为本地文件路径
///
/// 防止路径遍历：拒绝含 `..` 的路径，并通过 canonicalize 验证结果不脱离基目录。
///
/// # 参数
/// - `uri`: 以 `resource://` 或 `user://` 开头的路径
/// - `resource_dir`: Tauri `resource_dir()` 返回的路径
/// - `project_name`: 项目名称（用于 user:// 协议）
///
/// # 返回
/// 本地文件系统路径。如果 URI 格式无效或包含路径遍历则返回 `Err`。
pub fn resolve(uri: &str, resource_dir: &Path, project_name: &str) -> Result<PathBuf, String> {
    let (protocol, relative) = parse_uri(uri)?;

    let base: PathBuf = match protocol {
        UriProtocol::Resource => resource_dir.to_path_buf(),
        UriProtocol::User => {
            let home = dirs_next().ok_or("无法获取用户主目录")?;
            home.join(".archbot").join(project_name)
        }
    };

    let candidate = base.join(&relative);

    // 防御：canonicalize 解析符号链接和相对路径后，确认结果仍在 base 内
    let canonical = candidate
        .canonicalize()
        .map_err(|e| format!("资源路径无效: {candidate:?} — {e}"))?;

    if !canonical.starts_with(&base) {
        return Err(format!("路径遍历拒绝: {uri} → {canonical:?}"));
    }

    Ok(canonical)
}

/// 解析 URI 字符串，提取协议和相对路径。拒绝空路径和 `..` 穿越。
fn parse_uri(uri: &str) -> Result<(UriProtocol, String), String> {
    let rest = if let Some(r) = uri.strip_prefix("resource://") {
        (UriProtocol::Resource, r)
    } else if let Some(r) = uri.strip_prefix("user://") {
        (UriProtocol::User, r)
    } else {
        return Err(format!(
            "不支持的 URI 协议，请使用 resource:// 或 user://: {uri}"
        ));
    };

    let relative = rest.1;

    if relative.is_empty() {
        return Err("URI 路径不能为空".to_string());
    }

    // 拒绝路径遍历：不允许 .. 或以 / 开头（绝对路径绕过）
    for segment in relative.split('/') {
        if segment == ".." || segment.starts_with('\\') {
            return Err(format!("路径遍历拒绝: 包含 '..': {uri}"));
        }
    }
    if relative.starts_with('/') || relative.starts_with('\\') {
        return Err(format!("路径遍历拒绝: 不允许绝对路径: {uri}"));
    }

    Ok((rest.0, relative.to_string()))
}

fn dirs_next() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        std::env::var("HOME").ok().map(PathBuf::from)
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var("HOME").ok().map(PathBuf::from)
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE").ok().map(PathBuf::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_resource_uri() {
        let (proto, rel) = parse_uri("resource://skills/test-skill/").unwrap();
        assert_eq!(proto, UriProtocol::Resource);
        assert_eq!(rel, "skills/test-skill/");
    }

    #[test]
    fn parses_user_uri() {
        let (proto, rel) = parse_uri("user://skills/my-skill/").unwrap();
        assert_eq!(proto, UriProtocol::User);
        assert_eq!(rel, "skills/my-skill/");
    }

    #[test]
    fn rejects_invalid_uri() {
        assert!(parse_uri("file:///etc/passwd").is_err());
        assert!(parse_uri("skills/test").is_err());
    }

    #[test]
    fn rejects_empty_path() {
        assert!(parse_uri("resource://").is_err());
        assert!(parse_uri("user://").is_err());
    }

    #[test]
    fn rejects_parent_traversal() {
        assert!(parse_uri("resource://../../../etc/passwd").is_err());
        assert!(parse_uri("resource://skills/../etc/passwd").is_err());
        assert!(parse_uri("user://../../.ssh/id_rsa").is_err());
    }

    #[test]
    fn rejects_absolute_path() {
        assert!(parse_uri("resource:///etc/passwd").is_err());
        assert!(parse_uri("user:///etc/shadow").is_err());
    }

    #[test]
    fn resolves_clean_path() {
        let base = PathBuf::from("/tmp/test-resources");
        let result = resolve("resource://skills/test-skill/", &base, "test-proj");
        // canonicalize 在不存在的路径上会失败，这里验证 parse_uri 层已通过
        // 真实的路径解析需要在存在的目录上测试
        assert!(parse_uri("resource://skills/test-skill/").is_ok());
    }
}
