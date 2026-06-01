## ADDED Requirements

### Requirement: Pre-turn snapshot
The system SHALL record project state before each turn execution: current git commit, branch, dirty status, and key file hashes.

#### Scenario: Git project snapshot
- **WHEN** turn starts in a git repository at commit abc123 on branch main with clean working tree
- **THEN** snapshot records: git_commit="abc123", git_branch="main", is_dirty=false

#### Scenario: Non-git project snapshot
- **WHEN** turn starts in a non-git project
- **THEN** snapshot records file hashes for all tracked source files (recursive SHA256 from project root)

### Requirement: Post-turn diff scan
The system SHALL scan file changes after each turn completion by comparing current state against pre-turn snapshot.

#### Scenario: Git project diff
- **WHEN** turn completes with 2 files modified and 1 file added
- **THEN** system runs `git diff --name-status` and records changes: 2 modified, 1 new file

#### Scenario: Non-git project diff
- **WHEN** turn completes in non-git project
- **THEN** system compares current file hashes against pre-turn hash snapshot to identify changes

### Requirement: File change records
The system SHALL persist file changes in agent_file_change table with path, change_type, diff_content, and hash before/after.

#### Scenario: Record modified file with diff
- **WHEN** turn modified src/main.rs with 15 lines added, 3 removed
- **THEN** agent_file_change record contains change_type="modified", diff_content with full unified diff

### Requirement: Per-turn rollback
The system SHALL support reverting all file changes from a single turn.

#### Scenario: Rollback turn in git project
- **WHEN** user rolls back turn "turn_008" in a git project
- **THEN** system uses `git checkout` to restore files changed in that turn

#### Scenario: Rollback turn in non-git project
- **WHEN** user rolls back turn "turn_008" in a non-git project
- **THEN** system restores files from pre-turn hash backup

### Requirement: Per-file rollback
The system SHALL support reverting changes to a single file from a turn.

#### Scenario: Rollback single file
- **WHEN** user rolls back only src/main.rs from turn "turn_008"
- **THEN** only src/main.rs is restored, other files remain unchanged

### Requirement: File boundary enforcement
The system SHALL limit runtime file access to the project root directory by default. Access outside project root requires explicit configuration.

#### Scenario: Runtime accesses file within project
- **WHEN** runtime writes to src/new_file.rs (within project root)
- **THEN** operation is allowed

#### Scenario: Runtime accesses file outside project
- **WHEN** runtime attempts to access ~/.ssh/id_rsa
- **THEN** access is flagged as audit violation with severity="critical"
