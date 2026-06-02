# Silicon Corps Panel Refactoring — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor the 硅基军团 (Silicon Corps) panel to remove view mode (click=edit), remove handoff/agent/mcp sections, and replace hardcoded "默认操作" dropdown with a grouped, dynamic "默认能力" dropdown populated from installed skill packages and custom skills.

**Architecture:** New Rust module `skill_discovery.rs` walks installed skill package directories to enumerate individual slash commands. Frontend `DigitalEmployeePanel.vue` loads these commands + custom DB skills, builds grouped options, and displays them in an enhanced `VSelect`. The `editMode` state collapses from `'list'|'edit'|'view'` to `'list'|'edit'`. Handoff rules, Agent, and MCP sections are removed entirely.

**Tech Stack:** Vue 3 + TypeScript (frontend), Rust/Tauri 2 (backend), SeaORM + SQLite (data), PrimeVue Unstyled (UI), vue-i18n (internationalization)

**Spec:** `docs/superpowers/specs/2026-06-01-silicon-corps-panel-refactor-design.md`

---

## File Structure Map

| File | Responsibility |
|------|---------------|
| `src-tauri/src/agent_runtime/skill_discovery.rs` | **NEW** — Walk installed skill dirs, parse SKILL.md frontmatter, return `Vec<SkillCommand>` |
| `src-tauri/src/agent_runtime/mod.rs` | Register `skill_discovery` module |
| `src-tauri/src/lib.rs` | Register `agent_list_skill_commands` Tauri command |
| `src-tauri/src/db/entities/digital_employee.rs` | SeaORM entity — `default_op` → `default_capability` |
| `src-tauri/src/digital_employee.rs` | Business logic — update struct & row mapping |
| `src-tauri/src/handlers/de_handler.rs` | Tauri command handlers — update field references |
| `src-tauri/src/db/local_sqlite.rs` | Raw SQL — update default field reference |
| `src-tauri/src/db/migrations/m20260529_001_create_digital_tables.sql` | Migration SQL — add `default_capability` column |
| `src-tauri/src/db/migrations/m20260529_002_seed_digital_employees.sql` | Seed SQL — update seed data |
| `src/components/base/VSelect.vue` | Base component — add grouped option support |
| `src/components/domain/DigitalEmployeePanel.vue` | **MAJOR** — Remove view mode, handoffs/agent/mcp, add capability dropdown |
| `src/i18n/zh-CN.ts` | Chinese translations — add ~40 capability names, rename/remove keys |
| `src/i18n/en-US.ts` | English translations — mirror zh-CN |
| `src/api/types.ts` | TypeScript types — update `DigitalEmployee` interface |
| `prd.yml` | Product doc — reflect feature changes |
| `function-map.yml` | Function map — reflect capability changes |

---

### Task 1: Create skill_discovery.rs — walk installed skills and enumerate commands

**Files:**
- Create: `src-tauri/src/agent_runtime/skill_discovery.rs`

- [ ] **Step 1: Write the module with directory walking logic**

