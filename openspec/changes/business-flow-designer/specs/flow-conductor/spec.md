## ADDED Requirements

### Requirement: DAG-based flow execution
The Conductor SHALL parse a flow definition into a Directed Acyclic Graph, perform topological sort, and execute nodes in dependency order. At AND gateway forks, parallel branches SHALL execute concurrently via tokio::spawn.

#### Scenario: Sequential flow execution
- **WHEN** a flow with Start → Agent1 → Agent2 → End is executed
- **THEN** Agent1 runs first, waits for completion, then Agent2 runs

#### Scenario: Parallel gateway execution
- **WHEN** a flow reaches an AND gateway with 2 outgoing edges
- **THEN** both branches execute concurrently in separate tokio tasks

### Requirement: Streaming output via Tauri Event
The Conductor SHALL stream execution events to the frontend via Tauri `app.emit()`. Events SHALL include: node_started, node_completed, node_failed, quality_gate_result, flow_completed, flow_failed. The frontend SHALL render these events in real-time in the right-side output panel.

#### Scenario: Streaming node output
- **WHEN** an Agent node is executing and producing output
- **THEN** each output chunk is emitted as a Tauri event and displayed in the right panel within 200ms

#### Scenario: Flow completion event
- **WHEN** all nodes in a flow have completed
- **THEN** a `flow_completed` event is emitted with the final output paths

### Requirement: Quality gate enforcement
The Conductor SHALL evaluate quality gate nodes by checking configured conditions against upstream node outputs. If a quality gate fails, the Conductor SHALL follow the configured `onFail` action: retry, skip, or abort.

#### Scenario: Quality gate passes
- **WHEN** a quality gate checks `quality_score >= 0.8` and the upstream output has `quality_score = 0.9`
- **THEN** execution continues to the next node

#### Scenario: Quality gate fails with retry
- **WHEN** a quality gate checks `quality_score >= 0.8` and the upstream output has `quality_score = 0.6`, and onFail is "retry"
- **THEN** the upstream Agent node is re-executed up to maxRetries times

### Requirement: Degradation chain
The Conductor SHALL implement a 5-step degradation chain when an Agent node fails: warn → restart agent → skip node → force convergence → terminate flow. Each step SHALL be logged.

#### Scenario: Agent timeout triggers restart
- **WHEN** an Agent node exceeds its configured timeout
- **THEN** the Conductor logs a warning, restarts the agent with fresh input

#### Scenario: Max retries exceeded triggers skip
- **WHEN** an Agent node fails after maxRetries attempts
- **THEN** the Conductor skips the node and continues execution, logging the skip

### Requirement: Immutable artifact communication
Each Agent node execution SHALL write output to an isolated directory `{output_dir}/{run_id}/{node_id}/`. Downstream nodes SHALL only read from these directories. The Conductor SHALL enforce isolation via Manifest declarations (input_paths, output_paths, forbidden_paths).

#### Scenario: Agent writes to isolated directory
- **WHEN** Agent node "reviewer" completes execution in run "abc-123"
- **THEN** output artifacts are written to `{output_dir}/abc-123/reviewer/`

#### Scenario: Downstream node reads upstream output
- **WHEN** Agent node "writer" starts with input_path pointing to reviewer's output
- **THEN** writer reads the artifacts from `{output_dir}/abc-123/reviewer/` as read-only

### Requirement: Run tracking and status management
The system SHALL track each flow execution as a `flow_run` record with status progression: pending → running → completed/failed/aborted. The `started_at` and `completed_at` timestamps SHALL be recorded.

#### Scenario: Run starts with pending status
- **WHEN** a flow execution is initiated
- **THEN** a `flow_run` record is created with status "pending" and `started_at` set to current time

#### Scenario: Run completes successfully
- **WHEN** all nodes in a flow complete without errors
- **THEN** the run status is updated to "completed" and `completed_at` is set

### Requirement: Run configuration panel
Before executing a flow, the system SHALL display a run configuration panel with: material file selector (pre-populated from flow bindings), output directory override (optional), AI provider display (project default, read-only), and Confirm/Cancel buttons.

#### Scenario: Open run configuration
- **WHEN** user clicks the Run button
- **THEN** a configuration panel opens showing pre-populated materials, output settings, and a Confirm button

#### Scenario: Override output directory
- **WHEN** user changes the output directory in the run config panel and clicks Confirm
- **THEN** the flow executes with the overridden output directory

### Requirement: Abort running flow
The system SHALL allow aborting a running flow. When aborted, all active node tasks SHALL be cancelled, the run status SHALL be set to "aborted", and a cleanup SHALL occur.

#### Scenario: User aborts flow
- **WHEN** user clicks the Abort button during flow execution
- **THEN** all running node tasks are cancelled, run status becomes "aborted", partial artifacts are preserved

### Requirement: Sub-Flow execution with depth limit
The Conductor SHALL support Sub-Flow nodes that reference another flow. Sub-Flow execution SHALL be limited to a maximum depth of 3. Cycle detection SHALL be performed at save time.

#### Scenario: Execute sub-flow
- **WHEN** the Conductor encounters a Sub-Flow node referencing flow "sub-review"
- **THEN** "sub-review" is loaded and executed as a nested flow within the current run

#### Scenario: Sub-Flow depth exceeded
- **WHEN** a Sub-Flow nesting exceeds depth 3
- **THEN** the Conductor terminates the flow with an error "Maximum sub-flow depth exceeded"
