## ADDED Requirements

### Requirement: Vue Flow canvas renders flow graphs
The system SHALL render flow definitions as interactive graphs using Vue Flow with pan, zoom, and snap-to-grid support. Each flow SHALL have exactly one Start node and at least one End node.

#### Scenario: Open flow editor
- **WHEN** user clicks a flow in the list panel
- **THEN** a new tab opens in the center panel with a Vue Flow canvas showing the flow's nodes and edges

#### Scenario: Pan and zoom canvas
- **WHEN** user scrolls mouse wheel on canvas
- **THEN** canvas zooms in/out. Drag on empty space pans the view.

### Requirement: 12 node types supported
The system SHALL support 12+ node types: Start, End, Agent, Gateway-XOR, Gateway-AND, Gateway-OR, MaterialInput, QualityGate, SubFlow, Timer, Signal, ErrorHandler, HumanApproval. Each node type SHALL have a distinct visual representation.

#### Scenario: Drag node onto canvas
- **WHEN** user drags a node type icon from the left toolbar onto the canvas
- **THEN** a new node of that type appears at the drop position with default configuration

#### Scenario: Configure agent node
- **WHEN** user double-clicks an Agent node
- **THEN** a configuration modal opens with fields: agentId, skillName, inputPaths, outputPath, forbiddenPaths, timeout, retryPolicy

### Requirement: Edge semantics with conditions and quality gates
The system SHALL allow connecting nodes with edges. Each edge MAY define an action (skill name), a condition expression, a quality gate configuration, and a display label.

#### Scenario: Create edge between nodes
- **WHEN** user drags from a source node's output port to a target node's input port
- **THEN** an edge is created connecting the two nodes

#### Scenario: Configure edge condition
- **WHEN** user clicks an edge and enters a condition expression
- **THEN** the condition is stored on the edge and displayed as a label

### Requirement: Material input via drag-and-drop from FileTree
The system SHALL allow dragging files from the left FileTree panel onto the canvas. Dropped files SHALL appear as MaterialInput nodes with filename, path, and file type icon.

#### Scenario: Drop file onto canvas
- **WHEN** user drags a file from FileTree and drops it on the canvas
- **THEN** a MaterialInput node appears showing the file's name and path

#### Scenario: Material node is read-only reference
- **WHEN** a MaterialInput node is created from a dropped file
- **THEN** the node references the file path without copying the file content

### Requirement: Left toolbar with node palette and agent list
The system SHALL display a left toolbar in the editor panel with two sections: BPMN Node Palette (draggable icons for all node types) and Silicon Corps Member List (all configured digital employees, draggable as Agent nodes).

#### Scenario: Drag agent from toolbar
- **WHEN** user drags a silicon corps member from the toolbar onto the canvas
- **THEN** an Agent node is created pre-configured with that member's agentId

#### Scenario: Toolbar sections are collapsible
- **WHEN** user clicks a section header in the toolbar
- **THEN** that section collapses or expands

### Requirement: Undo and redo support
The system SHALL support undo/redo for canvas operations with a depth of at least 50 operations.

#### Scenario: Undo node creation
- **WHEN** user creates a node and presses Ctrl+Z
- **THEN** the node is removed from the canvas and the action is pushed to the redo stack

### Requirement: Mini-map for large flows
The system SHALL display a mini-map in the corner of the canvas showing the full flow graph with a viewport indicator.

#### Scenario: Navigate via mini-map
- **WHEN** user clicks on the mini-map
- **THEN** the canvas viewport jumps to the clicked position

### Requirement: Dark theme support
All flow canvas components SHALL support dark theme using Tailwind dark: variants. No `<style>` blocks allowed.

#### Scenario: Dark mode rendering
- **WHEN** the application is in dark mode
- **THEN** canvas background, node colors, edge colors, and toolbar all render with dark theme variants
