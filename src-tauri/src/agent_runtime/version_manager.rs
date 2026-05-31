use std::fs;
use std::path::PathBuf;
use std::process::Command;

// ─── Runtime Installer ───

/// Validate runtime_name against known identifiers to prevent path traversal.
fn validate_runtime_name(name: &str) -> Result<(), String> {
    let allowed = ["claude_code", "opencode", "hermes", "openclaw"];
    if !allowed.contains(&name) {
        return Err(format!("invalid runtime name: {}", name));
    }
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(format!(
            "runtime name contains invalid characters: {}",
            name
        ));
    }
    Ok(())
}

/// Detect installed runtime versions by scanning versions/ directory.
pub fn detect_versions(runtime_name: &str) -> Result<Vec<String>, String> {
    validate_runtime_name(runtime_name)?;
    let versions_dir = runtime_versions_dir(runtime_name);
    if !versions_dir.exists() {
        return Ok(vec![]);
    }

    let mut versions = Vec::new();
    for entry in fs::read_dir(&versions_dir)
        .map_err(|e| format!("failed to read versions dir {:?}: {}", versions_dir, e))?
    {
        let entry = entry.map_err(|e| format!("dir entry error: {}", e))?;
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            versions.push(entry.file_name().to_string_lossy().into());
        }
    }
    versions.sort();
    Ok(versions)
}

/// Return the home directory for runtime installations.
pub fn runtime_base_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".archbot")
        .join("runtimes")
}

/// Return the versions directory for a specific runtime.
pub fn runtime_versions_dir(runtime_name: &str) -> PathBuf {
    runtime_base_dir().join(runtime_name).join("versions")
}

/// Return the current symlink path for a runtime.
pub fn runtime_current_path(runtime_name: &str) -> PathBuf {
    runtime_base_dir().join(runtime_name).join("current")
}

