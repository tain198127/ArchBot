# Business Flow Designer — Requirements Specification

> **Status**: CONFIRMED (Step 1: Brainstorm complete — all 6 open questions resolved)
> **Date**: 2026-06-04
> **Scope**: MVP + Full Node Coverage
> **Methodology Reference**: `/Users/baodan/Nutstore Files/学习/个人重要感悟/multi-agent-design-methodology.md`

---

## 1. User Goals

### Primary Goal
Enable ArchBot users to visually design, validate, and execute multi-agent (硅基军团) business workflows using a simplified BPMN flow editor, with AI-powered generation and validation capabilities.

### User Personas
1. **ArchBot Configurator** — designs business flows for their team's specific scenarios
2. **Flow Operator** — triggers pre-built flows from context menus during daily work
3. **AI Director** — uses natural language to rapidly generate flow prototypes

---

## 2. Functional Requirements

### FR-1: Business Flow List Panel
**Trigger**: Config → Business Flow menu item

| # | Requirement | Priority |
|---|-------------|----------|
| FR-1.1 | Display list of all business flows (built-in + custom) | P0 |
| FR-1.2 | Each row shows: name, type badge (built-in/custom), associated scenarios | P0 |
| FR-1.3 | Associated scenarios: comma-separated labels; if >2, show first + "...N more" with tooltip | P1 |
| FR-1.4 | Click a row → open editor panel as a new tab in center panel | P0 |
| FR-1.5 | Multiple flows = multiple tabs (one tab per flow, tab title = flow name) | P0 |
| FR-1.6 | Built-in flows have lock icon, cannot be deleted | P1 |
| FR-1.7 | Empty state: "No business flows yet. Create one to get started." | P2 |

### FR-2: Flow Editor Panel

#### FR-2.1: Canvas Area (Center)
| # | Requirement | Priority |
|---|-------------|----------|
| FR-2.1.1 | Vue Flow canvas with pan, zoom, snap-to-grid | P0 |
| FR-2.1.2 | Drag-and-drop nodes from left toolbar onto canvas | P0 |
| FR-2.1.3 | Connect nodes with edges by dragging from source port to target port | P0 |
| FR-2.1.4 | Click edge to edit: action (skill name), condition expression, quality gate config | P1 |
| FR-2.1.5 | Double-click node to open configuration modal | P0 |
| FR-2.1.6 | Select node → delete key removes it (with confirmation) | P1 |
| FR-2.1.7 | Undo/Redo support for canvas operations | P2 |
| FR-2.1.8 | Mini-map in corner for large flows | P2 |

#### FR-2.2: Material Input Area
| # | Requirement | Priority |
|---|-------------|----------|
| FR-2.2.1 | Drop zone for files dragged from left FileTree panel | P0 |
| FR-2.2.2 | Dropped files appear as "Material Input" nodes on canvas | P0 |
| FR-2.2.3 | Material nodes are read-only references (no file copy) | P0 |
| FR-2.2.4 | Each material node shows: filename, path, file type icon | P1 |
| FR-2.2.5 | Materials are bound to the flow, available to any Agent node as input | P0 |

#### FR-2.3: Scenario Binding Area
| # | Requirement | Priority |
|---|-------------|----------|
| FR-2.3.1 | Multi-select picker: which left-panel business scenarios this flow applies to | P0 |
| FR-2.3.2 | Each selected scenario creates a right-click context menu entry | P0 |
| FR-2.3.3 | Context menu entry label = flow name | P1 |
| FR-2.3.4 | Clicking context menu entry triggers the flow with the clicked item as material input | P0 |
| FR-2.3.5 | Dynamically adds/removes context menu items when flow bindings change | P0 |

#### FR-2.4: Output Configuration
| # | Requirement | Priority |
|---|-------------|----------|
| FR-2.4.1 | Configure output directory (relative to project root or absolute) | P0 |
| FR-2.4.2 | Configure output filename pattern (supports template variables: `{date}`, `{flow}`, `{node}`) | P1 |
| FR-2.4.3 | Configure output file extension | P0 |
| FR-2.4.4 | One output config per flow (final output of the last node) | P0 |

