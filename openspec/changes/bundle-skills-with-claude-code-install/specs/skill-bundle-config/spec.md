## ADDED Requirements

### Requirement: Skill bundle YAML configuration
The system SHALL support a `skill_bundle` configuration section within each runtime entry in `runtimes.yml` that defines the curated list of skills to install.

#### Scenario: Default Claude Code skill bundle
- **WHEN** `runtimes.yml` contains `claude_code.skill_bundle.enabled: true` with a list of skills
- **THEN** `install_runtime` for `claude_code` reads this configuration and installs all listed skills after binary installation

#### Scenario: Disabled skill bundle
- **WHEN** `skill_bundle.enabled` is `false`
- **THEN** skill installation is skipped entirely during runtime install

#### Scenario: Skill with specific version ref
- **WHEN** a skill entry specifies `ref: v2.1.0`
- **THEN** the installer clones that specific tag/commit

#### Scenario: Skill with branch ref
- **WHEN** a skill entry specifies `ref: main`
- **THEN** the installer clones the latest commit on that branch

### Requirement: Skill bundle configuration schema
The system SHALL define a `SkillBundle` struct that deserializes from the `skill_bundle` YAML section.

#### Scenario: Valid bundle config
- **WHEN** `skill_bundle` contains `enabled: true` and a non-empty `skills` list
- **THEN** deserialization succeeds and returns a populated `SkillBundle`

#### Scenario: Empty skills list
- **WHEN** `skill_bundle.enabled` is `true` but `skills` is empty
- **THEN** installation completes with zero skills installed and a log message: "no skills configured"

#### Scenario: Missing enabled field
- **WHEN** `skill_bundle` section exists but `enabled` is missing
- **THEN** system defaults `enabled` to `true`

### Requirement: Default skill bundle for Claude Code
The system SHALL ship with a default skill bundle for Claude Code containing curated, high-quality skill packs.

#### Scenario: First-time installation
- **WHEN** a user installs Claude Code for the first time
- **THEN** the default bundle includes: Claude Code official skills, Superpowers, gstack, Super Claude, and Everything Claude Code with pinned refs to stable versions

#### Scenario: User customizes the bundle
- **WHEN** a user edits `runtimes.yml` to add or remove skills from the bundle
- **THEN** the next `agent_install_skills` call respects the user's customized list
