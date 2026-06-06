## Context

ArchBot is a Tauri 2 desktop app (Vue 3 + Rust) for managing digital employees (硅基军团). The current orchestration layer consists of:

- **FlowExecutor.ts** — sequential YAML-defined pipelines (e.g., `analysis.fullControllerTrace`)
- **ContextMenuResolver.ts** — YML-driven right-click menus
- **EditorPanel.vue** — tab-based center panel that dispatches config actions
- **useScenario.ts** — scenario dimension system with `businessFlow: string[]` field

The "Config → Business Flow" menu entry exists but is a dead end — EditorPanel.vue has no handler for `config.businessFlow`. The Rust backend has an empty `business_flow/` directory with no `mod.rs`. No database tables, no Vue components, no store.

The user's multi-agent design methodology (`multi-agent-design-methodology.md`) defines 4 principles that MUST guide the runtime design: scope-first, unique identity, immutable artifacts, spec-as-contract.

## Goals / Non-Goals

**Goals:**
- Visual BPMN flow editor with 12+ node types using simplified flow model
- Multi-tab editing (one flow per tab) with drag-and-drop materials
- Dynamic context menu integration on FileTree nodes (file-type-based binding)
- Rust Conductor runtime with tokio::spawn, streaming output, and degradation chain
- Publish/lock mechanism — published flows are immutable, copy-to-edit
- NL→BPMN generation and AI-powered validation via project default LLM provider
- Run configuration panel before execution

**Non-Goals:**
- Full BPMN 2.0 compliance (no pools, lanes, choreography, or BPMN XML export)
- Per-flow or per-node AI provider configuration (uses project default)
- Real-time collaborative editing (single-user desktop app)
- Flow versioning/history beyond optimistic locking
- Cloud-based flow sharing (YAML import/export is P2, local only)
- Marketplace or template gallery (future consideration)

## Decisions

### D1: Vue Flow as Canvas Engine

**Decision**: Use `@vue-flow/core` + `@vue-flow/background` + `@vue-flow/controls` for the flow editor canvas.

**Rationale**: Vue Flow is a Vue 3 native library with built-in drag-and-drop, pan/zoom, mini-map, and custom node/edge support. It uses a graph model (`nodes[]` + `edges[]`) that maps directly to our data model. Alternative (bpmn-js) is BPMN 2.0 specific, heavier, and React-oriented — overkill for our simplified flow model.

**Alternatives Considered**:
- **bpmn-js**: Full BPMN 2.0 but 800KB+ bundle, React-first, overkill for simplified model
- **React Flow + wrapper**: Adds unnecessary React dependency in Vue app
- **Canvas2D/SVG custom**: Maximum control but 2-3x development time

### D2: 3-Table SQLite Schema

**Decision**: Store flows in 3 tables: `business_flows` (definition + JSON graph), `flow_runs` (execution tracking), `flow_run_artifacts` (per-node output files).

**Rationale**: Storing the graph as a single JSON column in `business_flows` avoids the complexity of normalized node/edge tables while keeping queries simple. The graph is always loaded/saved as a unit. Runs and artifacts are normalized for queryability.

**Alternatives Considered**:
- **Normalized nodes/edges tables**: More queryable but adds 2+ tables, complex joins, and no real benefit since the graph is always loaded whole
- **File-based storage (YAML/JSON files)**: Simpler but loses transactional integrity and queryability

### D3: Rust Conductor Pattern

**Decision**: Implement the Conductor as a Rust async orchestrator using `tokio::spawn` for node execution. Each Agent node invocation creates a tokio task that calls the LLM via the existing AI provider system.

**Rationale**: Matches the user's multi-agent methodology — the Conductor is the independent monitor that:
- Parses flow → builds DAG → topological sort
- Spawns tasks for each node (sequential or parallel at AND gateways)
- Monitors for stall/timeout/deadlock via tokio watch channels
- Executes degradation chain: warn → restart → skip → force convergence → terminate
- Streams events via Tauri `app.emit()` to the frontend

**Alternatives Considered**:
- **Node.js sidecar**: Flexible but adds runtime dependency and IPC overhead
- **Frontend-driven execution**: Limited by browser environment, can't access filesystem directly
- **External workflow engine (n8n, Temporal)**: Overkill, requires separate service

