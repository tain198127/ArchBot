## ADDED Requirements

### Requirement: Create agent session
The system SHALL allow creating an AgentSession with a title, goal, default runtime type, and project association.

#### Scenario: Create session with required fields
- **WHEN** user provides title "Implement auth module", goal "Add JWT auth to API", and runtime "claude_code"
- **THEN** system creates session with status "active", assigns a UUID session_id, and returns the session

#### Scenario: Create session with missing title
- **WHEN** user provides empty title
- **THEN** system returns validation error "title is required"

### Requirement: Query agent sessions
The system SHALL support listing all sessions for a project, filtered by status.

#### Scenario: List active sessions
- **WHEN** user requests sessions with status filter "active"
- **THEN** system returns only sessions with status "active", ordered by updated_at descending

#### Scenario: List all sessions without filter
- **WHEN** user requests sessions without status filter
- **THEN** system returns all sessions for the project

### Requirement: Session lifecycle state machine
The system SHALL enforce valid state transitions: active→paused, active→closed, paused→active, paused→closed, closed→archived.

#### Scenario: Pause an active session
- **WHEN** session is "active" and user pauses it
- **THEN** session status changes to "paused"

#### Scenario: Invalid transition
- **WHEN** session is "closed" and user attempts to set it "active"
- **THEN** system returns error "invalid transition: closed → active"

### Requirement: Create agent turn
The system SHALL allow creating an AgentTurn within a session, linked to a user message and runtime invocation.

#### Scenario: Start a new turn
- **WHEN** user sends "Refactor UserService" to session "sess_001" with runtime "claude_code"
- **THEN** system creates turn with status "pending", turn directory at .archbot/agent/sessions/sess_001/turns/{turn_id}/, and returns turn object

#### Scenario: Create turn in closed session
- **WHEN** user attempts to create turn in a session with status "closed"
- **THEN** system returns error "cannot create turn in closed session"

### Requirement: Turn lifecycle state machine
The system SHALL enforce turn states: pending→running→streaming→completed/failed/cancelled/timeout.

#### Scenario: Complete turn successfully
- **WHEN** turn execution finishes without error
- **THEN** turn status changes to "completed" and finished_at is set

#### Scenario: Turn timeout
- **WHEN** turn execution exceeds configured timeout
- **THEN** turn status changes to "timeout" and error_message records the timeout

### Requirement: Session directory structure
The system SHALL create and manage the directory structure at .archbot/agent/sessions/{session_id}/turns/{turn_id}/ for each turn.

#### Scenario: Session directory on first turn
- **WHEN** first turn is created for session "sess_001"
- **THEN** system creates .archbot/agent/sessions/sess_001/ and .archbot/agent/sessions/sess_001/turns/{turn_id}/

#### Scenario: Turn directory contains required files
- **WHEN** turn execution completes
- **THEN** turn directory SHALL contain input.yml, prompt.txt, stdout.log, stderr.log, events.jsonl, result.md, result.json
