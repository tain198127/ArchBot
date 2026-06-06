## ADDED Requirements

### Requirement: NL→BPMN generation via prompt engineering
The system SHALL provide a "Command" button that opens a modal dialog with a text area. When the user submits natural language text, the system SHALL send a structured prompt to the project's default LLM provider that includes: available node types, silicon corps member list, and project context. The LLM SHALL return a JSON object with `nodes` and `edges` arrays that the frontend parses and renders as a Vue Flow graph.

#### Scenario: Generate flow from description
- **WHEN** user types "Review code for security issues, then generate test cases, then write documentation" and submits
- **THEN** the LLM returns a JSON flow with 3 Agent nodes connected sequentially, and the graph is rendered on the canvas

#### Scenario: Invalid LLM output handled gracefully
- **WHEN** the LLM returns malformed JSON or nodes with invalid types
- **THEN** the system displays an error message "Could not generate flow. Please try again or create manually." and does not modify the existing canvas

#### Scenario: Generated flow is editable
- **WHEN** a flow is generated from NL→BPMN
- **THEN** the user can manually add, remove, move, and reconfigure any node or edge

### Requirement: AI-powered flow validation
The system SHALL provide a "Validate" button that performs two levels of validation: (1) static checks — cycle detection, disconnected nodes, missing Start/End, orphan edges; (2) AI-powered semantic validation — send the flow definition to the LLM and ask whether the flow is logically sound, whether quality gates are positioned correctly, and whether there are missing steps.

#### Scenario: Static validation detects cycle
- **WHEN** user clicks Validate on a flow with a cycle (A→B→C→A)
- **THEN** a validation report shows "Cycle detected: A → B → C → A" and the involved nodes are highlighted on the canvas

#### Scenario: Static validation detects missing Start node
- **WHEN** user clicks Validate on a flow without a Start node
- **THEN** a validation report shows "Missing Start node. Every flow must have exactly one Start node."

#### Scenario: AI semantic validation
- **WHEN** user clicks Validate on a structurally valid flow
- **THEN** the LLM analyzes the flow and reports issues like "Quality gate after Agent 'reviewer' checks review_quality but the Agent does not output this metric"

#### Scenario: Validation highlights problematic nodes
- **WHEN** validation finds issues with specific nodes or edges
- **THEN** those nodes/edges are highlighted with a red/orange border on the canvas