```rust
//! Skill command discovery from installed skill packages.
//!
//! Walks `{skills_dir}/{package}/skills/*/SKILL.md` to enumerate
//! individual slash commands available in each installed package.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::agent_runtime::runtime_config::{self, RuntimeEntry};
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
///
/// Expected YAML frontmatter:
/// ```yaml
/// ---
/// name: Brainstorming
/// description: ...
/// ---
/// ```
///
/// Also scans for the first slash command pattern like `/brainstorm` in the body.
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
    // Check first line is "---"
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
/// Looks for patterns like `/command-name` or `Command: /command-name`.
fn find_first_command(content: &str, fallback_name: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        // Pattern: /command-name
        if let Some(start) = trimmed.find('/') {
            let rest = &trimmed[start..];
            let cmd: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == ':')
                .collect();
            if cmd.len() > 1 {
                return cmd;
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

// ── Tests ──

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

        // Create mock package structure: superpowers/skills/brainstorming/SKILL.md
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

        // Create a second skill
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
        // No SKILL.md — should use fallback

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
        assert_eq!(cmd, "/my-skill"); // fallback
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
```

- [ ] **Step 2: Run tests to verify**

Run: `cargo test --lib agent_runtime::skill_discovery -- --nocapture`
Expected: 10 tests pass

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/agent_runtime/skill_discovery.rs
git commit -m "feat: add skill_discovery module — walk installed skill packages to enumerate commands"
```

---

### Task 2: Register skill_discovery module and Tauri command

**Files:**
- Modify: `src-tauri/src/agent_runtime/mod.rs:17`
- Modify: `src-tauri/src/lib.rs` (find `agent_list_installed_skills` registration area)

- [ ] **Step 1: Add module declaration in mod.rs**

Edit `src-tauri/src/agent_runtime/mod.rs`, add after line 15 (`pub mod skill_installer;`):

```rust
pub mod skill_discovery;
```

- [ ] **Step 2: Make resolve_skills_dir_inner public in skill_installer.rs**

Edit `src-tauri/src/agent_runtime/skill_installer.rs`, change line 618 from `pub(crate)` to `pub`:

```rust
pub fn resolve_skills_dir_inner(
```

- [ ] **Step 3: Add to generate_handler! in lib.rs**

In `src-tauri/src/lib.rs`, add after line 199 (`agent_runtime::skill_installer::agent_update_skills,`):

```rust
            // agent_runtime — skill discovery
            agent_runtime::skill_discovery::agent_list_skill_commands,
```

- [ ] **Step 4: Build check**

Run: `cargo check 2>&1`
Expected: No errors

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/agent_runtime/mod.rs src-tauri/src/lib.rs src-tauri/src/agent_runtime/skill_installer.rs
git commit -m "feat: register skill_discovery module and agent_list_skill_commands Tauri command"
```

---

### Task 3: Update DB schema — add default_capability column

**Files:**
- Modify: `src-tauri/src/db/migrations/m20260529_001_create_digital_tables.sql:20`
- Modify: `src-tauri/src/db/migrations/m20260529_002_seed_digital_employees.sql`
- Modify: `src-tauri/src/db/entities/digital_employee.rs:19`
- Modify: `src-tauri/src/db/local_sqlite.rs:385`

- [ ] **Step 1: Add migration column in SQL**

Edit `src-tauri/src/db/migrations/m20260529_001_create_digital_tables.sql`, add after line 20 (`default_op VARCHAR(64) NOT NULL DEFAULT 'write'`):

```sql
  default_capability VARCHAR(256) NOT NULL DEFAULT '',
```

- [ ] **Step 2: Update seed SQL**

Edit `src-tauri/src/db/migrations/m20260529_002_seed_digital_employees.sql`, add `default_capability` column to each INSERT:

For each INSERT line, add `''` for `default_capability` right after `default_op`. For example, change:
```sql
('architect','架构师',1,'["系统思维","逻辑严密"]','系统设计','架构设计','design',...)
```
To:
```sql
('architect','架构师',1,'["系统思维","逻辑严密"]','系统设计','架构设计','design','',...)
```

And update the column list in the INSERT statement to include `default_capability`:
```sql
INSERT INTO digital_employees (code, name, is_builtin, personality_tags, focus_areas, deliverable_groups, default_op, default_capability, sort_order, created_at, updated_at) VALUES
```

- [ ] **Step 3: Update SeaORM entity**

Edit `src-tauri/src/db/entities/digital_employee.rs:19`:

```diff
-    pub default_op: String,
+    pub default_op: String,
+    pub default_capability: String,
```

- [ ] **Step 4: Update local_sqlite.rs default**

Find the INSERT in `src-tauri/src/db/local_sqlite.rs` around line 385 and add `default_capability`:

```diff
             ("default_op", Value::String("write".into())),
+            ("default_capability", Value::String("".into())),
```

- [ ] **Step 5: Build check**

Run: `cargo check 2>&1`
Expected: No errors

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/db/migrations/m20260529_001_create_digital_tables.sql src-tauri/src/db/migrations/m20260529_002_seed_digital_employees.sql src-tauri/src/db/entities/digital_employee.rs src-tauri/src/db/local_sqlite.rs
git commit -m "feat: add default_capability column to digital_employees table"
```

---

### Task 4: Update Rust DigitalEmployee struct and handlers

**Files:**
- Modify: `src-tauri/src/digital_employee.rs:26,81-82,274`
- Modify: `src-tauri/src/handlers/de_handler.rs` (any default_op references)

- [ ] **Step 1: Update DigitalEmployee struct**

Edit `src-tauri/src/digital_employee.rs:26`:

```diff
-    pub default_op: String,
+    pub default_op: String,
+    #[serde(default)]
+    pub default_capability: String,
```

- [ ] **Step 2: Update row_to_employee mapping**

Find the `row_to_employee` function around line 81-82, add after `default_op`:

```rust
        default_op: row
            .get("default_op")
            .and_then(|v| v.as_str())
            .unwrap_or("read")
            .to_string(),
        default_capability: row
            .get("default_capability")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
```

- [ ] **Step 3: Update save/insert data mapping**

Find line 274 (`data.insert("default_op"...)`) in the save function, add after:

```rust
    data.insert("default_op".to_string(), Value::String(employee.default_op.clone()));
    data.insert("default_capability".to_string(), Value::String(employee.default_capability.clone()));
```

- [ ] **Step 4: Build check**

Run: `cargo check 2>&1`
Expected: No errors

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/digital_employee.rs src-tauri/src/handlers/de_handler.rs
git commit -m "feat: add default_capability field to DigitalEmployee struct and handlers"
```

---

### Task 5: Update VSelect base component for grouped options

**Files:**
- Modify: `src/components/base/VSelect.vue:10-14` (Props interface)
- Modify: `src/components/base/VSelect.vue:44-48` (Select props)

- [ ] **Step 1: Extend VSelect Props to support grouped options**

Edit `src/components/base/VSelect.vue`, change the script section:

```typescript
interface Option {
  value: string | number
  label: string
  group?: string
}

interface Props {
  modelValue?: string | number | null
  options: Option[]
  placeholder?: string
  disabled?: boolean
  loading?: boolean
  /** When set, enables grouped display using this property as group label */
  optionGroupLabel?: string
}

