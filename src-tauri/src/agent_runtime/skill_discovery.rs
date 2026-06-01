//! Skill command discovery from installed skill packages.
//!
//! Walks `{skills_dir}/{package}/skills/*/SKILL.md` to enumerate
//! individual slash commands available in each installed package.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::agent_runtime::runtime_config;
use crate::agent_runtime::skill_installer;

/// A single skill command discovered from an installed skill package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCommand {
    /// Parent skill package name (e.g., "superpowers", "gstack")
    pub package: String,
    /// Skill directory name (e.g., "brainstorming", "qa")
    pub skill_name: String,
    /// The slash command (e.g., "/brainstorming", "/qa")
    pub command: String,
    /// English display name from SKILL.md or fallback
    pub display_name_en: String,
}

/// Enumerate all skill commands from installed skill packages for a runtime.
pub fn list_skill_commands(runtime_name: &str) -> Result<Vec<SkillCommand>, String> {
    let rt_config = runtime_config::load_runtimes_config()?;
    let entry = rt_config
        .runtimes
        .get(runtime_name)
        .ok_or_else(|| format!("Runtime not found: {}", runtime_name))?;

    let skills_dir = skill_installer::resolve_skills_dir_inner(entry)?;
    discover_commands(&skills_dir)
}

/// Discover commands by walking the skills directory.
fn discover_commands(skills_dir: &Path) -> Result<Vec<SkillCommand>, String> {
    let mut commands = Vec::new();

    let dir = match fs::read_dir(skills_dir) {
        Ok(d) => d,
        Err(_) => return Ok(commands), // No skills installed yet — empty
    };

    for entry in dir.flatten() {
        let package_path = entry.path();
        if !package_path.is_dir() {
            continue;
        }
        let package_name = entry.file_name().to_string_lossy().to_string();

        // Look for skills/ subdirectory inside the package
        let skills_subdir = package_path.join("skills");
        if !skills_subdir.is_dir() {
            continue;
        }

        let skills_dir_entries = match fs::read_dir(&skills_subdir) {
            Ok(d) => d,
            Err(_) => continue,
        };

        for skill_entry in skills_dir_entries.flatten() {
            let skill_path = skill_entry.path();
            if !skill_path.is_dir() {
                continue;
            }
            let skill_name = skill_entry.file_name().to_string_lossy().to_string();

            // Try to read SKILL.md for metadata
            let skill_md = skill_path.join("SKILL.md");
            let (display_name, command) = if skill_md.exists() {
                parse_skill_md(&skill_md, &skill_name)
            } else {
                // Fallback: derive from directory name
                (skill_name_to_display(&skill_name), format!("/{}", skill_name))
            };

            commands.push(SkillCommand {
                package: package_name.clone(),
                skill_name,
                command,
                display_name_en: display_name,
            });
        }
    }

    commands.sort_by(|a, b| {
        a.package
            .cmp(&b.package)
            .then_with(|| a.skill_name.cmp(&b.skill_name))
    });

    Ok(commands)
}

/// Parse SKILL.md frontmatter to extract display name and command trigger.
fn parse_skill_md(path: &Path, fallback_name: &str) -> (String, String) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            return (
                skill_name_to_display(fallback_name),
                format!("/{}", fallback_name),
            );
        }
    };

    // Extract frontmatter (between --- markers)
    let display_name = if let Some(fm) = extract_frontmatter(&content) {
        fm.get("name")
            .cloned()
            .unwrap_or_else(|| skill_name_to_display(fallback_name))
    } else {
        skill_name_to_display(fallback_name)
    };

    // Find first slash command pattern in the content
    let command = find_first_command(&content, fallback_name);

    (display_name, command)
}

/// Extract YAML frontmatter as key-value pairs.
fn extract_frontmatter(content: &str) -> Option<std::collections::HashMap<String, String>> {
    let mut lines = content.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    let mut map = std::collections::HashMap::new();
    for line in lines.by_ref() {
        let trimmed = line.trim();
        if trimmed == "---" {
            break;
        }
        if let Some((key, value)) = trimmed.split_once(':') {
            map.insert(
                key.trim().to_string(),
                value.trim().trim_matches('"').trim_matches('\'').to_string(),
            );
        }
    }
    Some(map)
}

