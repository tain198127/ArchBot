## 1. Foundation — Types & Data Model

- [ ] 1.1 Create `src/types/businessFlow.ts` with TypeScript interfaces: FlowDefinition, FlowNode, FlowEdge, FlowNodeData (per type), MaterialRef, ScenarioBinding, OutputConfig, QualityGateConfig, RunConfig, RunStatus, FlowRun, FlowRunArtifact
- [ ] 1.2 Create `src-tauri/src/business_flow/mod.rs` declaring sub-modules: model, commands, conductor, executor, validation
- [ ] 1.3 Create `src-tauri/src/business_flow/model.rs` with SeaORM entities: BusinessFlow, FlowRun, FlowRunArtifact (match TypeScript types)
- [ ] 1.4 Create SQL migration `src-tauri/src/db/migrations/m20260604_001_create_business_flow_tables.sql` with 3 tables (business_flows, flow_runs, flow_run_artifacts) including FK constraints and CHECK constraints
- [ ] 1.5 Register `mod business_flow;` in `src-tauri/src/lib.rs` and wire into `invoke_handler`

## 2. Backend CRUD — Tauri IPC Commands

- [ ] 2.1 Implement `list_flows` command — query all flows, return Vec<FlowSummary> with id, name, type, published, scenario_bindings
- [ ] 2.2 Implement `get_flow` command — load single flow by ID, deserialize flow_json into structured types
- [ ] 2.3 Implement `create_flow` command — insert new flow with UUID, serialize graph as JSON, version=1
- [ ] 2.4 Implement `update_flow` command — optimistic locking via version check, reject on conflict, increment version
- [ ] 2.5 Implement `delete_flow` command — reject for built-in flows, cascade delete runs and artifacts for custom flows
- [ ] 2.6 Implement `publish_flow` command — set published=true, published_at=now, reject if already published
- [ ] 2.7 Implement `copy_flow` command — deep clone flow with name "{Original} (Copy)", published=false, empty scenario_bindings
- [ ] 2.8 Implement `validate_flow` command — static validation (cycle detection, disconnected nodes, missing Start/End)
- [ ] 2.9 Implement `list_runs` and `get_run` commands — query flow_runs by flow_id, return run status and metadata

## 3. Frontend Store & Composables

- [ ] 3.1 Create `src/stores/flowStore.ts` (Pinia) — state: flows[], currentFlow, openTabs[], loading, error; actions: loadFlows, loadFlow, saveFlow, deleteFlow, publishFlow, copyFlow, validateFlow
- [ ] 3.2 Create `src/composables/useBusinessFlow.ts` — wraps flowStore actions, provides reactive computed properties for current flow, tab management, dirty state tracking
- [ ] 3.3 Create `src/composables/useFlowSceneMenus.ts` — computes context menu items from flows' scenario_bindings, registers/unregisters with ContextMenuResolver dynamically

## 4. Frontend — List Panel

- [ ] 4.1 Create `src/components/domain/BusinessFlowListPanel.vue` — table with columns: name, type badge (built-in/custom), file-type bindings (overflow with tooltip), published indicator
- [ ] 4.2 Add click handler on list rows → open editor tab via flowStore
- [ ] 4.3 Add "New Flow" button that creates a blank flow and opens editor tab
- [ ] 4.4 Add empty state component: "No business flows yet. Create one to get started."
- [ ] 4.5 Wire BusinessFlowListPanel into EditorPanel.vue as 'business-flow-list' tab type

## 5. Frontend — Editor Panel & Vue Flow Canvas

- [ ] 5.1 Install npm dependencies: `@vue-flow/core`, `@vue-flow/background`, `@vue-flow/controls`, `@vue-flow/minimap` (lazy-loaded)
- [ ] 5.2 Create `src/components/domain/BusinessFlowEditorPanel.vue` — layout with canvas (center), toolbar (left), action bar (bottom), binding config (top or sidebar)
- [ ] 5.3 Create `src/components/domain/business-flow/nodes/` — custom Vue Flow node components for each of the 12+ types (StartNode, EndNode, AgentNode, GatewayNode variants, MaterialInputNode, QualityGateNode, SubFlowNode, TimerNode, SignalNode, ErrorHandlerNode, HumanApprovalNode)
- [ ] 5.4 Create `src/components/domain/business-flow/edges/` — custom edge component with condition label, quality gate indicator, action badge
- [ ] 5.5 Implement drag-and-drop from toolbar → canvas: node palette icons spawn new nodes at drop position
- [ ] 5.6 Implement drag-and-drop from FileTree → canvas: dropped files create MaterialInput nodes with file reference
- [ ] 5.7 Implement node configuration modals — double-click opens config dialog per node type (AgentNode config is most complex with Manifest fields)
- [ ] 5.8 Implement edge configuration — click edge opens inline editor for action, condition, quality gate
- [ ] 5.9 Implement undo/redo using Vue Flow's history API or custom undo stack (50 operations)
- [ ] 5.10 Add mini-map component in canvas corner
- [ ] 5.11 Implement keyboard shortcuts: Delete (remove selected node/edge), Ctrl+Z (undo), Ctrl+Y (redo), Ctrl+S (save)
- [ ] 5.12 Wire BusinessFlowEditorPanel into EditorPanel.vue as 'business-flow-editor' tab type with flow ID prop

## 6. Frontend — Action Buttons

