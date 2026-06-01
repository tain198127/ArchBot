//! Skill bundle installer for agent runtimes.
//!
//! Installs curated skill packs into a runtime's isolated HOME directory
//! via shallow git clones. Skills are discovered by Claude Code from
//! `$HOME/.claude/skills/` at startup.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::agent_runtime::runtime_config::{self, SkillBundle, SkillEntry};
use crate::trace_fmt;

// ── Result types ──

/// Status of a single skill installation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SkillStatus {
    Installed,
    AlreadyInstalled,
    Failed,
    Skipped,
}

/// Per-skill installation result returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInstallResult {
    pub name: String,
    pub status: SkillStatus,
    /// Git ref (tag or commit) that was checked out.
    pub version: String,
    /// Error message if status is Failed.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub error_message: String,
}

/// Summary of all skill installations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInstallSummary {
    pub results: Vec<SkillInstallResult>,
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub skipped: usize,
}

/// Information about an installed skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledSkill {
    pub name: String,
    /// Current git ref (branch or tag).
    pub r#ref: String,
    /// Last commit hash.
    pub commit: String,
    /// ISO 8601 timestamp of last update (file mtime of .git/FETCH_HEAD).
    pub last_updated: String,
}

// ── Public API ──

/// Check whether `git` is available on the system PATH.
pub fn check_git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Install all skills from a bundle into the given skills directory.
///
/// Each skill is shallow-cloned from its git repository. Failures are
/// non-blocking — a failed clone logs an error and continues to the
/// next skill.
pub fn install_skill_bundle(bundle: &SkillBundle, skills_dir: &Path) -> SkillInstallSummary {
    if !bundle.enabled {
        trace_fmt!("skill:installer", "Skill bundle disabled, skipping");
        return SkillInstallSummary {
            results: vec![],
            total: 0,
            succeeded: 0,
            failed: 0,
            skipped: 0,
        };
    }

    if bundle.skills.is_empty() {
        trace_fmt!("skill:installer", "No skills configured in bundle");
        return SkillInstallSummary {
            results: vec![],
            total: 0,
            succeeded: 0,
            failed: 0,
            skipped: 0,
        };
    }

    if !check_git_available() {
        trace_fmt!(
            "skill:installer",
            "git not found on PATH — skipping all skill installs"
        );
        let results: Vec<SkillInstallResult> = bundle
            .skills
            .iter()
            .map(|s| SkillInstallResult {
                name: s.name.clone(),
                status: SkillStatus::Skipped,
                version: String::new(),
                error_message: "git not found on PATH".to_string(),
            })
            .collect();
        let total = results.len();
        return SkillInstallSummary {
            results,
            total,
            succeeded: 0,
            failed: 0,
            skipped: total,
        };
    }

    // Ensure the skills directory exists
    if let Err(e) = fs::create_dir_all(skills_dir) {
        trace_fmt!(
            "skill:installer",
            "Failed to create skills dir {:?}: {}",
            skills_dir,
            e
        );
        let results: Vec<SkillInstallResult> = bundle
            .skills
            .iter()
            .map(|s| SkillInstallResult {
                name: s.name.clone(),
                status: SkillStatus::Failed,
                version: String::new(),
                error_message: format!("cannot create skills directory: {}", e),
            })
            .collect();
        let total = results.len();
        return SkillInstallSummary {
            results,
            total,
            succeeded: 0,
            failed: total,
            skipped: 0,
        };
    }

    let mut results = Vec::with_capacity(bundle.skills.len());

    for skill in &bundle.skills {
        let target_dir = skills_dir.join(&skill.name);
        let result = install_one_skill(skill, &target_dir);
        results.push(result);
    }

    let succeeded = results
        .iter()
        .filter(|r| r.status == SkillStatus::Installed || r.status == SkillStatus::AlreadyInstalled)
        .count();
    let failed = results
        .iter()
        .filter(|r| r.status == SkillStatus::Failed)
        .count();
    let skipped = results
        .iter()
        .filter(|r| r.status == SkillStatus::Skipped)
        .count();
    let total = results.len();

    trace_fmt!(
        "skill:installer",
        "Bundle complete: {} succeeded, {} failed, {} skipped",
        succeeded,
        failed,
        skipped
    );

    SkillInstallSummary {
        results,
        total,
        succeeded,
        failed,
        skipped,
    }
}