defineProps<Props>()
```

- [ ] **Step 2: Update template to pass group prop to PrimeVue Select**

Edit the template section. The PrimeVue Select already supports `optionGroupLabel` prop. Add it:

```vue
<Select
  :class="[...]"
  :pt="{...}"
  :id="inputId"
  :model-value="modelValue"
  :options="options"
  option-label="label"
  option-value="value"
  :option-group-label="optionGroupLabel"
  :placeholder="placeholder"
  :disabled="disabled"
  :loading="loading"
  @update:model-value="emit('update:modelValue', $event)"
/>
```

Add the scroll panel template slot for grouped display (already present in the existing template around lines 55-65). The key addition is `:option-group-label="optionGroupLabel"`.

- [ ] **Step 3: Verify TypeScript compiles**

Run: `npx vue-tsc --noEmit src/components/base/VSelect.vue 2>&1 | head -20`
Expected: No errors related to VSelect

- [ ] **Step 4: Commit**

```bash
git add src/components/base/VSelect.vue
git commit -m "feat: add grouped option support to VSelect base component"
```

---

### Task 6: Remove view mode, handoff/agent/mcp from DigitalEmployeePanel

**Files:**
- Modify: `src/components/domain/DigitalEmployeePanel.vue` (major changes throughout)

- [ ] **Step 1: Remove view mode — state and method changes**

In the `<script setup>` section:

```diff
- const editMode = ref<'list' | 'edit' | 'view'>('list')
+ const editMode = ref<'list' | 'edit'>('list')
```

Delete these lines entirely:
```typescript
// DELETE:
const editAgents = ref<string[]>([])
const editMcps = ref<string[]>([])
const agentPick = ref('')
const mcpPick = ref('')
const editHandoffs = ref<any[]>([])

