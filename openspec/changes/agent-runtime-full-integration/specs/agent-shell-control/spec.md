## ADDED Requirements

### Requirement: Shell execution disabled by default
The system SHALL default to disallowing shell command execution for all runtimes. Shell execution must be explicitly enabled in project-level agent.yml.

#### Scenario: Shell execution blocked by default
- **WHEN** runtime attempts to execute any shell command and agent.yml has allow_agent_shell_in_project: false
- **THEN** execution is blocked and an audit entry with severity="warning" is created

### Requirement: Command allowlisting
The system SHALL support a whitelist of allowed shell commands configured in agent.yml sandbox.allowed_commands.

#### Scenario: Allowed command executes
- **WHEN** agent.yml allows "cargo" and "npm" and runtime attempts "cargo build"
- **THEN** command executes successfully

#### Scenario: Blocked command
- **WHEN** agent.yml allows "cargo" and "npm" and runtime attempts "rm -rf /"
- **THEN** command is blocked because "rm" is not in whitelist

### Requirement: Default blocked commands
The system SHALL always block dangerous commands regardless of allowlist: rm -rf, sudo, chmod, chown, curl piped to sh, wget piped to sh, ssh, scp, rsync, docker system prune, systemctl, modifying shell profiles.

#### Scenario: Dangerous command blocked
- **WHEN** runtime attempts "sudo make install"
- **THEN** "sudo" is blocked regardless of allowlist configuration, audit entry severity="critical"

### Requirement: Working directory restriction
The system SHALL restrict shell execution to the project root directory.

#### Scenario: Command within project root
- **WHEN** runtime executes "npm test" with working directory set to /project/
- **THEN** command runs from /project/

#### Scenario: Attempt to change to system directory
- **WHEN** runtime attempts "cd /etc && cat passwd"
- **THEN** directory change is blocked or flagged as audit violation

### Requirement: Command timeout
The system SHALL enforce a configurable timeout on shell command execution (default: 300 seconds).

#### Scenario: Command completes within timeout
- **WHEN** runtime executes "npm install" which takes 120 seconds
- **THEN** command completes and output is captured

#### Scenario: Command exceeds timeout
- **WHEN** runtime executes a command that runs longer than 300 seconds
- **THEN** process is killed and audit entry is created

### Requirement: Command audit logging
The system SHALL log all shell command execution attempts (allowed and blocked) to the audit log with: command, working directory, user, timestamp, and result.

#### Scenario: Log allowed command
- **WHEN** allowed command "cargo check" executes successfully
- **THEN** audit log entry created with action="shell_exec", severity="info", detail containing command and duration

#### Scenario: Log blocked command
- **WHEN** blocked command "rm -rf build/" is attempted
- **THEN** audit log entry created with action="shell_blocked", severity="critical", detail containing command