/// Update all installed skills by fetching the latest from their configured refs.
pub fn update_skills(bundle: &SkillBundle, skills_dir: &Path) -> SkillInstallSummary {
    if !bundle.enabled || bundle.skills.is_empty() {
        return SkillInstallSummary {
            results: vec![],
            total: 0,
            succeeded: 0,
            failed: 0,
            skipped: 0,
        };
    }

    if !check_git_available() {
        let results: Vec<SkillInstallResult> = bundle
            .skills
            .iter()
            .map(|s| SkillInstallResult {
                name: s.name.clone(),
                status: SkillStatus::Skipped,
                version: String::new(),
                error_message: "git not found on PATH".to_string(),
            })
            .collect();
        let total = results.len();
        return SkillInstallSummary {
            results,
            total,
            succeeded: 0,
            failed: 0,
            skipped: total,
        };
    }

    let mut results = Vec::with_capacity(bundle.skills.len());

    for skill in &bundle.skills {
        let target_dir = skills_dir.join(&skill.name);
        if target_dir.join(".git").exists() {
            let result = update_one_skill(skill, &target_dir);
            results.push(result);
        } else {
            // Not installed yet — install it
            let result = install_one_skill(skill, &target_dir);
            results.push(result);
        }
    }

    let succeeded = results
        .iter()
        .filter(|r| r.status == SkillStatus::Installed || r.status == SkillStatus::AlreadyInstalled)
        .count();
    let failed = results
        .iter()
        .filter(|r| r.status == SkillStatus::Failed)
        .count();
    let skipped = results
        .iter()
        .filter(|r| r.status == SkillStatus::Skipped)
        .count();
    let total = results.len();

    SkillInstallSummary {
        results,
        total,
        succeeded,
        failed,
        skipped,
    }
}

/// List all installed skills in the given directory.
pub fn list_installed_skills(skills_dir: &Path) -> Vec<InstalledSkill> {
    let mut skills = Vec::new();

    let dir = match fs::read_dir(skills_dir) {
        Ok(d) => d,
        Err(_) => return skills,
    };

    for entry in dir.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let git_dir = path.join(".git");
        if !git_dir.exists() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();

        // Read current ref from .git/HEAD
        let git_ref = read_git_head_ref(&path).unwrap_or_else(|| "unknown".to_string());

        // Read last commit hash
        let commit = read_git_rev_parse(&path).unwrap_or_else(|| "unknown".to_string());

        // Last updated from FETCH_HEAD mtime
        let last_updated = read_last_updated(&path);

        skills.push(InstalledSkill {
            name,
            r#ref: git_ref,
            commit,
            last_updated,
        });
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

// ── Internal helpers ──

/// Install a single skill by shallow-cloning its repo.
fn install_one_skill(skill: &SkillEntry, target_dir: &Path) -> SkillInstallResult {
    let name = skill.name.clone();
    let git_ref = skill.r#ref.clone();

    // Already installed — skip
    if target_dir.join(".git").exists() {
        trace_fmt!(
            "skill:installer",
            "Skill '{}' already installed, skipping",
            name
        );
        let version = read_git_rev_parse(target_dir).unwrap_or_else(|| git_ref.clone());
        return SkillInstallResult {
            name,
            status: SkillStatus::AlreadyInstalled,
            version,
            error_message: String::new(),
        };
    }

    // If target exists but isn't a git repo, remove it first
    if target_dir.exists() {
        trace_fmt!(
            "skill:installer",
            "Removing stale non-git directory for '{}'",
            name
        );
        let _ = fs::remove_dir_all(target_dir);
    }

    trace_fmt!(
        "skill:installer",
        "Cloning skill '{}' from {} (ref: {})",
        name,
        skill.repo,
        git_ref
    );

    let output = Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "--branch",
            &git_ref,
            "--single-branch",
            &skill.repo,
        ])
        .arg(target_dir.as_os_str())
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let version = read_git_rev_parse(target_dir).unwrap_or_else(|| git_ref.clone());
            trace_fmt!(
                "skill:installer",
                "Skill '{}' installed at ref {}",
                name,
                version
            );
            SkillInstallResult {
                name,
                status: SkillStatus::Installed,
                version,
                error_message: String::new(),
            }
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            let msg = if stderr.is_empty() {
                format!("git clone exited with status: {:?}", o.status.code())
            } else {
                // Trim to last meaningful line for brevity
                stderr.lines().last().unwrap_or(&stderr).to_string()
            };
            trace_fmt!("skill:installer", "FAILED to clone '{}': {}", name, msg);
            // Clean up partial clone
            let _ = fs::remove_dir_all(target_dir);
            SkillInstallResult {
                name,
                status: SkillStatus::Failed,
                version: git_ref,
                error_message: msg,
            }
        }
        Err(e) => {
            trace_fmt!(
                "skill:installer",
                "FAILED to spawn git for '{}': {}",
                name,
                e
            );
            let _ = fs::remove_dir_all(target_dir);
            SkillInstallResult {
                name,
                status: SkillStatus::Failed,
                version: git_ref,
                error_message: format!("cannot run git: {}", e),
            }
        }
    }
}

