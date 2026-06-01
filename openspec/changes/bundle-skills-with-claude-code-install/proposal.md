## Why

Currently, when users click "Install" for Claude Code in the agent config panel, only the Claude Code CLI binary itself is installed (via PATH detection + symlink). To get a productive AI coding environment, users must then manually discover and install popular skill packs — Claude official skills, Superpowers, gstack, Super Claude, Everything Claude Code — each through separate setup steps. This is friction-heavy and error-prone. Bundling skill installation into the one-click "Install" flow transforms ArchBot from a bare runtime installer into a complete AI coding workstation provisioner.

## What Changes

- Define a curated **skill bundle** for Claude Code: a configurable list of skill repositories that are installed alongside the runtime
- Extend `install_runtime` (Rust backend) to accept an optional skill bundle manifest, and after binary installation succeeds, clone/copy skills into the runtime's isolated `~/.claude/skills/` directory
- Add a new `SkillBundle` struct and `skill_installer.rs` module: manages skill repository cloning, version pinning, and updates
- Add Tauri commands: `agent_install_skills`, `agent_list_installed_skills`, `agent_update_skills`
- Extend `AgentConfigPanel.vue`: show skill installation progress during install, list installed skills post-install, offer "Reinstall Skills" button
- Skills are installed into the isolated HOME's `.claude/skills/` directory, respecting the runtime's sandbox
- Add i18n entries for new skill-related UI strings

## Capabilities

### New Capabilities

- `skill-bundle-installer`: Backend module that clones/updates skill repositories into the runtime's isolated skills directory, with version pinning and progress reporting
- `skill-bundle-config`: YAML configuration defining the curated skill list (repo URLs, default refs, descriptions) bundled with Claude Code

### Modified Capabilities

- `agent-version-manager`: `install_runtime` is extended to trigger skill installation as a post-install step for supported runtimes (currently `claude_code` only)

## Impact

- **Rust backend**: New `skill_installer.rs` module, changes to `version_manager.rs` (post-install hook), `lib.rs` (register new commands)
- **Frontend**: `AgentConfigPanel.vue` — new skill status section, progress indicators; `agentStore.ts` — new API wrappers
- **Config**: New `skills.yml` or section in `runtimes.yml` defining the bundle
- **Dependencies**: `git2` crate (or shell out to `git`) for cloning skill repos; alternatively use `reqwest` to download tarballs
- **i18n**: New keys in `en-US.ts` and `zh-CN.ts`
