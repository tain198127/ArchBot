## 1. Database Layer — Agent Tables

- [x] 1.1 Create migration file `m20260531_001_create_agent_tables.sql` with all 10 agent tables (agent_runtime, agent_runtime_version, agent_adapter, agent_session, agent_turn, agent_event, agent_artifact, agent_file_change, agent_decision, agent_context_snapshot, agent_audit_log)
- [x] 1.2 Create SeaORM entity files for all 10 tables under `src-tauri/src/db/entities/`
- [x] 1.3 Register new entities in `src-tauri/src/db/mod.rs` and update SeaORM migration registry
- [x] 1.4 Add DB handler CRUD operations in `src-tauri/src/handlers/agent_handler.rs` — session CRUD, turn CRUD, event query, artifact query
- [x] 1.5 Write unit tests for DB entity CRUD operations (in-memory SQLite)

## 2. Session Manager

- [x] 2.1 Create `src-tauri/src/agent_runtime/session_manager.rs` — AgentSession struct, SessionManager with create/query/pause/close/archive methods, state machine validation
- [x] 2.2 Implement session directory creation: `.archbot/agent/sessions/{session_id}/` with metadata file
- [x] 2.3 Implement turn creation within session: turn state machine, turn directory creation, sequence_number auto-increment
- [x] 2.4 Register Tauri commands: `agent_create_session`, `agent_list_sessions`, `agent_get_session`, `agent_update_session_status`, `agent_create_turn`
- [x] 2.5 Write unit tests for session state machine transitions and turn lifecycle

## 3. Adapter Manager

- [x] 3.1 Create `src-tauri/src/agent_runtime/adapter_manager.rs` — `AgentAdapter` trait definition with async methods (health, capabilities, version, execute_turn, cancel_turn, stream_events)
- [x] 3.2 Implement `ClaudeCodeAdapter` struct — wraps Claude Code CLI via launcher module
- [x] 3.3 Implement `OpenCodeAdapter`, `HermesAdapter`, `OpenClawAdapter` stubs — health check + CLI invocation skeleton (full CLI integration deferred)
- [x] 3.4 Create `AdapterRegistry` — maps runtime_type strings to adapter instances, adapter selection by config
- [x] 3.5 Refactor `turn_executor.rs` to use Adapter trait instead of directly calling launcher
- [x] 3.6 Register Tauri commands: `agent_check_runtime_health`, `agent_get_runtime_capabilities`
- [x] 3.7 Write unit tests for adapter selection and ClaudeCodeAdapter health check

## 4. SSE Event Stream

- [x] 4.1 Create `src-tauri/src/agent_runtime/event_stream.rs` — StandardEvent struct, event type enum (16 types), tokio broadcast channel setup
- [x] 4.2 Implement `EventBus` — global broadcast channel, publish/subscribe, event persistence to DB on publish
- [x] 4.3 Create Axum SSE handler: `GET /api/agent/sessions/{session_id}/turns/{turn_id}/events` with Last-Event-Id replay support
- [x] 4.4 Wire EventBus into turn_executor.rs — emit events at each execution stage (turn.started, turn.delta, turn.tool_*, turn.completed, etc.)
- [x] 4.5 Register SSE route in `src-tauri/src/server.rs` router
- [x] 4.6 Write integration test: verify SSE stream receives all expected events during a turn execution

## 5. Version Manager

- [x] 5.1 Create `src-tauri/src/agent_runtime/version_manager.rs` — version detection, download manager, checksum verification, extract, symlink switch
- [x] 5.2 Implement `RuntimeInstaller` — download from configured URL, SHA256 verify, extract to versions/ dir
- [x] 5.3 Implement version validation (executable --version check) and rollback (symlink switch to previous)
- [x] 5.4 Replace stubs in `agent_config_handler.rs` — wire `agent_install` and `agent_update` commands to version_manager
- [x] 5.5 Register Tauri commands: `agent_list_versions`, `agent_install`, `agent_update`, `agent_rollback`
- [x] 5.6 Write unit tests for version detection, symlink switching, and rollback logic

