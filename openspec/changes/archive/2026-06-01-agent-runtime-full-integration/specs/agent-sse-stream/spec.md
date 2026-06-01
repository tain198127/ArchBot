## ADDED Requirements

### Requirement: SSE event endpoint
The system SHALL provide an SSE endpoint at GET /api/agent/sessions/{session_id}/turns/{turn_id}/events that streams standard agent events to the frontend.

#### Scenario: Client connects to event stream
- **WHEN** frontend opens EventSource to /api/agent/sessions/sess_001/turns/turn_001/events
- **THEN** server responds with Content-Type: text/event-stream and keeps connection open

#### Scenario: Client receives events during turn execution
- **WHEN** a turn is executing and emitting events
- **THEN** connected SSE clients receive events in format "event: {type}\ndata: {json}\n\n"

### Requirement: Standard event types
The system SHALL support the 16 standard event types defined in the architecture: session.created, session.closed, turn.started, turn.delta, turn.reasoning_delta, turn.tool_started, turn.tool_delta, turn.tool_finished, turn.file_changed, turn.artifact_generated, turn.warning, turn.error, turn.completed, turn.failed, turn.cancelled, turn.timeout, runtime.started, runtime.health_changed, runtime.capability_changed, runtime.exited.

#### Scenario: Turn lifecycle events
- **WHEN** a turn executes from start to completion
- **THEN** SSE stream SHALL emit events in order: turn.started → turn.delta(s) → turn.tool_started → turn.tool_delta → turn.tool_finished → turn.file_changed → turn.artifact_generated → turn.completed

#### Scenario: Turn failure event
- **WHEN** turn execution fails with error "API rate limit exceeded"
- **THEN** SSE stream emits turn.error event with error details, then turn.failed

### Requirement: Event broadcast via tokio broadcast channel
The system SHALL use tokio::sync::broadcast for in-process event distribution to multiple SSE clients.

#### Scenario: Multiple clients receive same events
- **WHEN** two SSE clients are connected to the same turn's event stream
- **THEN** both clients receive identical event sequences

#### Scenario: Late client reconnection
- **WHEN** client reconnects with Last-Event-Id header
- **THEN** system replays events from after that event_id

### Requirement: Event persistence
The system SHALL persist each event to the agent_event database table upon emission.

#### Scenario: Event written to database
- **WHEN** a turn.delta event is emitted
- **THEN** the event is simultaneously written to agent_event table with session_id, turn_id, event_type, and payload

### Requirement: Event format
The system SHALL format each event with a standard envelope containing event_id, session_id, turn_id, runtime, event_type, timestamp, and payload.

#### Scenario: Event envelope validation
- **WHEN** any standard event is emitted
- **THEN** the event JSON MUST contain keys: event_id, session_id, turn_id, runtime, event_type, timestamp, payload
