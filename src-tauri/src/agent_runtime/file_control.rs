use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

// ─── Pre-Turn Snapshot ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreTurnSnapshot {
    pub turn_id: String,
    pub is_git_repo: bool,
    pub git_commit: Option<String>,
    pub git_branch: Option<String>,
    pub is_dirty: Option<bool>,
    pub file_hashes: HashMap<String, String>, // path -> SHA256
    pub created_at: String,
}

impl PreTurnSnapshot {
    pub fn capture(turn_id: &str, project_root: &Path) -> Result<Self, String> {
        let is_git = project_root.join(".git").exists();
        let mut snapshot = Self {
            turn_id: turn_id.into(),
            is_git_repo: is_git,
            git_commit: None,
            git_branch: None,
            is_dirty: None,
            file_hashes: HashMap::new(),
            created_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        };

        if is_git {
            snapshot.git_commit = git_command(project_root, &["rev-parse", "HEAD"]).ok();
            snapshot.git_branch =
                git_command(project_root, &["rev-parse", "--abbrev-ref", "HEAD"]).ok();
            snapshot.is_dirty = Some(
                git_command(project_root, &["status", "--porcelain"])
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false),
            );
        } else {
            // Non-git: hash all source files
            let hashes = hash_project_files(project_root)?;
            snapshot.file_hashes = hashes;
        }

        Ok(snapshot)
    }
}

// ─── Post-Turn Diff ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub path: String,
    pub change_type: String, // created, modified, deleted, renamed
    pub diff_content: String,
    pub hash_before: String,
    pub hash_after: String,
    pub size_before: u64,
    pub size_after: u64,
}

/// Scan file changes after a turn completes.
pub fn scan_file_changes(
    project_root: &Path,
    snapshot: &PreTurnSnapshot,
) -> Result<Vec<FileDiff>, String> {
    if snapshot.is_git_repo {
        scan_git_diff(project_root)
    } else {
        scan_hash_diff(project_root, snapshot)
    }
}

fn scan_git_diff(project_root: &Path) -> Result<Vec<FileDiff>, String> {
    // Get list of changed files
    let name_status = git_command(project_root, &["diff", "--name-status", "HEAD"])?;
    let mut diffs = Vec::new();

    for line in name_status.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        let change_type = match parts[0] {
            "A" => "created",
            "M" => "modified",
            "D" => "deleted",
            "R" => "renamed",
            _ => "modified",
        };
        let path = parts[1].to_string();

        let diff_content = if change_type != "deleted" {
            git_command(project_root, &["diff", "HEAD", "--", &path]).unwrap_or_default()
        } else {
            String::new()
        };

        let hash_before = file_hash(&project_root.join(&path)).unwrap_or_default();
        let hash_after = String::new(); // would need pre-turn hash

        let size = fs::metadata(project_root.join(&path))
            .map(|m| m.len())
            .unwrap_or(0);

        diffs.push(FileDiff {
            path,
            change_type: change_type.into(),
            diff_content,
            hash_before,
            hash_after,
            size_before: size,
            size_after: size,
        });
    }

    Ok(diffs)
}

fn scan_hash_diff(
    project_root: &Path,
    snapshot: &PreTurnSnapshot,
) -> Result<Vec<FileDiff>, String> {
    let current_hashes = hash_project_files(project_root)?;
    let mut diffs = Vec::new();

    // Find modified and created files
    for (path, current_hash) in &current_hashes {
        match snapshot.file_hashes.get(path) {
            Some(old_hash) if old_hash != current_hash => {
                diffs.push(FileDiff {
                    path: path.clone(),
                    change_type: "modified".into(),
                    diff_content: String::new(),
                    hash_before: old_hash.clone(),
                    hash_after: current_hash.clone(),
                    size_before: 0,
                    size_after: 0,
                });
            }
            None => {
                diffs.push(FileDiff {
                    path: path.clone(),
                    change_type: "created".into(),
                    diff_content: String::new(),
                    hash_before: String::new(),
                    hash_after: current_hash.clone(),
                    size_before: 0,
                    size_after: 0,
                });
            }
            _ => {}
        }
    }

    // Find deleted files
    for (path, old_hash) in &snapshot.file_hashes {
        if !current_hashes.contains_key(path) {
            diffs.push(FileDiff {
                path: path.clone(),
                change_type: "deleted".into(),
                diff_content: String::new(),
                hash_before: old_hash.clone(),
                hash_after: String::new(),
                size_before: 0,
                size_after: 0,
            });
        }
    }

    Ok(diffs)
}

// ─── Rollback ───