#### FR-2.5: Left Toolbar (Tools List)
| # | Requirement | Priority |
|---|-------------|----------|
| FR-2.5.1 | BPMN Node Palette: draggable icons for all 12+ node types | P0 |
| FR-2.5.2 | Silicon Corps Member List: all configured digital employees, draggable as Agent nodes | P0 |
| FR-2.5.3 | Collapsible sections: "Flow Controls" / "Agents" / "Recent" | P1 |
| FR-2.5.4 | Search/filter for agents | P2 |

### FR-3: Node Types (12+ Types)

#### Core Nodes (P0)
| Type | Visual | Purpose |
|------|--------|---------|
| Start | Green circle | Flow entry point (exactly 1 per flow) |
| End | Red circle | Flow exit point (1+ per flow) |
| Agent | Person icon + name | A 硅基军团 member executing a skill |
| Gateway (XOR) | Diamond ◇ | Exclusive decision: one path chosen |
| Gateway (AND) | Diamond + | Parallel fork: all paths execute |
| Gateway (OR) | Diamond ○ | Inclusive decision: one or more paths |

#### Extended Nodes (P1)
| Type | Visual | Purpose |
|------|--------|---------|
| Material Input | Document icon | Files dragged in as inputs |
| Quality Gate | Shield icon | Checkpoint: pass/fail/condition |
| Sub-Flow | Nested rectangle | Reference to another flow (recursion allowed, cycle detected) |

#### Advanced Nodes (P2)
| Type | Visual | Purpose |
|------|--------|---------|
| Timer | Clock icon | Delay/wait for specified duration |
| Signal | Lightning icon | Send/receive cross-flow signals |
| Error Handler | Warning icon | Catch and handle errors from upstream |
| Human Approval | User-check icon | Pause for human review/approval |

#### Node Configuration (Agent Node — most complex)
Each Agent node configures:
- **Agent ID**: which 硅基军团 member
- **Skill**: which skill to invoke
- **Input materials**: which Material Input nodes to consume
- **Input paths**: Manifest-style declaration (from methodology)
- **Output path**: where artifacts are written
- **Forbidden paths**: explicit isolation boundary
- **Timeout**: max execution time
- **Retry policy**: on failure, retry N times or skip

### FR-4: Edge Semantics

| Property | Type | Description |
|----------|------|-------------|
| action | string | Skill name to execute on traversal |
| condition | string | Expression for conditional edges (e.g., `quality_score >= 0.8`) |
| qualityGate | object | Gate config: { metric, threshold, onFail: "retry"|"skip"|"abort" } |
| label | string | Display label on the edge |

### FR-5: Action Buttons

#### FR-5.1: Save (P0)
- Validate flow structure (must have Start + End + connected)
- Persist to SQLite via Tauri IPC
- Built-in flows: save as new custom copy (fork)

#### FR-5.2: Delete (P0)
- Custom flows: confirm dialog → delete
- Built-in flows: button disabled with tooltip "Built-in flows cannot be deleted"

#### FR-5.3: Cancel (P0)
- Discard unsaved changes, close tab
- If unsaved changes exist, show confirmation dialog

#### FR-5.4: Validate (P1)
- **Static checks**: cycle detection, disconnected nodes, missing Start/End, orphan edges
- **AI-powered semantic validation**: send flow definition to LLM, ask "Is this flow logically sound? Are there missing steps? Are quality gates positioned correctly?"
- Display validation results as a report panel
- Highlight problematic nodes/edges on canvas

#### FR-5.5: Copy (P1)
- Clone current flow as a new custom flow
- New name = "{Original Name} (Copy)"
- Deep copy all nodes, edges, material references
- Does NOT copy scenario bindings (user must re-bind)