function onPickAgent(val: string | number) {...}
function onPickMcp(val: string | number) {...}
```

Delete `handleView()`:
```typescript
// DELETE entire function:
function handleView() { ... }
```

Delete handoff helpers:
```typescript
// DELETE:
function addHandoff() { ... }
function removeHandoff(idx: number) { ... }
```

- [ ] **Step 2: Update handleRowClick — edit not view**

```diff
  function handleRowClick(row: any) {
    selectedIds.value = new Set([row.id])
-   isNew.value = false; editMode.value = 'view'
+   isNew.value = false; editMode.value = 'edit'
    editForm.value = { ...row }
    editSkills.value = parseJsonArray(row.skills)
-   editAgents.value = parseJsonArray(row.agents)
-   editMcps.value = parseJsonArray(row.mcps)
-   editHandoffs.value = (parseJsonArray(row.handoffs) as any[]).map(...)
  }
```

- [ ] **Step 3: Update handleEdit — remove agent/mcp/handoff loading**

```diff
  function handleEdit() {
    // ... validation ...
    isNew.value = false; editMode.value = 'edit'
    editForm.value = { ...emp }
    editSkills.value = parseJsonArray(emp.skills)
-   editAgents.value = parseJsonArray(emp.agents)
-   editMcps.value = parseJsonArray(emp.mcps)
-   editHandoffs.value = (parseJsonArray(emp.handoffs) as any[]).map(...)
  }
```

- [ ] **Step 4: Update handleNew — remove agent/mcp/handoff init**

```diff
  function handleNew() {
    isNew.value = true; editMode.value = 'edit'
    editForm.value = { code: '', name: '', is_builtin: false, role: '', personality_tags: '[]', personality_desc: '', comm_style: '', decision_pref: '', focus_areas: '[]', deliverable_groups: '[]', default_op: 'read', default_capability: '', avatar: '' }
-   editSkills.value = []; editAgents.value = []; editMcps.value = []; editHandoffs.value = []
+   editSkills.value = []
  }
```

- [ ] **Step 5: Update handleCopy — remove agent/mcp/handoff copy**

```diff
  function handleCopy() {
    isNew.value = true
    editForm.value = { ...editForm.value, id: undefined, code: '', name: editForm.value.name + ' (副本)', is_builtin: false }
    editSkills.value = [...editSkills.value]
-   editAgents.value = [...editAgents.value]
-   editMcps.value = [...editMcps.value]
-   editHandoffs.value = editHandoffs.value.map((h: any) => ({ ...h, id: undefined }))
  }
```

- [ ] **Step 6: Remove view-related template elements**

Remove the "查看" button (line 255):
```diff
- <VButton size="sm" variant="ghost" @click="handleView">{{ de.view || '查看' }}</VButton>
```

Change the edit header (line 299):
```diff
- <span class="text-base font-semibold text-text-primary">{{ isNew ? (de.newEmployee || '新增数字员工') : (editMode === 'view' ? (de.viewEmployee || '查看数字员工') : (de.editEmployee || '编辑数字员工')) }}</span>
+ <span class="text-base font-semibold text-text-primary">{{ isNew ? (de.newEmployee || '新增数字员工') : (de.editEmployee || '编辑数字员工') }}</span>
```

- [ ] **Step 7: Remove all view-mode disabled bindings**

Remove every `:disabled="editMode === 'view'"` from the template (search and remove). Fields are now always editable in edit mode, no view mode.

- [ ] **Step 8: Remove handoff rules fieldset (lines 426-437)**

Delete the entire handoff rules `<fieldset>` block and its contents.

- [ ] **Step 9: Remove Agent and MCP rows from skill stack**

Delete the Agent row (lines 390-406) and MCP row (lines 407-423) from the skill stack fieldset.

- [ ] **Step 10: Update defaultOp label and field**

```diff
- <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">{{ de.defaultOp || '默认操作' }}</label>
- <VSelect v-model="editForm.default_op" :options="opOptions" class="!w-[200px]" :disabled="editMode === 'view'" />
+ <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">{{ de.defaultCapability || '默认能力' }}</label>
+ <VSelect v-model="editForm.default_capability" :options="capabilityOptions" option-group-label="group" class="!w-[200px]" :loading="capabilityLoading" />
```

- [ ] **Step 11: Remove unused variables**

Delete unused refs/computeds no longer needed:
- `const opOptions` (line 231)
- `const empOptions` (line 232)
- `const tmOptions` (line 233)
- `const transferModeOptions` (line 71-75)
- `const operationOptions` (line 70)

- [ ] **Step 12: Commit**

```bash
git add src/components/domain/DigitalEmployeePanel.vue
git commit -m "refactor: remove view mode, handoff rules, agent/mcp sections from Silicon Corps panel"
```

---

### Task 7: Add capability loading logic to DigitalEmployeePanel

**Files:**
- Modify: `src/components/domain/DigitalEmployeePanel.vue` (add new code)

- [ ] **Step 1: Add type definitions (inside `<script setup>`)**

```typescript
// Skill command as returned by the Rust backend
interface SkillCommand {
  package: string
  skill_name: string
  command: string
  display_name_en: string
}