/// Update a single installed skill via git fetch + reset.
fn update_one_skill(skill: &SkillEntry, target_dir: &Path) -> SkillInstallResult {
    let name = skill.name.clone();
    let git_ref = skill.r#ref.clone();

    trace_fmt!(
        "skill:installer",
        "Updating skill '{}' to ref {}",
        name,
        git_ref
    );

    // git fetch origin <ref>
    let fetch = Command::new("git")
        .args(["-C"])
        .arg(target_dir.as_os_str())
        .args(["fetch", "origin", &git_ref, "--depth", "1"])
        .output();

    match fetch {
        Ok(o) if o.status.success() => {}
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            return SkillInstallResult {
                name,
                status: SkillStatus::Failed,
                version: git_ref,
                error_message: stderr.lines().last().unwrap_or(&stderr).to_string(),
            };
        }
        Err(e) => {
            return SkillInstallResult {
                name,
                status: SkillStatus::Failed,
                version: git_ref,
                error_message: format!("cannot run git fetch: {}", e),
            };
        }
    }

    // git reset --hard FETCH_HEAD
    let reset = Command::new("git")
        .args(["-C"])
        .arg(target_dir.as_os_str())
        .args(["reset", "--hard", "FETCH_HEAD"])
        .output();

    match reset {
        Ok(o) if o.status.success() => {
            let version = read_git_rev_parse(target_dir).unwrap_or_else(|| git_ref.clone());
            trace_fmt!("skill:installer", "Skill '{}' updated to {}", name, version);
            SkillInstallResult {
                name,
                status: SkillStatus::Installed,
                version,
                error_message: String::new(),
            }
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            SkillInstallResult {
                name,
                status: SkillStatus::Failed,
                version: git_ref,
                error_message: stderr.lines().last().unwrap_or(&stderr).to_string(),
            }
        }
        Err(e) => SkillInstallResult {
            name,
            status: SkillStatus::Failed,
            version: git_ref,
            error_message: format!("cannot run git reset: {}", e),
        },
    }
}

