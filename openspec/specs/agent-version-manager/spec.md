## ADDED Requirements

### Requirement: Runtime version detection
The system SHALL detect installed runtime versions by scanning ~/.archbot/runtimes/{name}/versions/ directory.

#### Scenario: Detect installed Claude Code versions
- **WHEN** versions/ contains directories 2.1.100/ and 2.1.128/
- **THEN** system returns ["2.1.100", "2.1.128"] as installed versions

#### Scenario: No versions installed
- **WHEN** versions/ directory is empty or missing
- **THEN** system returns empty list and marks runtime as "not installed"

### Requirement: Runtime installation
The system SHALL download and install a runtime version to ~/.archbot/runtimes/{name}/versions/{version}/.

#### Scenario: Install new version
- **WHEN** user triggers install for "claude_code" version "2.1.128"
- **THEN** system downloads the package, verifies SHA256 checksum, extracts to versions/2.1.128/, and runs {executable} --version to validate

#### Scenario: Failed checksum verification
- **WHEN** downloaded package SHA256 does not match expected checksum
- **THEN** system deletes the download, marks version as "failed", and returns checksum mismatch error

### Requirement: Symbolic link switching
The system SHALL manage a `current` symlink pointing to the active version directory.

#### Scenario: Switch to new version
- **WHEN** user upgrades claude_code from 2.1.100 to 2.1.128
- **THEN** current symlink changes from versions/2.1.100/ to versions/2.1.128/

### Requirement: Version rollback
The system SHALL support rolling back to a previous version by switching the current symlink.

#### Scenario: Rollback after failed upgrade
- **WHEN** user requests rollback for claude_code
- **THEN** current symlink is changed to point to the previous version directory, and rollback is logged

#### Scenario: No previous version for rollback
- **WHEN** only one version is installed and user requests rollback
- **THEN** system returns error "no previous version available for rollback"

### Requirement: Minimum viability check
The system SHALL run a minimum viability test (--version or --help) after installing or upgrading a runtime.

#### Scenario: Installed runtime is broken
- **WHEN** runtime --version returns non-zero exit code or crashes
- **THEN** system marks version as "failed", does NOT switch current symlink, and returns error

### Requirement: Tauri commands for version management
The system SHALL expose version management operations as Tauri commands: `agent_list_versions`, `agent_install`, `agent_update`, `agent_rollback`.

#### Scenario: Frontend triggers install
- **WHEN** Vue frontend calls `invoke("agent_install", { runtime: "claude_code", version: "2.1.128" })`
- **THEN** backend executes install flow and returns status