interface CapabilityOption {
  value: string
  label: string
  group: string
  command: string
}
```

- [ ] **Step 2: Add SKILL_NAME_MAP and PACKAGE_DISPLAY_NAMES constants**

```typescript
const PACKAGE_DISPLAY_NAMES: Record<string, string> = {
  'superpowers':            'Super Power',
  'gstack':                 'Gstack',
  'everything-claude-code': 'Everything Claude Code',
  'super-claude':           'Super Claude',
  'claude-official-skills': 'OpenSpec',
}

const SKILL_NAME_MAP: Record<string, string> = {
  // ── superpowers ──
  'superpowers/brainstorming':              'de.cap.brainstorming',
  'superpowers/writing-plans':              'de.cap.writingPlans',
  'superpowers/executing-plans':            'de.cap.executingPlans',
  'superpowers/test-driven-development':    'de.cap.testDrivenDevelopment',
  'superpowers/systematic-debugging':       'de.cap.systematicDebugging',
  'superpowers/subagent-driven-development':'de.cap.subagentDrivenDev',
  'superpowers/verification-before-completion':'de.cap.verificationCheck',
  'superpowers/requesting-code-review':     'de.cap.requestingCodeReview',
  'superpowers/dispatching-parallel-agents':'de.cap.dispatchingAgents',
  'superpowers/using-git-worktrees':       'de.cap.gitWorktrees',
  'superpowers/graphify':                  'de.cap.graphify',
  // ── gstack ──
  'gstack/browse':              'de.cap.browse',
  'gstack/qa':                  'de.cap.qa',
  'gstack/ship':                'de.cap.ship',
  'gstack/plan-eng-review':     'de.cap.planEngReview',
  'gstack/cso':                 'de.cap.cso',
  'gstack/document-generate':   'de.cap.documentGenerate',
  'gstack/office-hours':        'de.cap.officeHours',
  'gstack/review':              'de.cap.gstackReview',
  'gstack/land-and-deploy':     'de.cap.landAndDeploy',
  'gstack/design-consultation': 'de.cap.designConsultation',
  'gstack/investigate':         'de.cap.investigate',
  'gstack/retro':               'de.cap.retro',
  // ── everything-claude-code ──
  'everything-claude-code/code-review':      'de.cap.codeReview',
  'everything-claude-code/arch-review':      'de.cap.archReview',
  'everything-claude-code/security-review':  'de.cap.securityReview',
  'everything-claude-code/sc:design':        'de.cap.scDesign',
  'everything-claude-code/sc:implement':     'de.cap.scImplement',
  'everything-claude-code/sc:analyze':       'de.cap.scAnalyze',
  'everything-claude-code/e2e':              'de.cap.e2e',
  'everything-claude-code/deep-research':    'de.cap.deepResearch',
  'everything-claude-code/orchestrate':      'de.cap.orchestrate',
  'everything-claude-code/tdd':              'de.cap.tdd',
  'everything-claude-code/docs':             'de.cap.docs',
  'everything-claude-code/plan':             'de.cap.plan',
  'everything-claude-code/loop':             'de.cap.loop',
  // ── super-claude ── (placeholder — will be refined after repo inspection)
  // 'super-claude/...': 'de.cap.superClaudeXxx',
  // ── claude-official-skills (OpenSpec) ──
  'claude-official-skills/openspec-propose':       'de.cap.openspecPropose',
  'claude-official-skills/openspec-apply-change':  'de.cap.openspecApply',
  'claude-official-skills/openspec-explore':       'de.cap.openspecExplore',
  'claude-official-skills/openspec-new-change':    'de.cap.openspecNewChange',
  'claude-official-skills/openspec-archive-change':'de.cap.openspecArchive',
  'claude-official-skills/openspec-verify-change': 'de.cap.openspecVerify',
}
```

- [ ] **Step 3: Add capability state and loading functions**

```typescript
const capabilityOptions = ref<CapabilityOption[]>([])
const capabilityLoading = ref(false)

