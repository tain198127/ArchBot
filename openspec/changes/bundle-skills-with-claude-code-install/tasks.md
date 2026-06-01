## 1. Config & Data Model

- [x] 1.1 Define `SkillBundle` and `SkillEntry` structs in `src-tauri/src/agent_runtime/runtime_config.rs` — deserialize from `skill_bundle` section in `runtimes.yml`
- [x] 1.2 Define default skill bundle for `claude_code` with 5 curated skills: Claude official skills, Superpowers, gstack, Super Claude, Everything Claude Code (with repo URLs and pinned refs)
- [x] 1.3 Add `SkillInstallResult` and `SkillStatus` structs for per-skill installation status reporting
- [x] 1.4 Write unit tests for `SkillBundle` deserialization (valid config, missing fields, empty list)

## 2. Skill Installer Module

- [x] 2.1 Create `src-tauri/src/agent_runtime/skill_installer.rs` — `install_skill_bundle()` function that iterates over skills, runs `git clone --depth 1 --branch <ref>` into `{isolated_home}/.claude/skills/{name}/`
- [x] 2.2 Implement `check_git_available()` — verify `git` is on PATH before attempting clones
- [x] 2.3 Implement `update_skill()` — `git fetch origin {ref}` + `git reset --hard FETCH_HEAD` for each installed skill
- [x] 2.4 Implement `list_installed_skills()` — scan `{isolated_home}/.claude/skills/` for directories with `.git` and read current ref
- [x] 2.5 Implement non-blocking failure: if a skill clone fails, log error and continue with remaining skills
- [x] 2.6 Handle edge cases: target directory already exists (skip), invalid ref (report error), permission denied (report error)
- [x] 2.7 Write unit tests: successful install, git-not-found, already-installed, partial-failure, invalid ref

## 3. Version Manager Integration

- [x] 3.1 Modify `install_runtime()` in `version_manager.rs` — after successful binary install and `link_current()`, call `install_skill_bundle()` if the runtime has an enabled skill bundle
- [x] 3.2 Read the runtime's isolated HOME path from the runtime config to determine skill install target
- [x] 3.3 Return combined result: binary install status + per-skill installation results
- [x] 3.4 Ensure backward compatibility: if no `skill_bundle` config exists, `install_runtime()` behaves exactly as before

## 4. Tauri Commands

- [x] 4.1 Register `agent_install_skills` command — accepts `runtime` string, reads config, calls `install_skill_bundle()`, returns `Vec<SkillInstallResult>`
- [x] 4.2 Register `agent_list_installed_skills` command — accepts `runtime` string, calls `list_installed_skills()`, returns list with name/ref/timestamp
- [x] 4.3 Register `agent_update_skills` command — accepts `runtime` string, calls `update_skill()` for each installed skill
- [x] 4.4 Register all new commands in `src-tauri/src/lib.rs`

## 5. Frontend — AgentConfigPanel

- [x] 5.1 Add skill-related state to `AgentState` interface: `skillStatuses`, `skillInstalling`, `skillResult`
- [x] 5.2 Extend `installRuntime()` to display skill installation progress after binary install completes
- [x] 5.3 Add "Installed Skills" section below the Install section — list installed skills with name, version, status icon
- [x] 5.4 Add "Reinstall Skills" button that calls `agent_install_skills`
- [x] 5.5 Add "Update Skills" button that calls `agent_update_skills`
- [x] 5.6 Add skill-related API wrappers to `src/stores/agentStore.ts`

## 6. i18n

- [x] 6.1 Add new i18n keys to `src/i18n/en-US.ts`: `agentConfig.skillsTitle`, `agentConfig.skillsInstalling`, `agentConfig.skillsReinstall`, `agentConfig.skillsUpdate`, `agentConfig.skillsNone`, `agentConfig.skillsFailed`, `agentConfig.skillsGitNotFound`
- [x] 6.2 Mirror all new keys in `src/i18n/zh-CN.ts`

## 7. Integration, Testing & Polish

- [x] 7.1 Run `cargo check` and `cargo clippy` — fix all warnings
- [x] 7.2 Run `cargo test` — ensure all unit tests pass with 80%+ coverage on `skill_installer.rs`
- [x] 7.3 Run `cargo fmt` — format all Rust source files
- [ ] 7.4 Manual E2E test: click Install for Claude Code → verify skills appear in `~/.archbot/runtimes/claude_code/current/.claude/skills/`
- [ ] 7.5 Manual test: verify skill reinstall and update flows from the UI
- [ ] 7.6 Manual test: verify graceful degradation when git is not on PATH
- [x] 7.7 Update `prd.yml` with skill bundle feature
- [x] 7.8 Update `function-map.yml` with new `skill_installer` module mapping