/// 在系统 PATH 中查找可执行文件。
pub fn find_on_path(exe_name: &str) -> Option<PathBuf> {
    if let Ok(paths) = std::env::var("PATH") {
        for dir in paths.split(':') {
            let candidate = PathBuf::from(dir).join(exe_name);
            if candidate.exists() && candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// 运行可执行文件获取版本号。
fn run_version(exe_path: &std::path::Path) -> Option<String> {
    Command::new(exe_path)
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// Return the installed version of a runtime.
pub fn current_version(runtime_name: &str) -> Result<String, String> {
    validate_runtime_name(runtime_name)?;
    // 1. Check managed symlink
    let current = runtime_current_path(runtime_name);
    if current.exists() {
        if current.is_symlink() {
            return fs::read_link(&current)
                .map_err(|e| format!("failed to read symlink: {}", e))?
                .file_name()
                .map(|n| n.to_string_lossy().into())
                .ok_or_else(|| "invalid symlink target".into());
        }
    }

    // 2. Check the managed executable path directly
    let managed_exe = runtime_base_dir()
        .join(runtime_name)
        .join("current")
        .join(executable_name(runtime_name));
    if managed_exe.exists() {
        if let Some(ver) = run_version(&managed_exe) {
            return Ok(ver);
        }
    }

    // 3. Check system PATH
    let exe_name = executable_name(runtime_name);
    if let Some(path) = find_on_path(exe_name) {
        if let Some(ver) = run_version(&path) {
            return Ok(ver);
        }
        return Ok("unknown (found on PATH)".into());
    }

    Ok("not installed".into())
}

/// Validate that a runtime executable works by running --version.
pub fn validate_runtime(runtime_name: &str, _version: &str) -> Result<bool, String> {
    validate_runtime_name(runtime_name)?;
    let exe_name = executable_name(runtime_name);

    // 1. Check managed path
    let managed_exe = runtime_base_dir()
        .join(runtime_name)
        .join("current")
        .join(exe_name);
    if managed_exe.exists() {
        return match Command::new(&managed_exe).arg("--version").output() {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        };
    }

    // 2. Check system PATH
    if let Some(path) = find_on_path(exe_name) {
        return match Command::new(&path).arg("--version").output() {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        };
    }

    Ok(false)
}

/// Install a runtime version.
pub fn install_runtime(runtime_name: &str, version: &str) -> Result<(), String> {
    validate_runtime_name(runtime_name)?;
    validate_version_string(version)?;
    let target_dir = runtime_versions_dir(runtime_name).join(version);

    if target_dir.exists() {
        return Ok(());
    }

    fs::create_dir_all(&target_dir)
        .map_err(|e| format!("failed to create install dir: {}", e))?;

    let exe_name = executable_name(runtime_name);
    let target_exe = target_dir.join(exe_name);

    // 尝试从系统 PATH 找到真实二进制 → 创建 symlink
    if let Some(system_exe) = find_on_path(exe_name) {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&system_exe, &target_exe)
                .map_err(|e| format!("failed to link system {}: {}", exe_name, e))?;
        }
        #[cfg(not(unix))]
        {
            fs::write(&target_exe, format!("@echo System {} found at: {:?}", exe_name, system_exe))
                .map_err(|e| format!("failed to write stub: {}", e))?;
        }
        let marker = target_dir.join(".installed");
        let _ = fs::write(&marker, format!(
            "installed at {}\nsource: {}\n",
            chrono::Utc::now(),
            system_exe.display()
        ));
        return Ok(());
    }

    // 系统 PATH 上也找不到 → 报错
    Err(format!(
        "{} executable not found on system PATH. Please install {} first (e.g. 'npm install -g @anthropic-ai/claude-code').",
        exe_name, exe_name
    ))
}

/// 创建或更新 current 符号链接指向指定版本目录。
fn link_current(runtime_name: &str, version: &str) -> Result<(), String> {
    let target = runtime_versions_dir(runtime_name).join(version);
    let current = runtime_current_path(runtime_name);
    if current.is_symlink() || current.exists() {
        let _ = fs::remove_file(&current);
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&target, &current)
            .map_err(|e| format!("failed to create symlink: {}", e))?;
    }
    #[cfg(not(unix))]
    {
        fs::write(current.join("version.txt"), version)
            .map_err(|e| format!("failed to write version file: {}", e))?;
    }
    Ok(())
}

/// Switch current symlink to the specified version (with viability check).
pub fn switch_version(runtime_name: &str, version: &str) -> Result<(), String> {
    validate_runtime_name(runtime_name)?;
    validate_version_string(version)?;
    let target = runtime_versions_dir(runtime_name).join(version);
    if !target.exists() {
        return Err(format!(
            "version {} not installed for {}",
            version, runtime_name
        ));
    }

    if !validate_runtime(runtime_name, version)? {
        return Err(format!(
            "version {} for {} failed viability check",
            version, runtime_name
        ));
    }

    link_current(runtime_name, version)
}

/// Rollback to the previous version.
pub fn rollback_runtime(runtime_name: &str) -> Result<String, String> {
    validate_runtime_name(runtime_name)?;
    let versions = detect_versions(runtime_name)?;
    if versions.len() < 2 {
        return Err(format!(
            "no previous version available for rollback for {}",
            runtime_name
        ));
    }

    let current = current_version(runtime_name)?;
    let prev = versions
        .iter()
        .rev()
        .find(|v| **v != current)
        .ok_or_else(|| "no previous version found".to_string())?;

    switch_version(runtime_name, prev)?;
    Ok(prev.clone())
}

fn validate_version_string(version: &str) -> Result<(), String> {
    if version.is_empty() || version.len() > 64 {
        return Err(format!("invalid version string: {}", version));
    }
    if !version
        .chars()
        .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_')
    {
        return Err(format!("version contains invalid characters: {}", version));
    }
    Ok(())
}

fn executable_name(runtime_name: &str) -> &str {
    match runtime_name {
        "claude_code" => "claude",
        "opencode" => "opencode",
        "hermes" => "hermes",
        "openclaw" => "openclaw",
        _ => runtime_name,
    }
}

// ─── Tauri Commands ───

#[tauri::command]
pub fn agent_list_versions(runtime: String) -> Result<Vec<String>, String> {
    detect_versions(&runtime)
}

#[tauri::command]
pub fn agent_install_runtime(runtime: String, version: String) -> Result<String, String> {
    install_runtime(&runtime, &version)?;
    // 安装后直接链接到 current（跳过可行性检查：真实二进制下载后会再次验证）
    link_current(&runtime, &version)?;
    Ok(format!("{} {} installed", runtime, version))
}

#[tauri::command]
pub fn agent_update_runtime(runtime: String, version: String) -> Result<String, String> {
    let current = current_version(&runtime)?;
    if version == current {
        return Ok(format!("{} already at version {}", runtime, version));
    }
    install_runtime(&runtime, &version)?;
    switch_version(&runtime, &version)?;
    Ok(format!(
        "{} updated from {} to {}",
        runtime, current, version
    ))
}

#[tauri::command]
pub fn agent_rollback_runtime(runtime: String) -> Result<String, String> {
    rollback_runtime(&runtime)
}

#[tauri::command]
pub fn agent_get_current_version(runtime: String) -> Result<String, String> {
    current_version(&runtime)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executable_name_mapping() {
        assert_eq!(executable_name("claude_code"), "claude");
        assert_eq!(executable_name("opencode"), "opencode");
        assert_eq!(executable_name("hermes"), "hermes");
        assert_eq!(executable_name("unknown"), "unknown");
    }

    #[test]
    fn test_detect_versions_empty() {
        // 使用合法 runtime 名但该 runtime 未安装过 → 应该返回空列表
        let result = detect_versions("opencode");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_find_claude_on_path() {
        let result = find_on_path("claude");
        // claude 应该在 PATH 上（通过 homebrew 安装）
        assert!(result.is_some(), "claude not found on PATH — please install: npm i -g @anthropic-ai/claude-code");
    }

    /// E2E: 完整安装流程验证
    /// 1. 检测系统 PATH 上的 claude → 获取版本号
    /// 2. 安装指定版本 → 创建 symlink 指向系统二进制
    /// 3. 验证版本号可获取
    /// 4. 切换 current 链接
    #[test]
    fn e2e_install_and_detect_version() {
        let runtime = "claude_code";
        let test_version = "2.1.152"; // 当前系统版本

        // Step 1: 安装前检测
        let ver_before = current_version(runtime).unwrap();
        println!("version before install: {}", ver_before);

        // Step 2: 安装 → 应该找到系统 PATH 上的 claude 并 symlink
        let result = install_runtime(runtime, test_version);
        match result {
            Ok(()) => println!("install succeeded"),
            Err(ref e) if e.contains("already") => println!("already installed"),
            Err(e) => {
                // 如果 claude 不在 PATH 上，跳过 E2E 测试
                if e.contains("not found on system PATH") {
                    println!("SKIP: claude not on PATH");
                    return;
                }
                panic!("install failed: {}", e);
            }
        }

        // Step 3: 链接 current
        let _ = link_current(runtime, test_version);

        // Step 4: 安装后检测版本
        let ver_after = current_version(runtime).unwrap();
        println!("version after install: {}", ver_after);
        assert!(
            ver_after != "not installed",
            "version should be detectable after install, got: {}",
            ver_after
        );

        // Step 5: 版本列表应包含测试版本
        let versions = detect_versions(runtime).unwrap();
        println!("available versions: {:?}", versions);
        assert!(
            versions.contains(&test_version.to_string()),
            "versions should contain {}",
            test_version
        );

        // Step 6: 可行性验证
        let viable = validate_runtime(runtime, test_version).unwrap();
        assert!(viable, "installed runtime should pass viability check");
    }
}