async function loadCapabilities() {
  capabilityLoading.value = true
  try {
    const commands = await invoke<SkillCommand[]>('agent_list_skill_commands', { runtime: 'claude_code' })
    capabilityOptions.value = buildGroupedOptions(commands)
  } catch {
    // Commands may fail if no skills installed — use empty list
    capabilityOptions.value = buildGroupedOptions([])
  } finally {
    capabilityLoading.value = false
  }
}

function buildGroupedOptions(commands: SkillCommand[]): CapabilityOption[] {
  const options: CapabilityOption[] = []

  // Group installed skill commands by package
  const grouped = new Map<string, SkillCommand[]>()
  for (const cmd of commands) {
    if (!grouped.has(cmd.package)) grouped.set(cmd.package, [])
    grouped.get(cmd.package)!.push(cmd)
  }
  for (const [pkg, cmds] of grouped) {
    const pkgDisplayName = PACKAGE_DISPLAY_NAMES[pkg] || pkg
    for (const cmd of cmds) {
      const mapKey = `${cmd.package}/${cmd.skill_name}`
      const i18nKey = SKILL_NAME_MAP[mapKey]
      const displayName = i18nKey ? tt(i18nKey) : (cmd.display_name_en || cmd.skill_name)
      options.push({
        value: mapKey,
        label: `${displayName} (${cmd.command})`,
        group: pkgDisplayName,
        command: cmd.command,
      })
    }
  }

  return options
}
```

- [ ] **Step 4: Call loadCapabilities in onMounted**

Add to the `onMounted` callback:

```typescript
onMounted(async () => {
  await loadEmployees()
  await loadLookups()
  await loadCapabilities()
})
```

- [ ] **Step 5: Verify TypeScript compiles**

Run: `npx vue-tsc --noEmit 2>&1 | head -30`
Expected: No new errors

- [ ] **Step 6: Commit**

```bash
git add src/components/domain/DigitalEmployeePanel.vue
git commit -m "feat: add grouped capability dropdown loading from installed skills"
```

---

### Task 8: Update i18n files — zh-CN.ts and en-US.ts

**Files:**
- Modify: `src/i18n/zh-CN.ts:263-307`
- Modify: `src/i18n/en-US.ts:263-307`

- [ ] **Step 1: Update zh-CN.ts**

In `src/i18n/zh-CN.ts`, replace the `digitalEmployee` block (lines 263-307) with the updated version:

```typescript
  digitalEmployee: {
    title: '硅基军团',
    searchPlaceholder: '查找员工...',
    name: '姓名',
    code: '编码',
    role: '角色',
    personality: '性格',
    type: '类型',
    builtin: '内置',
    custom: '自定义',
    focusAreas: '专注领域',
    noData: '暂无数据',
    selectOneHint: '请选择一个员工',
    new: '新增',
    edit: '编辑',
    back: '返回列表',
    newEmployee: '新增硅基军团',
    editEmployee: '编辑硅基军团',
    basicInfo: '基础信息',
    avatar: '头像',
    personalityTags: '性格标签',
    personalityDesc: '性格描述',
    commStyle: '沟通风格',
    decisionPref: '决策偏好',
    capabilities: '能力配置',
    deliverableGroups: '交付物组',
    defaultCapability: '默认能力',
    skillStack: '技能栈',
    guerrillas: '游击队',
    create: '创建',
    save: '保存',
    copy: '复制',
    cancel: '取消',
    delete: '删除',
    nameOrCodeRequired: '编码和姓名不能为空',
    createSuccess: '创建成功',
    saveSuccess: '保存成功',
    builtinCannotDelete: '内置员工不可删除',
    deleteConfirmTitle: '确认删除',
    deleteConfirmMessage: '确定删除员工「{name}」？',
    deleted: '已删除',
    cap: {
      // superpowers
      brainstorming: '头脑风暴',
      writingPlans: '编写计划',
      executingPlans: '执行计划',
      testDrivenDevelopment: '测试驱动开发',
      systematicDebugging: '系统调试',
      subagentDrivenDev: '子代理驱动开发',
      verificationCheck: '完成前验证',
      requestingCodeReview: '请求代码评审',
      dispatchingAgents: '并行代理调度',
      gitWorktrees: 'Git工作树',
      graphify: '知识图谱',
      // gstack
      browse: '网页浏览',
      qa: 'QA测试',
      ship: '部署发布',
      planEngReview: '工程评审',
      cso: '安全审计',
      documentGenerate: '文档生成',
      officeHours: '办公时间',
      gstackReview: '代码评审',
      landAndDeploy: '部署上线',
      designConsultation: '设计咨询',
      investigate: '问题调查',
      retro: '回顾总结',
      // everything-claude-code
      codeReview: '代码评审',
      archReview: '架构评审',
      securityReview: '安全审查',
      scDesign: '系统设计',
      scImplement: '代码实现',
      scAnalyze: '代码分析',
      e2e: '端到端测试',
      deepResearch: '深度研究',
      orchestrate: '多智能体编排',
      tdd: '测试驱动开发',
      docs: '文档生成',
      plan: '制定计划',
      loop: '自主循环',
      // openspec
      openspecPropose: '创建变更提案',
      openspecApply: '实施变更',
      openspecExplore: '需求探索',
      openspecNewChange: '新建变更',
      openspecArchive: '归档变更',
      openspecVerify: '验证变更',
    },
  },
