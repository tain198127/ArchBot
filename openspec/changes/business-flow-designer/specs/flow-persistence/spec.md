## ADDED Requirements

### Requirement: SQLite 3-table schema for flows
The system SHALL create 3 database tables: `business_flows` (flow definitions with JSON graph column), `flow_runs` (execution tracking), `flow_run_artifacts` (per-node output files). The `business_flows` table SHALL include `published` boolean and `version` integer for optimistic locking.

#### Scenario: Database migration runs on startup
- **WHEN** the application starts
- **THEN** migration `m20260604_001_create_business_flow_tables.sql` creates the 3 tables if they do not exist

#### Scenario: Flow CRUD operations
- **WHEN** user saves a new flow
- **THEN** a row is inserted into `business_flows` with a UUID, the graph serialized as JSON, and version set to 1

#### Scenario: Optimistic locking on save
- **WHEN** user saves a flow that was modified by another tab
- **THEN** the save is rejected with a version conflict error and the user is notified

### Requirement: Tauri IPC commands for flow management
The system SHALL expose the following Tauri IPC commands: `list_flows`, `get_flow`, `create_flow`, `update_flow`, `delete_flow`, `publish_flow`, `copy_flow`, `validate_flow`.

#### Scenario: List all flows
- **WHEN** frontend calls `list_flows`
- **THEN** all flows are returned with id, name, type, published status, scenario_bindings summary

#### Scenario: Delete custom flow
- **WHEN** frontend calls `delete_flow` with a custom flow's ID
- **THEN** the flow and all associated data are removed from the database

#### Scenario: Delete built-in flow rejected
- **WHEN** frontend calls `delete_flow` with a built-in flow's ID
- **THEN** the command returns an error "Built-in flows cannot be deleted"

### Requirement: Flow definition serialization
The system SHALL serialize flow definitions as JSON in the `flow_json` column. The JSON SHALL contain `nodes` (array of FlowNode objects) and `edges` (array of FlowEdge objects). Each node SHALL have `id`, `type`, `position`, and `data`. Each edge SHALL have `id`, `source`, `target`, and optional `action`, `condition`, `qualityGate`, `label`.

#### Scenario: Round-trip serialization
- **WHEN** a flow is saved and then loaded
- **THEN** the loaded flow's nodes and edges are identical to the saved version

### Requirement: Publish and lock mechanism
The system SHALL support publishing flows. When a flow is published (`published = true`), it SHALL become immutable — no edits, no deletion. Users MUST copy the flow to make changes. The `published_at` timestamp SHALL be set on publish.

#### Scenario: Publish a flow
- **WHEN** user clicks publish on a saved flow
- **THEN** the flow's `published` field is set to true, `published_at` is set to current timestamp, and the editor becomes read-only

#### Scenario: Attempt to edit published flow
- **WHEN** user tries to modify a published flow's canvas
- **THEN** the edit is blocked and a message shows: "This flow is published. Copy it to make changes."

#### Scenario: Copy published flow
- **WHEN** user clicks copy on a published flow
- **THEN** a new custom flow is created with name "{Original} (Copy)", containing the same nodes and edges but with `published = false` and empty `scenario_bindings`

### Requirement: YAML import/export (P2)
The system SHALL support exporting flow definitions as YAML and importing YAML files as new flows. This is P2 priority — the data model reserves the `yaml_export` field but the UI is not implemented in MVP.

#### Scenario: Export flow as YAML
- **WHEN** user exports a flow
- **THEN** a YAML file is generated containing the flow's nodes, edges, and configuration
