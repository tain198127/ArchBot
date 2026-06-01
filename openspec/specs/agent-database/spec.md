## ADDED Requirements

### Requirement: Agent runtime table
The system SHALL persist agent runtime records with fields: id, runtime_type (claude_code/opencode/hermes/openclaw), enabled, mode, current_version, executable_path, adapter_config (JSON), provider_config (JSON), model_config (JSON), env_vars (JSON), execution_config (JSON), created_at, updated_at.

#### Scenario: Register a runtime
- **WHEN** system starts and loads runtimes.yml
- **THEN** each configured runtime is upserted into agent_runtime table

### Requirement: Agent runtime version table
The system SHALL persist runtime version records with fields: id, runtime_id (FK), version, install_path, checksum, status (installed/available/failed), installed_at, created_at.

#### Scenario: Record version installation
- **WHEN** a new runtime version is installed
- **THEN** a record is inserted into agent_runtime_version with status "installed"

### Requirement: Agent adapter table
The system SHALL persist adapter process records with fields: id, runtime_id (FK), adapter_type, host, port, auth_token_hash, status (running/stopped/failed), pid, started_at, stopped_at.

#### Scenario: Track adapter process
- **WHEN** an adapter process starts and binds to port 9101
- **THEN** agent_adapter record is created with port=9101, status="running", pid={pid}

### Requirement: Agent session table
The system SHALL persist sessions with fields: session_id (UUID PK), title, goal, project_id, runtime_type, default_model, current_state (text), status (active/paused/closed/archived), created_at, updated_at.

#### Scenario: Session persisted to database
- **WHEN** a session is created via SessionManager::create()
- **THEN** session is inserted into agent_session table and queryable by session_id

### Requirement: Agent turn table
The system SHALL persist turns with fields: turn_id (PK), session_id (FK), sequence_number, user_message, interpreted_intent, input_file_path, prompt_file_path, status (pending/running/streaming/completed/failed/cancelled/timeout), runtime_type, runtime_version, model, started_at, finished_at, error_message, duration_ms.

#### Scenario: Turn status transitions
- **WHEN** turn status changes from "pending" to "running" to "completed"
- **THEN** each status change is updated in agent_turn table

### Requirement: Agent event table
The system SHALL persist events with fields: event_id (PK), session_id, turn_id, event_type, sequence_number, payload (JSON), timestamp.

#### Scenario: Event stream replay
- **WHEN** querying events for turn "turn_001" ordered by sequence_number
- **THEN** system returns all events in chronological order for replay

### Requirement: Agent artifact table
The system SHALL persist artifacts with fields: artifact_id (PK), turn_id (FK), artifact_type, file_path, mime_type, size_bytes, created_at.

#### Scenario: Record generated artifact
- **WHEN** a turn generates result.md
- **THEN** an artifact record is created with artifact_type="turn_result", file_path pointing to the file

### Requirement: Agent file change table
The system SHALL persist file changes with fields: change_id (PK), turn_id (FK), file_path, change_type (created/modified/deleted), diff_content, file_hash_before, file_hash_after, size_before, size_after, created_at.

#### Scenario: Track modified file
- **WHEN** turn execution modifies src/main.rs
- **THEN** agent_file_change record is created with change_type="modified", diff_content populated

### Requirement: Agent decision table
The system SHALL persist key decisions with fields: decision_id (PK), session_id (FK), turn_id (FK, nullable), decision_text, status (proposed/accepted/rejected/superseded), rationale, created_at, superseded_by (nullable).

#### Scenario: Record accepted decision
- **WHEN** a turn proposes a decision and user accepts it
- **THEN** decision record status changes from "proposed" to "accepted"

### Requirement: Agent context snapshot table
The system SHALL persist context snapshots with fields: snapshot_id (PK), session_id (FK), turn_id (FK, nullable), snapshot_type, content (JSON), token_count, created_at.

#### Scenario: Save context snapshot before turn
- **WHEN** a turn starts
- **THEN** system saves a context snapshot containing sliding window messages, summary, and decision log

### Requirement: Agent audit log table
The system SHALL persist audit entries with fields: log_id (PK), session_id, turn_id, event_type, actor, action, detail (JSON), severity (info/warning/critical/block), created_at.

#### Scenario: Log permission violation
- **WHEN** runtime attempts to access a prohibited path
- **THEN** an audit log entry is created with severity="critical" and detail containing the path

### Requirement: SQL migration for agent tables
The system SHALL provide a migration file that creates all 10 agent tables with correct foreign keys and indexes.

#### Scenario: Migration runs successfully
- **WHEN** SeaORM migration is applied
- **THEN** all 10 agent tables exist in SQLite with proper schema

#### Scenario: Migration is idempotent
- **WHEN** migration is applied twice
- **THEN** second application is a no-op, no errors