```

- [ ] **Step 2: Update en-US.ts**

In `src/i18n/en-US.ts`, replace the `digitalEmployee` block with the English equivalent:

```typescript
  digitalEmployee: {
    title: 'Silicon Corps',
    searchPlaceholder: 'Search employees...',
    name: 'Name',
    code: 'Code',
    role: 'Role',
    personality: 'Personality',
    type: 'Type',
    builtin: 'Built-in',
    custom: 'Custom',
    focusAreas: 'Focus Areas',
    noData: 'No data',
    selectOneHint: 'Please select an employee',
    new: 'New',
    edit: 'Edit',
    back: 'Back to List',
    newEmployee: 'New Silicon Corps',
    editEmployee: 'Edit Silicon Corps',
    basicInfo: 'Basic Info',
    avatar: 'Avatar',
    personalityTags: 'Personality Tags',
    personalityDesc: 'Personality Description',
    commStyle: 'Communication Style',
    decisionPref: 'Decision Preference',
    capabilities: 'Capabilities',
    deliverableGroups: 'Deliverable Groups',
    defaultCapability: 'Default Capability',
    skillStack: 'Skill Stack',
    guerrillas: 'Guerrillas',
    create: 'Create',
    save: 'Save',
    copy: 'Copy',
    cancel: 'Cancel',
    delete: 'Delete',
    nameOrCodeRequired: 'Code and name cannot be empty',
    createSuccess: 'Created successfully',
    saveSuccess: 'Saved successfully',
    builtinCannotDelete: 'Built-in employees cannot be deleted',
    deleteConfirmTitle: 'Confirm Delete',
    deleteConfirmMessage: 'Delete employee "{name}"?',
    deleted: 'Deleted',
    cap: {
      // superpowers
      brainstorming: 'Brainstorming',
      writingPlans: 'Writing Plans',
      executingPlans: 'Executing Plans',
      testDrivenDevelopment: 'Test-Driven Development',
      systematicDebugging: 'Systematic Debugging',
      subagentDrivenDev: 'Subagent-Driven Development',
      verificationCheck: 'Verification Before Completion',
      requestingCodeReview: 'Requesting Code Review',
      dispatchingAgents: 'Dispatching Parallel Agents',
      gitWorktrees: 'Using Git Worktrees',
      graphify: 'Knowledge Graph',
      // gstack
      browse: 'Web Browsing',
      qa: 'QA Testing',
      ship: 'Deploy & Ship',
      planEngReview: 'Engineering Review',
      cso: 'Security Audit',
      documentGenerate: 'Document Generation',
      officeHours: 'Office Hours',
      gstackReview: 'Code Review',
      landAndDeploy: 'Land & Deploy',
      designConsultation: 'Design Consultation',
      investigate: 'Investigate',
      retro: 'Retrospective',
      // everything-claude-code
      codeReview: 'Code Review',
      archReview: 'Architecture Review',
      securityReview: 'Security Review',
      scDesign: 'System Design',
      scImplement: 'Code Implementation',
      scAnalyze: 'Code Analysis',
      e2e: 'E2E Testing',
      deepResearch: 'Deep Research',
      orchestrate: 'Multi-Agent Orchestration',
      tdd: 'Test-Driven Development',
      docs: 'Documentation',
      plan: 'Planning',
      loop: 'Autonomous Loop',
      // openspec
      openspecPropose: 'Create Change Proposal',
      openspecApply: 'Apply Change',
      openspecExplore: 'Explore Requirements',
      openspecNewChange: 'New Change',
      openspecArchive: 'Archive Change',
      openspecVerify: 'Verify Change',
    },
  },
