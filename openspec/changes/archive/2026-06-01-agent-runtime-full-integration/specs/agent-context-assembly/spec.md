## ADDED Requirements

### Requirement: Sliding window message selection
The system SHALL maintain a configurable sliding window of recent messages (default N=10) for each session to include in turn context.

#### Scenario: Sliding window with more messages than N
- **WHEN** session has 25 recent messages and sliding window size is 10
- **THEN** only the most recent 10 messages are included in the turn input

#### Scenario: Sliding window with fewer messages than N
- **WHEN** session has 3 messages and sliding window size is 10
- **THEN** all 3 messages are included

### Requirement: Historical summary generation
The system SHALL generate and store a compressed summary of messages that fall outside the sliding window.

#### Scenario: Generate summary after window overflow
- **WHEN** a session's messages exceed the sliding window size
- **THEN** messages falling outside the window are summarized and stored as compressed_history in the session

### Requirement: Decision log maintenance
The system SHALL maintain a running decision log for each session, appending new decisions from turn output (decision_candidates.yml).

#### Scenario: Append decisions from turn
- **WHEN** a turn completes and decision_candidates.yml contains 2 new decisions
- **THEN** decisions are added to session decision log with status "proposed"

#### Scenario: Update decision status
- **WHEN** user changes decision D001 status from "proposed" to "accepted"
- **THEN** decision is updated in both database and in-memory log

### Requirement: Context snapshot persistence
The system SHALL save a context snapshot before each turn containing: sliding window messages, historical summary, decision log, and working context.

#### Scenario: Snapshot before turn
- **WHEN** turn "turn_005" starts
- **THEN** a context snapshot is created with all current context sections, serialized to JSON, and persisted to agent_context_snapshot table

### Requirement: Context assembly for turn input
The system SHALL assemble the complete context for each turn into the input.yml structure, merging: current user message, interpreted intent, sliding window messages, compressed history, decision log, and working context (relevant files, artifacts).

#### Scenario: Full context assembly
- **WHEN** SessionManager prepares turn input for session "sess_001"
- **THEN** resulting input.yml contains all 7 context sections as defined in the architecture §13

### Requirement: Working context update from turn
The system SHALL update the session's working context based on context_update.yml output from completed turns.

#### Scenario: Context update after turn
- **WHEN** turn completes and context_update.yml specifies new relevant files and updated current_state
- **THEN** session's working_context is updated with the new files and current_state
