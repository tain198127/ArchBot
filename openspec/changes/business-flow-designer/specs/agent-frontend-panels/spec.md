## MODIFIED Requirements

### Requirement: EditorPanel handles config.businessFlow action
EditorPanel.vue SHALL handle the `config.businessFlow` action by opening a new tab of type `'business-flow-list'` in the center panel. This tab renders the `BusinessFlowListPanel` component.

#### Scenario: Click Config → Business Flow menu
- **WHEN** user clicks Config → Business Flow in the menu bar
- **THEN** a new tab opens in EditorPanel with type 'business-flow-list' showing the flow list

### Requirement: EditorPanel supports business-flow tab types
EditorPanel.vue SHALL support rendering two new tab types: `'business-flow-list'` (renders BusinessFlowListPanel) and `'business-flow-editor'` (renders BusinessFlowEditorPanel with flow ID as prop). Multiple `'business-flow-editor'` tabs MAY be open simultaneously, one per flow.

#### Scenario: Open flow editor tab
- **WHEN** user clicks a flow in BusinessFlowListPanel
- **THEN** a new tab of type 'business-flow-editor' opens with the flow's ID, tab title shows the flow name

#### Scenario: Multiple flow editor tabs
- **WHEN** user opens flow A and then flow B
- **THEN** two tabs are visible: "Flow A" and "Flow B", each rendering independently

#### Scenario: Close flow editor tab
- **WHEN** user closes a flow editor tab with unsaved changes
- **THEN** a confirmation dialog appears: "You have unsaved changes. Close anyway?"

## ADDED Requirements

### Requirement: BusinessFlowListPanel component
The system SHALL provide a BusinessFlowListPanel component that displays all business flows in a table with columns: name, type badge (built-in/custom with appropriate styling), associated file-type bindings, and status indicator (published/draft).

#### Scenario: Display flow list
- **WHEN** BusinessFlowListPanel loads
- **THEN** all flows are listed with name, type badge, and file-type bindings

#### Scenario: Bindings with overflow
- **WHEN** a flow has more than 2 file-type bindings
- **THEN** the first 2 bindings are shown followed by "...N more" with a tooltip listing all bindings

### Requirement: BusinessFlowEditorPanel component
The system SHALL provide a BusinessFlowEditorPanel component that contains: Vue Flow canvas (center), left toolbar (node palette + agent list), action button bar (Save, Delete, Cancel, Validate, Copy, Command, Run), and scenario binding configuration area.

#### Scenario: Editor loads flow
- **WHEN** BusinessFlowEditorPanel receives a flow ID
- **THEN** it loads the flow definition from the backend and renders nodes/edges on the canvas

### Requirement: BusinessFlowRunPanel component
The system SHALL provide a BusinessFlowRunPanel component in the right panel that displays streaming execution output. The panel SHALL show: run status badge, per-node output sections (collapsible), and a progress indicator.

#### Scenario: Run panel shows streaming output
- **WHEN** a flow is executing
- **THEN** the right panel displays each node's output in real-time as it arrives

#### Scenario: Run panel shows completion
- **WHEN** a flow completes
- **THEN** the run panel shows "Completed" status, output file paths, and a summary