```

- [ ] **Step 3: Verify TypeScript — i18n key structure mirrors**

Run: `npx vue-tsc --noEmit 2>&1 | grep -i 'digitalEmployee\|de\.cap' | head -10`
Expected: No type errors about missing i18n keys

- [ ] **Step 4: Commit**

```bash
git add src/i18n/zh-CN.ts src/i18n/en-US.ts
git commit -m "feat: add capability i18n keys, rename defaultOp→defaultCapability, remove view/handoff keys"
```

---

### Task 9: Update TypeScript types and config files

**Files:**
- Modify: `src/api/types.ts` (add `DigitalEmployee` interface if not present, or update existing)
- Modify: `prd.yml`
- Modify: `function-map.yml`

- [ ] **Step 1: Update or verify DigitalEmployee type**

Check `src/api/types.ts` for a `DigitalEmployee` interface. If present, add `default_capability: string`. If not present (the type is inferred from `any` in the panel), no change needed — the panel uses `any` types.

Search for DigitalEmployee in types:
```bash
grep -n 'DigitalEmployee\|digital_employee' src/api/types.ts
```

If no interface exists, no change needed. If one exists, add:
```typescript
default_capability: string
```

- [ ] **Step 2: Update prd.yml**

Update the Silicon Corps feature description to reflect the new capability system. Change "默认操作" references to "默认能力" and note that handoff rules have been removed.

- [ ] **Step 3: Update function-map.yml**

Update any references to `default_op` to `default_capability` in function-map.yml.

- [ ] **Step 4: Commit**

```bash
git add src/api/types.ts prd.yml function-map.yml
git commit -m "docs: sync types, prd, and function-map with Silicon Corps capability changes"
```

---

### Task 10: End-to-end verification — build and run

- [ ] **Step 1: Full Rust build**

Run: `cargo build 2>&1`
Expected: Build succeeds with no errors

- [ ] **Step 2: Rust tests**

Run: `cargo test --lib 2>&1`
Expected: All existing tests pass + new skill_discovery tests pass (18+ tests total)

- [ ] **Step 3: Frontend type check**

Run: `npx vue-tsc --noEmit 2>&1`
Expected: No type errors

- [ ] **Step 4: Frontend build**

Run: `npm run build 2>&1`
Expected: Build succeeds

- [ ] **Step 5: Manual verification checklist**

Launch app and verify:
- [ ] Click any row in Silicon Corps list → enters edit mode directly (not view)
- [ ] No "查看" button in toolbar
- [ ] No "交互规则" section in edit form
- [ ] No "Agent" row in skill stack
- [ ] No "MCP" row in skill stack
- [ ] "默认能力" label shows instead of "默认操作"
- [ ] Default capability dropdown loads and shows grouped options
- [ ] Selecting a capability and saving persists correctly
- [ ] Switch language to English → all labels update correctly
- [ ] Creating a new employee works with default_capability field

- [ ] **Step 6: Final commit (if any tweaks needed)**

```bash
git add -A
git commit -m "chore: final verification tweaks for Silicon Corps panel refactoring"
```