#### FR-5.6: Command (NL→BPMN) (P1)
- Open large modal dialog with text area
- User types natural language description of desired flow
- System prompt includes: available node types, 硅基军团 member list, project context
- LLM generates structured JSON: { nodes: [...], edges: [...] }
- Frontend parses JSON → renders as Vue Flow graph on canvas
- User can then manually refine the generated flow
- **Recommended implementation**: Use `sc:workflow` skill pattern with structured output schema

#### FR-5.7: Run (P0)
- **OQ-2**: Always opens run configuration panel first:
  - Material file selector (pre-populated from flow bindings, user can override)
  - Output directory override (optional)
  - AI provider display (project default, read-only)
  - Confirm/Cancel buttons
- On confirm: send flow definition + material files to Rust backend
- Backend Conductor orchestrates execution:
  1. Parse flow → build DAG
  2. Topological sort → determine execution order
  3. Execute nodes sequentially, parallel where AND gateways fork
  4. Each Agent node: invoke LLM with configured skill + materials
  5. Quality gates: check conditions, branch accordingly
  6. Write output to configured directory
- Right panel shows streaming output (via Tauri Event)
- Run status: running / completed / failed / aborted
- Multiple runs tracked with timestamps

### FR-6: Runtime (Conductor)

Follows multi-agent methodology principles:

| Principle | Implementation |
|-----------|---------------|
| 不可变产物传递 | Each Agent node writes to isolated output dir, other nodes read-only |
| 零共享内存 | No shared state between agents; all communication via file artifacts |
| Manifest 契约 | Each Agent node declares: input_paths, output_paths, forbidden_paths |
| Conductor 监控 | Rust async task monitors: stall detection, timeout, deadlock, output quality |
| 降级模式 | On agent failure: retry → skip → use fallback agent → abort flow |
| 调停决策树 | Conductor follows: warn → restart agent → skip node → force convergence → terminate |

### FR-7: Context Menu Integration

| # | Requirement | Priority |
|---|-------------|----------|
| FR-7.1 | On app load, query all flows with scenario bindings | P0 |
| FR-7.2 | For each binding, add a right-click menu item to the matching left-panel context | P0 |
| FR-7.3 | Menu item appears under a "Business Flows" submenu group | P1 |
| FR-7.4 | Clicking menu item: opens run panel with the context item pre-loaded as material | P0 |
| FR-7.5 | Menu updates dynamically when flows are saved/deleted | P0 |

---

## 3. Non-Functional Requirements

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-1 | Canvas performance with 50+ nodes | 60fps, <100ms drag latency |
| NFR-2 | Flow definition file size | < 500KB for 100-node flow |
| NFR-3 | Validation speed (static) | < 500ms for 100-node flow |
| NFR-4 | Runtime startup latency | < 2s from click to first agent execution |
| NFR-5 | Streaming output latency | < 200ms from backend event to UI render |
| NFR-6 | Tab memory footprint | < 50MB per open flow tab |
| NFR-7 | Undo history depth | 50 operations |
| NFR-8 | Dark theme support | All components must use dark: variants |
| NFR-9 | i18n support | All text via en-US / zh-CN |
| NFR-10 | Accessibility | Keyboard navigable canvas, ARIA labels |

---

## 4. User Stories

### US-1: Create a Simple Review Flow
**As a** Configurator, **I want to** create a flow with 3 agents (code review → security review → documentation), **so that** I can run a complete review pipeline on any file with one click.

**Acceptance Criteria**:
- [ ] Can drag 3 Agent nodes and connect them sequentially
- [ ] Can assign different 硅基军团 members to each node
- [ ] Can bind the flow to the "Java Source" context in left panel
- [ ] Right-clicking a .java file shows the flow as a menu option
- [ ] Clicking runs the flow, streaming output to right panel

### US-2: Generate Flow from Description
**As an** AI Director, **I want to** type "Review this code for security issues, then generate test cases, then write documentation" **so that** the system auto-generates a 3-node flow on the canvas.

