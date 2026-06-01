## ADDED Requirements

### Requirement: Skill installation from git repositories
The system SHALL install skill packs by performing shallow git clones (`git clone --depth 1 --branch <ref>`) of each skill repository into `{isolated_home}/.claude/skills/{skill_name}/`.

#### Scenario: Successful skill installation
- **WHEN** `install_skill_bundle` is called for runtime `claude_code` with a valid skill list
- **THEN** each skill repository is shallow-cloned into `{isolated_home}/.claude/skills/{name}/` and the system returns a list of installed skills with their versions

#### Scenario: Skill already installed
- **WHEN** a skill's target directory already exists
- **THEN** the system skips that skill and reports it as "already installed"

#### Scenario: Network failure during clone
- **WHEN** a git clone fails due to network error or unreachable repository
- **THEN** the system logs the error, marks that skill as "failed", and continues installing remaining skills without blocking the overall installation

#### Scenario: Git not available on system
- **WHEN** `git` is not found on PATH
- **THEN** the system skips all skill installations and returns a clear message: "git not found — skills skipped"

### Requirement: Skill update via git fetch
The system SHALL support updating installed skills by fetching the latest commit from the configured ref and resetting to it.

#### Scenario: Update to latest version
- **WHEN** `agent_update_skills` is called for a runtime
- **THEN** each installed skill runs `git fetch origin {ref}` and `git reset --hard FETCH_HEAD`, reporting the new commit hash

#### Scenario: Skill has local modifications
- **WHEN** a skill directory has uncommitted changes
- **THEN** the update force-resets (`git reset --hard`) to the remote ref, discarding local modifications, and logs a warning

### Requirement: Skill progress reporting
The system SHALL emit progress events during skill installation so the frontend can display per-skill status.

#### Scenario: Multi-skill installation progress
- **WHEN** installing 5 skills
- **THEN** the system emits progress events: `skill.installing` (per skill start), `skill.installed` (per skill success), `skill.failed` (per skill error), and `skills.complete` (all done)

#### Scenario: Frontend receives progress events
- **WHEN** the Vue frontend calls `agent_install_skills`
- **THEN** it receives a structured result with per-skill status (name, status, version, error_message) for display

### Requirement: Tauri commands for skill management
The system SHALL expose skill management operations as Tauri commands: `agent_install_skills`, `agent_list_installed_skills`, `agent_update_skills`.

#### Scenario: Frontend lists installed skills
- **WHEN** Vue frontend calls `invoke("agent_list_installed_skills", { runtime: "claude_code" })`
- **THEN** backend scans `{isolated_home}/.claude/skills/` and returns a list of skill names with their git ref and last-updated timestamp

#### Scenario: Frontend triggers skill reinstall
- **WHEN** Vue frontend calls `invoke("agent_install_skills", { runtime: "claude_code" })`
- **THEN** backend reads the skill bundle config and executes the full install flow, returning per-skill statuses