## 6. Context Assembly

- [x] 6.1 Create `src-tauri/src/agent_runtime/context_assembly.rs` — ContextAssembler with sliding window selection, summary storage, decision log merge
- [x] 6.2 Implement `assemble_turn_input()` — builds complete input.yml from all context sections (session header, current turn, recent messages, compressed history, decision log, working context, execution policy)
- [x] 6.3 Implement context snapshot persistence — save context snapshot to agent_context_snapshot table before each turn
- [x] 6.4 Implement context update from turn output — parse context_update.yml and update session working context
- [x] 6.5 Wire ContextAssembler into SessionManager — session-level context tracking, per-turn assembly
- [x] 6.6 Register Tauri commands: `agent_get_session_context`, `agent_update_context`

## 7. File Change Control

- [x] 7.1 Create `src-tauri/src/agent_runtime/file_control.rs` — SnapshotManager with pre-turn git snapshot and file hash snapshot
- [x] 7.2 Implement post-turn diff scan — git diff parsing for git repos, hash comparison for non-git projects
- [x] 7.3 Implement rollback — per-turn git checkout/revert, per-file hash-based restore
- [x] 7.4 Implement file boundary enforcement — path validation against project root
- [x] 7.5 Wire FileControl into turn_executor.rs — snapshot before execution, diff after execution
- [x] 7.6 Register Tauri commands: `agent_get_file_changes`, `agent_rollback_turn`, `agent_rollback_file`
- [x] 7.7 Write unit tests for diff parsing and rollback logic

## 8. Shell Allowlisting

- [x] 8.1 Create `src-tauri/src/agent_runtime/shell_control.rs` — CommandGuard with default blocked commands list, per-project allowlist
- [x] 8.2 Implement command validation — check against blocked list (always blocked) and allowlist (must be in both to pass)
- [x] 8.3 Implement working directory enforcement and command timeout
- [x] 8.4 Write unit tests for allowlist/blocklist matching and validation edge cases

## 9. Frontend Panels

- [x] 9.1 Create `src/stores/agentStore.ts` — Pinia store for agent state (sessions, turns, events, file changes, audit log), Tauri invoke wrappers
- [x] 9.2 Create `src/components/domain/AgentSessionPanel.vue` — session list, create/close/archive actions, session metadata display
- [x] 9.3 Create `src/components/domain/AgentTurnPanel.vue` — turn list within session, message input, turn status display, result.md rendering
- [x] 9.4 Create `src/components/domain/AgentEventStreamPanel.vue` — EventSource SSE client, real-time event list with type filtering and color coding
- [x] 9.5 Create `src/components/domain/AgentDiffReviewPanel.vue` — file change list, unified diff view with accept/reject buttons
- [x] 9.6 Create `src/components/domain/AgentAuditLogPanel.vue` — audit entries with severity filter, search, date range
- [x] 9.7 Integrate agent panels into `BottomPanel.vue` tab system — add "Agent" tab group
- [x] 9.8 Add i18n entries to `src/i18n/en-US.ts` and `src/i18n/zh-CN.ts` for all new panel strings

## 10. Integration, Testing & Documentation

- [x] 10.1 Update `src-tauri/src/agent_runtime/mod.rs` — register all new submodules
- [x] 10.2 Update `src-tauri/src/lib.rs` — register all new Tauri commands
- [x] 10.3 Run `cargo check` and `cargo clippy` — fix all warnings
- [x] 10.4 Run `cargo test` — ensure all unit tests pass with 80%+ coverage on new modules
- [x] 10.5 Run `cargo fmt` — format all Rust source files
- [ ] 10.6 End-to-end test: create session → execute turn → verify SSE events → verify file changes → verify audit log
- [ ] 10.7 Update `prd.yml` with agent control plane integration details
- [ ] 10.8 Update `function-map.yml` with new module mappings
- [ ] 10.9 Security review: verify no hardcoded secrets, input validation at boundaries, audit log completeness