### D4: File-Type Context Binding

**Decision**: Flow-to-context binding is file-type based. Each flow declares which file extensions or glob patterns it applies to. The ContextMenuResolver dynamically adds right-click menu items matching the selected file's type.

**Rationale**: Users think in terms of "right-click this .java file → run code review flow". File-type binding is intuitive, deterministic, and integrates cleanly with the existing FileTree + ContextMenuResolver infrastructure.

**Alternatives Considered**:
- **Scenario dimension binding** (greenfield/maintenance): Too abstract for right-click context
- **Directory-based binding**: Too coarse — a directory contains many file types
- **Hybrid (file-type + scenario)**: More flexible but adds complexity; can be added later

### D5: Prompt Engineering for NL→BPMN

**Decision**: The "Command" button uses prompt engineering — send natural language + available node types + silicon corps member list as a structured prompt to the project's default LLM. The LLM returns a JSON object with `nodes[]` and `edges[]` that the frontend parses into a Vue Flow graph.

**Rationale**: No additional infrastructure needed. Reuses existing AI provider system. The LLM is given a strict JSON schema to follow, and the frontend validates the response before rendering.

**Alternatives Considered**:
- **Fine-tuned model**: Higher quality but requires training infrastructure
- **Template matching**: Deterministic but inflexible; can't handle novel descriptions

### D6: Optimistic Locking for Concurrency

**Decision**: Use `version` integer field in `business_flows` for optimistic locking. On save, increment version and check `WHERE version = expected`. If mismatch, reject and notify user.

**Rationale**: Single-user desktop app makes write contention rare. Optimistic locking is simpler than row-level locks and handles the edge case of two tabs editing the same flow.

### D7: Immutable Artifact Communication

**Decision**: Each Agent node execution writes output to an isolated directory (`{output_dir}/{run_id}/{node_id}/`). Downstream nodes read from these directories but never modify them. The Conductor enforces this via the Manifest contract (input_paths, output_paths, forbidden_paths).

**Rationale**: Directly implements Principle 3 from the multi-agent methodology — "不可变产物传递，零共享内存". File-based communication is auditable, debuggable, and prevents the "spooky action at a distance" bugs common in shared-memory agent systems.

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Vue Flow bundle size** (~150KB) | Increases frontend bundle | Lazy-load the editor panel; Vue Flow only loads when a flow tab is opened |
| **Graph as JSON column** | No SQL-level node/edge queries | Acceptable — flows are always loaded whole; complex queries not needed for MVP |
| **Conductor complexity** | Runtime bugs hard to reproduce | Comprehensive logging; structured event stream; unit tests for each node type |
| **LLM JSON output reliability** | NL→BPMN may produce invalid graphs | Validate JSON schema strictly; fallback to error message + manual creation |
| **Context menu explosion** | Too many flows = too many menu items | Group under "Business Flows" submenu; limit to flows matching the file type |
| **Sub-Flow recursion depth** | Runtime stack overflow | Hard limit of depth 3; cycle detection at save time; Conductor enforces at runtime |
| **12+ node types in MVP** | Increased implementation time | Phase delivery: P0 (6 core), P1 (3 extended), P2 (3 advanced) |
| **Published flow immutability** | User frustration if they want to tweak | Clear UX: "This flow is published. Copy it to make changes." + one-click copy button |

## Migration Plan

1. **Add npm dependency**: `@vue-flow/core`, `@vue-flow/background`, `@vue-flow/controls`, `@vue-flow/minimap`
2. **Run SQL migration**: `m20260604_001_create_business_flow_tables.sql` creates 3 tables
3. **Register Rust module**: Add `mod business_flow;` to `lib.rs`, register commands in `invoke_handler`
4. **Wire EditorPanel**: Add handler for `config.businessFlow` action + `'business-flow'` tab type
5. **No breaking changes**: Existing tables, commands, and components are unaffected
6. **Rollback**: Delete the 3 tables, remove the npm packages, revert EditorPanel changes

## Open Questions

None — all 6 questions from the brainstorm session (OQ-1 through OQ-6) have been resolved.
