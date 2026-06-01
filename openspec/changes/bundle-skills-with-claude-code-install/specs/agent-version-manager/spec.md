## MODIFIED Requirements

### Requirement: Runtime installation
The system SHALL download and install a runtime version to ~/.archbot/runtimes/{name}/versions/{version}/. After successful binary installation, if the runtime has an enabled skill bundle, the system SHALL automatically install the configured skill packs into the runtime's isolated HOME directory.

#### Scenario: Install new version
- **WHEN** user triggers install for "claude_code" version "2.1.128"
- **THEN** system downloads the package, verifies SHA256 checksum, extracts to versions/2.1.128/, runs {executable} --version to validate, then installs the skill bundle (if enabled) into the isolated HOME's `.claude/skills/` directory

#### Scenario: Failed checksum verification
- **WHEN** downloaded package SHA256 does not match expected checksum
- **THEN** system deletes the download, marks version as "failed", and returns checksum mismatch error (skill installation is NOT attempted)

#### Scenario: Binary install succeeds but skill install partially fails
- **WHEN** binary installation and validation succeed but one or more skills fail to clone
- **THEN** the runtime installation is still marked as successful, and the result includes a list of failed skills with error messages

#### Scenario: Skill bundle disabled
- **WHEN** the runtime's `skill_bundle.enabled` is `false` or no `skill_bundle` section exists
- **THEN** installation proceeds as before without skill installation (backward compatible)
