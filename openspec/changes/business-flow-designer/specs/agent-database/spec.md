## MODIFIED Requirements

### Requirement: Business flow tables migration
The database migration system SHALL execute a new migration `m20260604_001_create_business_flow_tables.sql` that creates 3 tables: `business_flows`, `flow_runs`, `flow_run_artifacts`.

#### Scenario: Migration creates business_flows table
- **WHEN** the migration runs
- **THEN** `business_flows` table is created with columns: id (TEXT PK), name (TEXT UNIQUE), description (TEXT), type (TEXT CHECK IN 'builtin','custom'), published (BOOLEAN DEFAULT FALSE), flow_json (TEXT NOT NULL), output_dir (TEXT), output_filename_pattern (TEXT), output_extension (TEXT), scenario_bindings (TEXT DEFAULT '[]'), yaml_export (TEXT), created_at (TEXT), updated_at (TEXT), published_at (TEXT), version (INTEGER DEFAULT 1)

#### Scenario: Migration creates flow_runs table
- **WHEN** the migration runs
- **THEN** `flow_runs` table is created with columns: id (TEXT PK), flow_id (TEXT FK), status (TEXT CHECK IN 'pending','running','completed','failed','aborted'), triggered_by (TEXT), material_paths (TEXT), started_at (TEXT), completed_at (TEXT), output_log (TEXT), error_message (TEXT)

#### Scenario: Migration creates flow_run_artifacts table
- **WHEN** the migration runs
- **THEN** `flow_run_artifacts` table is created with columns: id (TEXT PK), run_id (TEXT FK), node_id (TEXT), agent_id (TEXT), artifact_path (TEXT), artifact_type (TEXT), created_at (TEXT), checksum (TEXT)

#### Scenario: Foreign key constraints
- **WHEN** a flow_run references a non-existent flow
- **THEN** the insert is rejected by the foreign key constraint on flow_id

## ADDED Requirements

### Requirement: Business flow Rust module
The Rust backend SHALL include a `business_flow` module registered in `lib.rs` with sub-modules for: model (SeaORM entities), commands (Tauri IPC handlers), conductor (runtime executor), executor (node-level execution), validation (static + AI validation).

#### Scenario: Module registered in lib.rs
- **WHEN** the application compiles
- **THEN** `mod business_flow;` is declared and all Tauri commands from the module are registered in `invoke_handler`

### Requirement: Tauri IPC commands for flow operations
The backend SHALL expose these Tauri IPC commands: `list_flows`, `get_flow(id)`, `create_flow(flow)`, `update_flow(id, flow, version)`, `delete_flow(id)`, `publish_flow(id)`, `copy_flow(id)`, `validate_flow(id)`, `run_flow(id, config)`, `abort_run(run_id)`, `list_runs(flow_id)`, `get_run(run_id)`.

#### Scenario: List flows returns typed results
- **WHEN** frontend calls `list_flows`
- **THEN** a `Vec<FlowSummary>` is returned with id, name, type, published, scenario_bindings, created_at, updated_at

#### Scenario: Run flow returns run ID
- **WHEN** frontend calls `run_flow` with a flow ID and run configuration
- **THEN** the backend starts the Conductor and returns a `run_id` UUID for tracking