**Acceptance Criteria**:
- [ ] "Command" button opens modal with text area
- [ ] Typing description and submitting generates a flow graph
- [ ] Generated graph has correct node types and connections
- [ ] User can manually adjust the generated flow

### US-3: Validate Flow Correctness
**As a** Configurator, **I want to** click "Validate" and see a report of issues **so that** I catch design errors before running.

**Acceptance Criteria**:
- [ ] Detects cycles in the flow graph
- [ ] Detects disconnected nodes
- [ ] Detects missing Start/End nodes
- [ ] AI validates logical coherence of the flow
- [ ] Problem nodes are highlighted on canvas

### US-4: Run Flow with Streaming Output
**As a** Flow Operator, **I want to** click "Run" and see real-time output in the right panel **so that** I can monitor progress and catch errors early.

**Acceptance Criteria**:
- [ ] Clicking Run sends flow + materials to backend
- [ ] Right panel shows streaming agent outputs
- [ ] Status updates: running → completed/failed
- [ ] Can abort a running flow
- [ ] Output files saved to configured directory

---

## 5. Data Model (Conceptual)

### 5.1: Database Tables

```
business_flows
├── id (PK, UUID)
├── name (TEXT, unique)
├── description (TEXT)
├── type (TEXT: "builtin" | "custom")
├── published (BOOLEAN, default false)          -- OQ-6: published flows are locked
├── flow_json (TEXT, serialized nodes + edges)
├── output_dir (TEXT)
├── output_filename_pattern (TEXT)
├── output_extension (TEXT)
├── scenario_bindings (TEXT, JSON array of file-type contexts)  -- OQ-1: file-type based
├── yaml_export (TEXT, nullable)                -- OQ-5: YAML snapshot for import/export (P2)
├── created_at (TIMESTAMP)
├── updated_at (TIMESTAMP)
├── published_at (TIMESTAMP, nullable)
└── version (INTEGER, optimistic locking)

flow_runs
├── id (PK, UUID)
├── flow_id (FK → business_flows.id)
├── status (TEXT: "pending" | "running" | "completed" | "failed" | "aborted")
├── triggered_by (TEXT: "menu" | "manual" | "api")
├── material_paths (TEXT, JSON array)
├── started_at (TIMESTAMP)
├── completed_at (TIMESTAMP, nullable)
├── output_log (TEXT, streaming log)
└── error_message (TEXT, nullable)

flow_run_artifacts
├── id (PK, UUID)
├── run_id (FK → flow_runs.id)
├── node_id (TEXT)
├── agent_id (TEXT)
├── artifact_path (TEXT)
├── artifact_type (TEXT)
├── created_at (TIMESTAMP)
└── checksum (TEXT)
```

### 5.2: TypeScript Types (Conceptual)

```typescript
interface FlowDefinition {
  id: string
  name: string
  description: string
  type: 'builtin' | 'custom'
  nodes: FlowNode[]
  edges: FlowEdge[]
  materials: MaterialRef[]
  scenarioBindings: ScenarioBinding[]
  outputConfig: OutputConfig
  version: number
}

type NodeType = 'start' | 'end' | 'agent' | 'gateway_xor' | 'gateway_and' |
  'gateway_or' | 'material_input' | 'quality_gate' | 'sub_flow' |
  'timer' | 'signal' | 'error_handler' | 'human_approval'

interface FlowNode {
  id: string
  type: NodeType
  position: { x: number; y: number }
  data: NodeData  // varies by type
}

interface AgentNodeData {
  agentId: string
  skillName: string
  inputPaths: string[]
  outputPath: string
  forbiddenPaths: string[]
  timeout: number
  retryPolicy: { maxRetries: number; onFail: 'retry' | 'skip' | 'abort' }
  personality?: string  // role description
}

interface FlowEdge {
  id: string
  source: string
  target: string
  action?: string
  condition?: string
  qualityGate?: QualityGateConfig
  label?: string
}
```

---

## 6. Resolved Questions