- [ ] 6.1 Implement Save button — validate structure (Start+End+connected), call update_flow IPC, show success/error toast
- [ ] 6.2 Implement Delete button — confirm dialog, reject for built-in flows, call delete_flow IPC
- [ ] 6.3 Implement Cancel button — check dirty state, confirm if unsaved, close tab
- [ ] 6.4 Implement Copy button — call copy_flow IPC, open new tab with copied flow
- [ ] 6.5 Implement Publish button — confirm dialog, call publish_flow IPC, set editor to read-only mode

## 7. Context Menu Integration

- [ ] 7.1 Modify `src/orchestration/ContextMenuResolver.ts` — add "Business Flows" submenu group, dynamically populate from flowStore based on file-type matching
- [ ] 7.2 Implement menu item click handler — open run configuration panel with clicked file as material
- [ ] 7.3 Add flow binding configuration UI in editor — multi-select file-type picker (e.g., ".java", ".py", "*.yml")
- [ ] 7.4 Implement dynamic menu refresh — watch flowStore changes, re-register context menus on save/delete/publish

## 8. Backend — Conductor Runtime

- [ ] 8.1 Create `src-tauri/src/business_flow/conductor.rs` — parse flow JSON into DAG, topological sort, spawn tokio tasks per node
- [ ] 8.2 Implement sequential execution — follow topological order, wait for each node to complete before next
- [ ] 8.3 Implement parallel execution at AND gateways — fork into concurrent tokio tasks, join at convergence point
- [ ] 8.4 Implement conditional branching at XOR/OR gateways — evaluate condition expressions, choose path(s)
- [ ] 8.5 Create `src-tauri/src/business_flow/executor.rs` — execute individual Agent nodes: invoke LLM via AI provider system, pass materials as input, write artifacts to isolated output dir
- [ ] 8.6 Implement Manifest enforcement — validate input_paths/output_paths/forbidden_paths per node, reject violations
- [ ] 8.7 Implement quality gate evaluation — check conditions against upstream output metrics, follow onFail policy
- [ ] 8.8 Implement degradation chain — warn → restart (up to maxRetries) → skip node → force convergence → terminate
- [ ] 8.9 Implement streaming events — emit Tauri events (node_started, node_completed, node_failed, quality_gate_result, flow_completed, flow_failed) via app.emit()
- [ ] 8.10 Implement abort mechanism — tokio::CancellationToken per run, cancel all node tasks on abort
- [ ] 8.11 Implement Sub-Flow execution — recursively load referenced flow, track depth, enforce max depth 3
- [ ] 8.12 Implement `run_flow` IPC command — start Conductor in tokio::spawn, return run_id immediately
- [ ] 8.13 Implement `abort_run` IPC command — signal cancellation token for the given run_id

## 9. Frontend — Run Panel & Streaming

- [ ] 9.1 Create run configuration dialog component — material file selector (pre-populated), output directory override, AI provider display, Confirm/Cancel
- [ ] 9.2 Create `src/components/domain/BusinessFlowRunPanel.vue` — right panel component showing streaming output
- [ ] 9.3 Implement Tauri event listener — listen for conductor events, update run panel in real-time
- [ ] 9.4 Implement per-node output sections — collapsible, with agent name, status badge, streaming text
- [ ] 9.5 Implement run status badge — pending → running → completed/failed/aborted with color coding
- [ ] 9.6 Implement abort button — call abort_run IPC, show abort confirmation
- [ ] 9.7 Implement run completion summary — show output file paths, artifact count, duration

## 10. AI Features — Validation & NL→BPMN

- [ ] 10.1 Implement AI validation — send flow definition to LLM, parse semantic issues, merge with static validation results
- [ ] 10.2 Implement validation result display — report panel listing all issues, severity levels, click-to-highlight on canvas
- [ ] 10.3 Implement NL→BPMN modal — large text area, system prompt builder (node types + agent list + context), JSON schema enforcement
- [ ] 10.4 Implement NL→BPMN response parsing — validate JSON against expected schema, create nodes/edges on canvas, show error on failure
- [ ] 10.5 Implement Validate button handler — run static + AI validation, display results

## 11. i18n & Polish

- [ ] 11.1 Add all business flow UI text to `src/i18n/en-US.ts` — section: businessFlow with subsections: list, editor, nodes, edges, actions, validation, run, errors
- [ ] 11.2 Add matching entries to `src/i18n/zh-CN.ts` — mirror key structure exactly
- [ ] 11.3 Apply dark theme — all components use Tailwind dark: variants, no `<style>` blocks
- [ ] 11.4 Apply keyboard accessibility — canvas nodes focusable, ARIA labels on all interactive elements, tab order follows logical flow
- [ ] 11.5 Add lazy loading — BusinessFlowEditorPanel and Vue Flow loaded via `defineAsyncComponent` to reduce initial bundle

## 12. Testing

- [ ] 12.1 Unit tests for flow validation logic (cycle detection, missing nodes, orphan edges)
- [ ] 12.2 Unit tests for Rust CRUD commands (create, read, update, delete, optimistic locking)
- [ ] 12.3 Unit tests for Conductor DAG parsing and topological sort
- [ ] 12.4 Integration test: full flow execution (Start → Agent → End) with mocked LLM provider
- [ ] 12.5 Integration test: context menu registration and file-type matching
- [ ] 12.6 E2E test: create flow, add nodes, save, validate, run (with mocked backend)