/// Read the symbolic ref or commit hash from a git repo's HEAD.
fn read_git_head_ref(repo_path: &Path) -> Option<String> {
    let head_path = repo_path.join(".git").join("HEAD");
    let content = fs::read_to_string(&head_path).ok()?;
    let trimmed = content.trim();
    // If it's a symref: "ref: refs/heads/main"
    if let Some(rest) = trimmed.strip_prefix("ref: ") {
        rest.rsplit('/').next().map(|s| s.to_string())
    } else {
        // Detached HEAD — return short hash
        Some(trimmed.chars().take(8).collect())
    }
}

/// Get the current HEAD commit hash (short form).
fn read_git_rev_parse(repo_path: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["-C"])
        .arg(repo_path.as_os_str())
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Read the mtime of .git/FETCH_HEAD as an ISO 8601 string.
fn read_last_updated(repo_path: &Path) -> String {
    let fetch_head = repo_path.join(".git").join("FETCH_HEAD");
    match fs::metadata(&fetch_head) {
        Ok(meta) => {
            match meta.modified() {
                Ok(time) => {
                    // Convert SystemTime to chrono DateTime
                    let duration_since_epoch = time
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default();
                    let secs = duration_since_epoch.as_secs() as i64;
                    let nsecs = duration_since_epoch.subsec_nanos();
                    chrono::DateTime::from_timestamp(secs, nsecs)
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_else(|| "unknown".to_string())
                }
                Err(_) => "unknown".to_string(),
            }
        }
        Err(_) => "unknown".to_string(),
    }
}

// ── Tauri Commands ──

/// Install the skill bundle for a given runtime.
///
/// Reads the runtime config, resolves the isolated HOME path, and
/// installs all skills from the bundle into `.claude/skills/`.
#[tauri::command]
pub fn agent_install_skills(runtime: String) -> Result<SkillInstallSummary, String> {
    let rt_config = runtime_config::load_runtimes_config()?;
    let entry = rt_config
        .runtimes
        .get(&runtime)
        .ok_or_else(|| format!("Runtime not found: {}", runtime))?;

    let bundle = match &entry.skill_bundle {
        Some(b) if b.enabled => b.clone(),
        _ => {
            return Ok(SkillInstallSummary {
                results: vec![],
                total: 0,
                succeeded: 0,
                failed: 0,
                skipped: 0,
            });
        }
    };

    let skills_dir = resolve_skills_dir(entry)?;
    Ok(install_skill_bundle(&bundle, &skills_dir))
}

/// List installed skills for a given runtime.
#[tauri::command]
pub fn agent_list_installed_skills(runtime: String) -> Result<Vec<InstalledSkill>, String> {
    let rt_config = runtime_config::load_runtimes_config()?;
    let entry = rt_config
        .runtimes
        .get(&runtime)
        .ok_or_else(|| format!("Runtime not found: {}", runtime))?;

    let skills_dir = resolve_skills_dir(entry)?;
    Ok(list_installed_skills(&skills_dir))
}

/// Update all skills for a given runtime to their configured refs.
#[tauri::command]
pub fn agent_update_skills(runtime: String) -> Result<SkillInstallSummary, String> {
    let rt_config = runtime_config::load_runtimes_config()?;
    let entry = rt_config
        .runtimes
        .get(&runtime)
        .ok_or_else(|| format!("Runtime not found: {}", runtime))?;

    let bundle = match &entry.skill_bundle {
        Some(b) if b.enabled => b.clone(),
        _ => {
            return Ok(SkillInstallSummary {
                results: vec![],
                total: 0,
                succeeded: 0,
                failed: 0,
                skipped: 0,
            });
        }
    };

    let skills_dir = resolve_skills_dir(entry)?;
    Ok(update_skills(&bundle, &skills_dir))
}

/// Resolve the skills directory for a runtime entry.
///
/// Skills are installed into `{isolated_home}/.claude/skills/`.
fn resolve_skills_dir(entry: &runtime_config::RuntimeEntry) -> Result<PathBuf, String> {
    resolve_skills_dir_inner(entry)
}

