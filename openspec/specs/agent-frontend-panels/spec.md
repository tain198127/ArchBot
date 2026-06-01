## ADDED Requirements

### Requirement: Agent Session panel
The system SHALL provide an AgentSessionPanel component that displays session list, allows creating/closing/archiving sessions, and shows session metadata (title, goal, runtime, status, turn count, last activity).

#### Scenario: Create session from panel
- **WHEN** user clicks "New Session" and fills in title, goal, and selects runtime
- **THEN** new session appears in the session list with status "active"

#### Scenario: View session details
- **WHEN** user clicks a session row
- **THEN** panel shows session metadata including created_at, turn count, current state

### Requirement: Agent Turn panel
The system SHALL provide an AgentTurnPanel component that shows turns within a session, allows sending new messages to create turns, and displays turn status and results.

#### Scenario: Send message to create turn
- **WHEN** user types "Analyze the authentication module" and clicks Send in a session
- **THEN** a new turn is created, execution starts, and turn appears with status "running"

#### Scenario: View turn result
- **WHEN** a turn completes with result.md generated
- **THEN** panel renders the result.md as formatted markdown

### Requirement: Agent Event Stream panel
The system SHALL provide an AgentEventStreamPanel that connects to the SSE endpoint and displays real-time events with filtering by event type.

#### Scenario: Real-time event display
- **WHEN** a turn is executing
- **THEN** EventStreamPanel shows events appearing in real-time: turn.delta text, tool calls, file changes

#### Scenario: Filter events by type
- **WHEN** user selects filter "turn.tool_started" and "turn.tool_finished"
- **THEN** only tool-related events are displayed

#### Scenario: Event color coding
- **WHEN** events are displayed
- **THEN** different event types SHALL have distinct visual treatment: errors in red, warnings in yellow, deltas in default, completions in green

### Requirement: Agent Diff Review panel
The system SHALL provide an AgentDiffReviewPanel that shows file changes from a turn with side-by-side or unified diff view.

#### Scenario: View turn file changes
- **WHEN** user selects a completed turn with 3 file changes
- **THEN** panel shows list of changed files with change type icons (created/modified/deleted)

#### Scenario: View file diff
- **WHEN** user clicks a modified file
- **THEN** panel displays the unified diff with added lines in green, removed lines in red

#### Scenario: Accept/reject changes
- **WHEN** user reviews a file change
- **THEN** user can accept (keep) or reject (rollback) individual file changes

### Requirement: Agent Audit Log panel
The system SHALL provide an AgentAuditLogPanel that displays audit entries with filtering by severity, session, turn, and date range.

#### Scenario: View critical audit entries
- **WHEN** user filters audit log by severity "critical"
- **THEN** only critical entries are shown with timestamp, session_id, turn_id, and detail

#### Scenario: Search audit log
- **WHEN** user searches for "file access" in audit log
- **THEN** entries matching the search term in action or detail are displayed

### Requirement: Panel integration with BottomPanel
The system SHALL integrate agent panels into the existing BottomPanel tab system alongside log and context tabs.

#### Scenario: Agent panels as tabs
- **WHEN** user opens BottomPanel
- **THEN** "Agent" tab group is available containing Session, Event Stream, Diff Review, and Audit Log sub-tabs

### Requirement: i18n for all agent panels
The system SHALL use vue-i18n for all user-visible text in agent panels, with entries in both zh-CN.ts and en-US.ts.

#### Scenario: Chinese locale
- **WHEN** locale is set to zh-CN
- **THEN** all panel labels, buttons, and messages display in Chinese

#### Scenario: English locale
- **WHEN** locale is set to en-US
- **THEN** all panel labels, buttons, and messages display in English