/// Find the first slash command reference in the content.
fn find_first_command(content: &str, fallback_name: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(start) = trimmed.find('/') {
            // Skip past the '/' and collect the command name
            let after_slash = &trimmed[start + 1..];
            let cmd: String = after_slash
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == ':')
                .collect();
            if !cmd.is_empty() {
                return format!("/{}", cmd);
            }
        }
    }
    format!("/{}", fallback_name)
}

/// Convert a kebab-case directory name to a Title Case display name.
fn skill_name_to_display(name: &str) -> String {
    name.split('-')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Tauri command: list all discovered skill commands for a runtime.
#[tauri::command]
pub fn agent_list_skill_commands(runtime: String) -> Result<Vec<SkillCommand>, String> {
    list_skill_commands(&runtime)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_name_to_display() {
        assert_eq!(skill_name_to_display("brainstorming"), "Brainstorming");
        assert_eq!(
            skill_name_to_display("test-driven-development"),
            "Test Driven Development"
        );
        assert_eq!(skill_name_to_display("qa"), "Qa");
    }

    #[test]
    fn test_find_first_command() {
        let content = "# Brainstorming\n\nUsage: /brainstorming to start";
        assert_eq!(
            find_first_command(content, "fallback"),
            "/brainstorming"
        );
    }

    #[test]
    fn test_find_first_command_fallback() {
        let content = "No commands here";
        assert_eq!(find_first_command(content, "my-skill"), "/my-skill");
    }

    #[test]
    fn test_extract_frontmatter() {
        let content = "---\nname: Test Skill\ndescription: A test\n---\nBody";
        let fm = extract_frontmatter(content).unwrap();
        assert_eq!(fm.get("name").unwrap(), "Test Skill");
        assert_eq!(fm.get("description").unwrap(), "A test");
    }

    #[test]
    fn test_extract_frontmatter_no_delimiters() {
        let content = "Just text, no frontmatter";
        assert!(extract_frontmatter(content).is_none());
    }

    #[test]
    fn test_discover_empty_skills_dir() {
        let tmp = std::env::temp_dir().join("archbot_test_discover_empty");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let commands = discover_commands(&tmp).unwrap();
        assert!(commands.is_empty());
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_discover_with_skill_dirs() {
        let tmp = std::env::temp_dir().join("archbot_test_discover_skills");
        let _ = fs::remove_dir_all(&tmp);

        let skill_dir = tmp
            .join("superpowers")
            .join("skills")
            .join("brainstorming");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: Brainstorming\n---\n\nUse /brainstorming to start.\n",
        )
        .unwrap();

        let tdd_dir = tmp.join("superpowers").join("skills").join("test-driven-development");
        fs::create_dir_all(&tdd_dir).unwrap();
        fs::write(
            tdd_dir.join("SKILL.md"),
            "---\nname: Test Driven Development\n---\n\nRun /test-driven-development first.\n",
        )
        .unwrap();

        let commands = discover_commands(&tmp).unwrap();
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].package, "superpowers");
        assert_eq!(commands[0].skill_name, "brainstorming");
        assert_eq!(commands[0].command, "/brainstorming");
        assert_eq!(commands[0].display_name_en, "Brainstorming");

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_discover_skill_without_skill_md() {
        let tmp = std::env::temp_dir().join("archbot_test_discover_no_md");
        let _ = fs::remove_dir_all(&tmp);

        let skill_dir = tmp.join("gstack").join("skills").join("browse");
        fs::create_dir_all(&skill_dir).unwrap();

        let commands = discover_commands(&tmp).unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].skill_name, "browse");
        assert_eq!(commands[0].display_name_en, "Browse");
        assert_eq!(commands[0].command, "/browse");

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_parse_skill_md_no_command_in_body() {
        let tmp = std::env::temp_dir().join("archbot_test_parse_md");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let md = tmp.join("test.md");
        fs::write(&md, "---\nname: My Skill\n---\n\nSome description").unwrap();
        let (name, cmd) = parse_skill_md(&md, "my-skill");
        assert_eq!(name, "My Skill");
        assert_eq!(cmd, "/my-skill");
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_parse_skill_md_missing_file() {
        let path = PathBuf::from("/nonexistent/path/SKILL.md");
        let (name, cmd) = parse_skill_md(&path, "missing");
        assert_eq!(name, "Missing");
        assert_eq!(cmd, "/missing");
    }
}