/// Public version for use by other modules (e.g., version_manager, skill_discovery).
pub fn resolve_skills_dir_inner(
    entry: &runtime_config::RuntimeEntry,
) -> Result<PathBuf, String> {
    let isolation = entry
        .execution
        .as_ref()
        .and_then(|e| e.isolation.as_ref())
        .ok_or_else(|| "[skill_installer] Missing isolation section".to_string())?;

    let home = crate::agent_runtime::runtime_config::expand_home(&isolation.isolated_home);
    Ok(PathBuf::from(home).join(".claude").join("skills"))
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_git_available() {
        // git should be available on dev machines
        assert!(check_git_available(), "git should be on PATH");
    }

    #[test]
    fn test_install_bundle_disabled() {
        let bundle = SkillBundle {
            enabled: false,
            skills: vec![SkillEntry {
                name: "test-skill".into(),
                repo: "https://example.com/repo.git".into(),
                r#ref: "main".into(),
                description: String::new(),
            }],
        };
        let tmp = std::env::temp_dir().join("archbot_test_disabled");
        let summary = install_skill_bundle(&bundle, &tmp);
        assert_eq!(summary.total, 0);
        assert_eq!(summary.succeeded, 0);
    }

    #[test]
    fn test_install_bundle_empty_skills() {
        let bundle = SkillBundle {
            enabled: true,
            skills: vec![],
        };
        let tmp = std::env::temp_dir().join("archbot_test_empty");
        let summary = install_skill_bundle(&bundle, &tmp);
        assert_eq!(summary.total, 0);
    }

    #[test]
    fn test_install_skill_already_exists() {
        let tmp = std::env::temp_dir().join("archbot_test_exists");
        let _ = fs::remove_dir_all(&tmp);
        let skill_dir = tmp.join("already-there");
        fs::create_dir_all(skill_dir.join(".git")).unwrap();

        let skill = SkillEntry {
            name: "already-there".into(),
            repo: "https://example.com/repo.git".into(),
            r#ref: "main".into(),
            description: String::new(),
        };

        let result = install_one_skill(&skill, &skill_dir);
        assert_eq!(result.status, SkillStatus::AlreadyInstalled);
        assert_eq!(result.name, "already-there");

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_install_skill_invalid_repo() {
        let tmp = std::env::temp_dir().join("archbot_test_invalid");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let skill = SkillEntry {
            name: "nonexistent".into(),
            repo: "https://example.com/does-not-exist.git".into(),
            r#ref: "main".into(),
            description: String::new(),
        };

        let result = install_one_skill(&skill, &tmp.join("nonexistent"));
        assert_eq!(result.status, SkillStatus::Failed);
        assert!(!result.error_message.is_empty());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_list_empty_skills_dir() {
        let tmp = std::env::temp_dir().join("archbot_test_list_empty");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let skills = list_installed_skills(&tmp);
        assert!(skills.is_empty());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_skill_status_serialization() {
        let result = SkillInstallResult {
            name: "test".into(),
            status: SkillStatus::Installed,
            version: "abc1234".into(),
            error_message: String::new(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("installed"));
        assert!(json.contains("abc1234"));
    }

    #[test]
    fn test_skill_summary_counts() {
        let summary = SkillInstallSummary {
            results: vec![
                SkillInstallResult {
                    name: "a".into(),
                    status: SkillStatus::Installed,
                    version: "v1".into(),
                    error_message: String::new(),
                },
                SkillInstallResult {
                    name: "b".into(),
                    status: SkillStatus::Failed,
                    version: "main".into(),
                    error_message: "network error".into(),
                },
                SkillInstallResult {
                    name: "c".into(),
                    status: SkillStatus::AlreadyInstalled,
                    version: "v2".into(),
                    error_message: String::new(),
                },
            ],
            total: 3,
            succeeded: 2,
            failed: 1,
            skipped: 0,
        };
        assert_eq!(summary.total, 3);
        assert_eq!(summary.succeeded, 2);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.skipped, 0);
    }
}