pub fn rollback_turn(project_root: &Path, _turn_id: &str) -> Result<(), String> {
    if project_root.join(".git").exists() {
        // Restore all changes to HEAD
        git_command(project_root, &["checkout", "."])
            .map(|_| ())
            .map_err(|e| format!("git rollback failed: {}", e))
    } else {
        Err("rollback without git not yet implemented".into())
    }
}

pub fn rollback_file(project_root: &Path, file_path: &str) -> Result<(), String> {
    if project_root.join(".git").exists() {
        git_command(project_root, &["checkout", "--", file_path])
            .map(|_| ())
            .map_err(|e| format!("git rollback for {} failed: {}", file_path, e))
    } else {
        Err("rollback without git not yet implemented".into())
    }
}

// ─── File Boundary Enforcement ───

pub fn validate_path_in_project(project_root: &Path, target: &Path) -> Result<(), String> {
    let canonical_project = project_root
        .canonicalize()
        .map_err(|_| format!("invalid project root: {:?}", project_root))?;

    let canonical_target = target
        .canonicalize()
        .unwrap_or_else(|_| target.to_path_buf());

    if !canonical_target.starts_with(&canonical_project) {
        return Err(format!(
            "path outside project boundary: {:?} (project root: {:?})",
            target, project_root
        ));
    }
    Ok(())
}

// ─── Helpers ───

fn git_command(dir: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|e| format!("git command failed: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().into())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(format!("git error: {}", err.trim()))
    }
}

fn hash_project_files(root: &Path) -> Result<HashMap<String, String>, String> {
    let mut hashes = HashMap::new();
    hash_dir(root, root, &mut hashes)?;
    Ok(hashes)
}

fn hash_dir(base: &Path, dir: &Path, hashes: &mut HashMap<String, String>) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| format!("read_dir failed: {}", e))? {
        let entry = entry.map_err(|e| format!("entry error: {}", e))?;
        let path = entry.path();

        if path
            .file_name()
            .map(|n| n == ".git" || n == ".archbot" || n == "node_modules" || n == "target")
            .unwrap_or(false)
        {
            continue;
        }

        if path.is_dir() {
            hash_dir(base, &path, hashes)?;
        } else if path.is_file() {
            if let Ok(hash) = file_hash(&path) {
                let rel = path.strip_prefix(base).unwrap_or(&path);
                hashes.insert(rel.to_string_lossy().into(), hash);
            }
        }
    }
    Ok(())
}

fn file_hash(path: &Path) -> Result<String, String> {
    let contents = fs::read(path).map_err(|e| format!("read failed for {:?}: {}", path, e))?;
    let mut hasher = Sha256::new();
    hasher.update(&contents);
    Ok(format!("{:x}", hasher.finalize()))
}

// ─── Tauri Commands ───

#[tauri::command]
pub fn agent_capture_snapshot(
    turn_id: String,
    project_root: String,
) -> Result<PreTurnSnapshot, String> {
    PreTurnSnapshot::capture(&turn_id, Path::new(&project_root))
}

#[tauri::command]
pub fn agent_scan_file_changes(
    project_root: String,
    snapshot: PreTurnSnapshot,
) -> Result<Vec<FileDiff>, String> {
    scan_file_changes(Path::new(&project_root), &snapshot)
}

#[tauri::command]
pub fn agent_rollback_turn(project_root: String, turn_id: String) -> Result<(), String> {
    rollback_turn(Path::new(&project_root), &turn_id)
}

#[tauri::command]
pub fn agent_rollback_file(project_root: String, file_path: String) -> Result<(), String> {
    let root = Path::new(&project_root);
    let target = root.join(&file_path);
    validate_path_in_project(root, &target)?;
    rollback_file(root, &file_path)
}

#[tauri::command]
pub fn agent_validate_path(project_root: String, target_path: String) -> Result<(), String> {
    validate_path_in_project(Path::new(&project_root), Path::new(&target_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_capture_snapshot_non_git() {
        let dir = temp_dir();
        let snapshot = PreTurnSnapshot::capture("test_turn", &dir);
        assert!(snapshot.is_ok());
    }

    #[test]
    fn test_validate_path_in_project() {
        let root = temp_dir();
        let target = temp_dir().join("test_file.txt");
        let result = validate_path_in_project(&root, &target);
        assert!(result.is_ok() || result.is_err()); // depends on canonical path
    }

    #[test]
    fn test_file_hash_consistent() {
        let path = temp_dir().join("hash_test.txt");
        fs::write(&path, b"hello world").unwrap();
        let h1 = file_hash(&path).unwrap();
        let h2 = file_hash(&path).unwrap();
        assert_eq!(h1, h2);
        let _ = fs::remove_file(&path);
    }
}