> All open questions resolved during Step 1 brainstorm session (2026-06-04).

| # | Question | Decision | Rationale |
|---|----------|----------|-----------|
| OQ-1 | Business scenario binding model | **File-type context** — right-click menu on FileTree nodes | Matches user mental model: "right-click this .java file → run review flow" |
| OQ-2 | Run button behavior | **Run configuration panel** — always show config before execution | Allows material override, output path customization, provider selection |
| OQ-3 | Sub-Flow recursion | **Max depth 3 + cycle detection at save time** | Prevents infinite recursion while allowing reasonable nesting |
| OQ-4 | AI provider for validation/execution | **Project default provider** | Simplicity; no per-flow provider configuration needed |
| OQ-5 | Flow import/export | **YAML format (P2 priority)** | Human-readable, git-friendly, data model reserves field |
| OQ-6 | Publish/lock mechanism | **Full MVP implementation** — published flows locked, copy-to-edit | User explicitly wants this in MVP, not just data model placeholder |

---

## 7. Recommended 5-Step Implementation Pipeline

### Step 1: Office Hours ✅ (This Session)
- [x] Clarify requirements
- [x] Confirm BPMN scope: Simplified Flow
- [x] Confirm runtime: Tauri Rust backend
- [x] Confirm node types: 12+ full set
- [x] Confirm NL→BPMN: Prompt Engineering
- [x] Produce this requirements document

### Step 2: OpenSpec Propose (Next Session)
- [ ] Write formal spec file at `.specs/business-flow-designer.md`
- [ ] Define boundaries: what's in/out of scope
- [ ] Define API contracts (Tauri IPC commands)
- [ ] Define component hierarchy
- [ ] Get user sign-off on spec

### Step 3: Engineering Review (After Step 2)
- [ ] Architecture review: data model, runtime, integration points
- [ ] Security review: file access, agent isolation, path traversal prevention
- [ ] Performance review: canvas optimization, streaming efficiency
- [ ] Identify design issues before coding

### Step 4: Subagent Implementation (After Step 3)
Decompose into parallel workstreams:

**Team Structure** (5 subagents + 1 coordinator):
| Agent | Responsibility | Depends On |
|-------|---------------|------------|
| Types Architect | TypeScript types + Rust structs + DB migration | None |
| Frontend List | BusinessFlowListPanel + tab management | Types |
| Frontend Editor | BusinessFlowEditorPanel + Vue Flow canvas + toolbar | Types |
| Backend CRUD | Rust business_flow module + Tauri commands | Types |
| Backend Runtime | Conductor + flow executor + streaming | Backend CRUD |
| **Coordinator** | Resolve conflicts, enforce spec, prevent deadlocks | All |

**Implementation Phases**:
- P0: Types → DB Migration → Backend CRUD → List Panel → Basic Editor (no runtime)
- P1: Full Editor (12 node types) → Context Menu Integration → Validate Button
- P2: Conductor Runtime → Run Button → Streaming Output
- P3: NL→BPMN (Command Button) → AI Validation → Sub-Flow → Advanced Nodes

### Step 5: OpenSpec Archive
- [ ] Update spec to match actual implementation
- [ ] Archive in `.specs/archive/`
- [ ] Update CLAUDE.md with new skill entries if needed
- [ ] Update prd.yml with new feature entries

---

## 8. Skills Recommendations

| Feature | Recommended Skill | Rationale |
|---------|------------------|-----------|
| NL→BPMN (Button 6) | `sc:workflow` | Already handles structured multi-step generation; adapt for JSON output |
| AI Validation (Button 4) | `sc:analyze` | Analyzes flow structure for logical coherence |
| Flow Execution (Button 7) | `sc:workflow` with streaming | Extended to support node-by-node execution with Tauri Event streaming |
| Flow Design | `sc:design` | Architecture design for the feature itself |

---

*This requirements document is the output of Step 1 (Brainstorm). Proceed to Step 2 (OpenSpec propose) to formalize the specification.*
