## ADDED Requirements

### Requirement: Adapter trait definition
The system SHALL define a unified `AgentAdapter` trait that all runtime adapters implement, with methods for health check, capabilities query, turn execution, and event streaming.

#### Scenario: Adapter trait methods
- **WHEN** a runtime adapter implements AgentAdapter trait
- **THEN** it SHALL provide: `health()`, `capabilities()`, `version()`, `execute_turn()`, `cancel_turn()`, `stream_events()`

### Requirement: Claude Code adapter
The system SHALL provide a `ClaudeCodeAdapter` implementing AgentAdapter trait that wraps Claude Code CLI invocation.

#### Scenario: Execute turn via Claude Code
- **WHEN** ArchBot calls `adapter.execute_turn(config)` on ClaudeCodeAdapter
- **THEN** adapter launches `claude` CLI with --input-file, --prompt-file, --output-dir args and returns TurnResult

#### Scenario: Claude Code health check
- **WHEN** ArchBot calls `adapter.health()` on ClaudeCodeAdapter
- **THEN** adapter runs `claude --version` and returns health status with version info

### Requirement: OpenCode adapter
The system SHALL provide an `OpenCodeAdapter` implementing AgentAdapter trait.

#### Scenario: OpenCode health check
- **WHEN** ArchBot calls `adapter.health()` on OpenCodeAdapter
- **THEN** adapter runs `opencode --version` and returns health status

### Requirement: Hermes adapter
The system SHALL provide a `HermesAdapter` implementing AgentAdapter trait with virtualenv-based execution.

#### Scenario: Hermes health check
- **WHEN** ArchBot calls `adapter.health()` on HermesAdapter
- **THEN** adapter checks hermes executable and venv status, returns health status

### Requirement: OpenClaw adapter
The system SHALL provide an `OpenClawAdapter` implementing AgentAdapter trait.

#### Scenario: OpenClaw health check
- **WHEN** ArchBot calls `adapter.health()` on OpenClawAdapter
- **THEN** adapter runs `openclaw --version` and returns health status

### Requirement: Adapter selection by runtime type
The system SHALL select the correct adapter based on runtime type string ("claude_code", "opencode", "hermes", "openclaw").

#### Scenario: Select Claude Code adapter
- **WHEN** runtime type is "claude_code"
- **THEN** system returns ClaudeCodeAdapter instance

#### Scenario: Unknown runtime type
- **WHEN** runtime type is "unknown_runtime"
- **THEN** system returns error "unsupported runtime type: unknown_runtime"

### Requirement: Adapter HTTP+SSE facade
The system SHALL expose adapter operations through the standard HTTP API defined in the architecture (POST /v1/sessions/{id}/turns, GET /v1/sessions/{id}/turns/{id}/events) as a facade layer, even though adapters run in-process in v1.

#### Scenario: HTTP facade forwards to adapter trait
- **WHEN** HTTP POST /v1/sessions/sess_001/turns is called
- **THEN** request is routed to the selected adapter's execute_turn() method

### Requirement: Adapter error standardization
The system SHALL convert all adapter-level errors to ArchBot standard error codes.

#### Scenario: Runtime not found error
- **WHEN** adapter cannot find runtime executable
- **THEN** system returns standard error "RUNTIME_NOT_FOUND" with descriptive message

#### Scenario: Turn execution timeout
- **WHEN** turn execution exceeds timeout
- **THEN** system returns standard error "TURN_TIMEOUT"
