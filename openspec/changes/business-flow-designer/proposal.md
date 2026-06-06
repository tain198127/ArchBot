## Why

ArchBot needs a visual business flow designer so users can orchestrate multi-agent (硅基军团) workflows without writing code. Currently, the "Config → Business Flow" menu entry is a dead end — clicking it does nothing because EditorPanel.vue has no handler. The existing FlowExecutor handles simple sequential pipelines, but there is no visual editor, no multi-agent collaboration model, no context-menu integration, and no runtime for executing BPMN-style flows. This feature transforms ArchBot from a tool-driven platform into a workflow-driven one where users design, validate, and execute complex multi-step processes.

## What Changes

- **Business Flow List Panel**: New center-panel tab showing all flows (built-in + custom) with name, type badge, and file-type context bindings
- **Flow Editor Panel**: Vue Flow canvas with 12+ node types, drag-and-drop material inputs, BPMN-style edges with conditions/quality gates
- **Left Toolbar**: Node palette + silicon corps member list, draggable onto canvas
- **Context Menu Integration**: Right-click on FileTree nodes dynamically shows applicable flows; clicking runs the flow with the selected file as material input
- **Run Configuration Panel**: Pre-execution dialog for material selection, output path override
- **Publish/Lock Mechanism**: Published flows are locked (read-only); copy-to-edit workflow
- **Rust Backend**: Full CRUD for flow definitions, 3-table SQLite schema, Conductor runtime with tokio::spawn for async node execution
- **Streaming Output**: Tauri Event-based streaming from Rust Conductor to right-side output panel
- **AI Features**: NL→BPMN generation (Command button) via prompt engineering, AI-powered flow validation
- **7 Action Buttons**: Save, Delete, Cancel, Validate, Copy, Command (NL→BPMN), Run
- **Sub-Flow Support**: Nested flows with max depth 3 and cycle detection at save time
- **YAML Import/Export**: (P2) Flow definitions exportable as YAML for cross-project sharing

## Capabilities

### New Capabilities

- `flow-canvas`: Visual BPMN flow editor using Vue Flow — 12+ node types, edge semantics, drag-and-drop, undo/redo, mini-map
- `flow-persistence`: SQLite-backed CRUD for flow definitions, runs, and artifacts — 3-table schema with optimistic locking
- `flow-conductor`: Rust async runtime that executes flow DAGs — topological sort, parallel gateway branching, quality gates, streaming events, degradation chain
- `flow-context-menu`: Dynamic right-click menu integration on FileTree nodes — file-type-based binding, menu lifecycle management
- `flow-ai-features`: NL→BPMN generation and AI-powered flow validation using project default LLM provider

### Modified Capabilities

- `agent-frontend-panels`: EditorPanel.vue needs handler for `config.businessFlow` action and new tab type `'business-flow'`
- `agent-database`: New migration for 3 business flow tables (business_flows, flow_runs, flow_run_artifacts)

## Impact

- **Frontend**: 6+ new Vue components (ListPanel, EditorPanel, RunPanel, RunConfigDialog, NodePalette, EdgeEditor), new Pinia store (flowStore), 2 composables (useBusinessFlow, useFlowSceneMenus), i18n entries in both locale files
- **Backend**: New Rust module `business_flow/` with 5+ sub-modules (model, commands, conductor, executor, validation), 1 DB migration, 8+ Tauri IPC commands
- **Dependencies**: `@vue-flow/core` + `@vue-flow/background` + `@vue-flow/controls` (new npm packages)
- **Integration Points**: EditorPanel.vue (tab routing), ContextMenuResolver.ts (dynamic menus), useScenario.ts (scenario dimension), FlowExecutor.ts (extends existing orchestration)
- **Migration**: 1 SQL migration file; no breaking changes to existing tables
