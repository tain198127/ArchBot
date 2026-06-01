## 1. Config & Data Model

- [ ] 1.1 Define `SkillBundle` and `SkillEntry` structs in `src-tauri/src/agent_runtime/runtime_config.rs` — deserialize from `skill_bundle` section in `runtimes.yml`
- [ ] 1.2 Define default skill bundle for `claude_code` with 5 curated skills: Claude official skills, Superpowers, gstack, Super Claude, Everything Claude Code (with repo URLs and pinned refs)
- [ ] 1.3 Add `SkillInstallResult` and `SkillStatus` structs for per-skill installation status reporting
- [ ] 1.4 Write unit tests for `SkillBundle` deserialization (valid config, missing fields, empty list)

## 2. Skill Installer Module

- [ ] 2.1 Create `src-tauri/src/agent_runtime/skill_installer.rs` — `install_skill_bundle()` function that iterates over skills, runs `git clone --depth 1 --branch <ref>` into `{isolated_home}/.claude/skills/{name}/`
- [ ] 2.2 Implement `check_git_available()` — verify `git` is on PATH before attempting clones
- [ ] 2.3 Implement `update_skill()` — `git fetch origin {ref}` + `git reset --hard FETCH_HEAD` for each installed skill
- [ ] 2.4 Implement `list_installed_skills()` — scan `{isolated_home}/.claude/skills/` for directories with `.git` and read current ref
- [ ] 2.5 Implement non-blocking failure: if a skill clone fails, log error and continue with remaining skills
- [ ] 2.6 Handle edge cases: target directory already exists (skip), invalid ref (report error), permission denied (report error)
- [ ] 2.7 Write unit tests: successful install, git-not-found, already-installed, partial-failure, invalid ref

## 3. Version Manager Integration

- [ ] 3.1 Modify `install_runtime()` in `version_manager.rs` — after successful binary install and `link_current()`, call `install_skill_bundle()` if the runtime has an enabled skill bundle
- [ ] 3.2 Read the runtime's isolated HOME path from the runtime config to determine skill install target
- [ ] 3.3 Return combined result: binary install status + per-skill installation results
- [ ] 3.4 Ensure backward compatibility: if no `skill_bundle` config exists, `install_runtime()` behaves exactly as before

## 4. Tauri Commands

- [ ] 4.1 Register `agent_install_skills` command — accepts `runtime` string, reads config, calls `install_skill_bundle()`, returns `Vec<SkillInstallResult>`
- [ ] 4.2 Register `agent_list_installed_skills` command — accepts `runtime` string, calls `list_installed_skills()`, returns list with name/ref/timestamp
- [ ] 4.3 Register `agent_update_skills` command — accepts `runtime` string, calls `update_skill()` for each installed skill
- [ ] 4.4 Register all new commands in `src-tauri/src/lib.rs`

## 5. Frontend — AgentConfigPanel

- [ ] 5.1 Add skill-related state to `AgentState` interface: `skillStatuses`, `skillInstalling`, `skillResult`
- [ ] 5.2 Extend `installRuntime()` to display skill installation progress after binary install completes
- [ ] 5.3 Add "Installed Skills" section below the Install section — list installed skills with name, version, status icon
- [ ] 5.4 Add "Reinstall Skills" button that calls `agent_install_skills`
- [ ] 5.5 Add "Update Skills" button that calls `agent_update_skills`
- [ ] 5.6 Add skill-related API wrappers to `src/stores/agentStore.ts`

## 6. i18n

- [ ] 6.1 Add new i18n keys to `src/i18n/en-US.ts`: `agentConfig.skillsTitle`, `agentConfig.skillsInstalling`, `agentConfig.skillsReinstall`, `agentConfig.skillsUpdate`, `agentConfig.skillsNone`, `agentConfig.skillsFailed`, `agentConfig.skillsGitNotFound`
- [ ] 6.2 Mirror all new keys in `src/i18n/zh-CN.ts`

## 7. Integration, Testing & Polish

- [ ] 7.1 Run `cargo check` and `cargo clippy` — fix all warnings
- [ ] 7.2 Run `cargo test` — ensure all unit tests pass with 80%+ coverage on `skill_installer.rs`
- [ ] 7.3 Run `cargo fmt` — format all Rust source files
- [ ] 7.4 Manual E2E test: click Install for Claude Code → verify skills appear in `~/.archbot/runtimes/claude_code/current/.claude/skills/`
- [ ] 7.5 Manual test: verify skill reinstall and update flows from the UI
- [ ] 7.6 Manual test: verify graceful degradation when git is not on PATH
- [ ] 7.7 Update `prd.yml` with skill bundle feature
- [ ] 7.8 Update `function-map.yml` with new `skill_installer` module mapping
