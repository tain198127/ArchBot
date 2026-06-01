## Context

ArchBot's agent runtime system currently installs Claude Code by detecting the `claude` binary on PATH and symlinking it into `~/.archbot/runtimes/claude_code/versions/{version}/`. The isolated HOME directory prepared by `home_setup.rs` only sets up `.gitconfig` and SSH keys. However, Claude Code's power comes largely from its skill ecosystem — skill packs like Superpowers, gstack, and official Claude skills dramatically enhance its capabilities. Without these, a freshly-installed Claude Code is a bare shell.

Currently, users must manually discover, clone, and configure each skill pack. This is the gap this design fills.

## Goals / Non-Goals

**Goals:**
- Define a curated, configurable list of skill repositories (the "bundle") that ship with Claude Code
- Automatically install skills into the isolated HOME's `.claude/skills/` directory after runtime installation
- Support subsequent skill updates (re-clone/pull to latest tagged version)
- Show installation progress and results in the AgentConfigPanel UI
- Keep the bundle config editable so advanced users can customize or disable skills

**Non-Goals:**
- A general-purpose skill registry or marketplace — this is a curated bundle, not a package manager
- Skill dependency resolution — skills are installed independently
- Skill conflict detection — if two skills overlap, the last one installed wins
- Cross-runtime skill sharing — each runtime's skills are isolated (may be addressed later)
- Auto-updating skills on a schedule — updates are manual or triggered on runtime update

## Decisions

### Decision 1: Git-based installation with shallow clone

**Chosen**: Use `git clone --depth 1 --branch <ref>` for each skill repository.

**Rationale**: Most skill packs are distributed as GitHub repositories. Shallow clones are fast (~few seconds per repo) and give us exact version pinning via tags/commits. No additional crate dependencies needed — we shell out to `git` (already a system dependency for the target audience).

**Alternative considered**: Download tarballs via `reqwest`. Rejected because tarball URLs vary across GitHub/GitLab and add complexity for private repos. Git is universally available on developer machines.

**Alternative considered**: Use `git2` crate. Rejected because it adds a native build dependency (libgit2) and `git` CLI is more battle-tested for shallow clones.

### Decision 2: Install into isolated HOME's `.claude/skills/`

**Chosen**: Skills go into `{isolated_home}/.claude/skills/{skill_name}/`.

**Rationale**: Claude Code discovers skills by scanning `.claude/skills/` relative to HOME. Since ArchBot sets `HOME` to an isolated directory, placing skills there ensures:
- Skills are sandboxed per runtime instance
- No pollution of the user's real `~/.claude/skills/`
- Clean teardown when a runtime version is removed

**Alternative considered**: Install to a shared location (`~/.archbot/skills/`) and symlink. Rejected because it breaks isolation — a compromised skill could affect all runtimes.

### Decision 3: YAML bundle config embedded in `runtimes.yml`

**Chosen**: Add a `skill_bundle` key to each runtime entry in `runtimes.yml`.

**Rationale**: Keeps configuration co-located with the runtime definition. Users already edit `runtimes.yml` for runtime configuration. The format is declarative and human-readable.

```yaml
runtimes:
  claude_code:
    # ... existing fields ...
    skill_bundle:
      enabled: true
      skills:
        - name: superpowers
          repo: https://github.com/.../superpowers.git
          ref: v2.1.0
          description: "Agent orchestration and development workflow skills"
        - name: gstack
          repo: https://github.com/.../gstack.git
          ref: main
          description: "Headless browser, QA, deployment, and project management"
```

### Decision 4: Post-install hook, not a separate step

**Chosen**: `install_runtime` calls `install_skill_bundle` as a final step before returning success.

**Rationale**: From the user's perspective, "Install Claude Code" means "set up everything I need to use Claude Code." Making skill installation a separate manual step defeats the purpose. The install button already has a loading state — we extend it to show skill progress.

The backend provides separate commands (`agent_install_skills`, `agent_update_skills`) so the frontend can also trigger reinstallation independently.

### Decision 5: Failure is non-blocking

**Chosen**: If a skill fails to clone (network error, repo unavailable), log the error and continue installing remaining skills. The runtime installation itself still succeeds.

**Rationale**: A transient network issue or a deleted repository shouldn't prevent the user from using Claude Code. The UI shows which skills succeeded and which failed, with retry options.

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| Skill repo removed or renamed → install fails for that skill | Non-blocking failure; user can edit `skills_bundle.yml` to remove or update the URL |
| Git not installed on user's machine | Check for `git` on PATH before attempting; if missing, skip skill installation with a clear message |
| Large repos slow down installation | Shallow clone (`--depth 1`) keeps downloads small; show per-skill progress in UI |
| Skill version conflicts between updates | Version pinning via `ref` (tag/commit); user controls when to update |
| Skills may contain malicious code | Same trust model as any cloned repo; skills run inside the isolated runtime sandbox |

## Open Questions

1. **Default skill list**: Which exact repositories and versions should be in the default bundle? The user mentioned Claude official skills, Super Claude, Everything Claude Code, Superpowers, and gstack — we need to confirm exact repo URLs.
2. **Skill updates on runtime update**: When user clicks "Update" to switch Claude Code versions, should skills also be re-fetched? Leaning toward: only if `--update-skills` flag is passed, or a separate button.
3. **Windows support**: `git` shell-out works on Windows if Git for Windows is installed. Should we document this as a prerequisite?
