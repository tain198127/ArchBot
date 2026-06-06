## ADDED Requirements

### Requirement: File-type-based context menu binding
Each flow SHALL declare `scenario_bindings` as a JSON array of file-type patterns (e.g., `[".java"]`, `["*.yml", "*.yaml"]`). The ContextMenuResolver SHALL dynamically add right-click menu items for flows whose bindings match the selected file's type.

#### Scenario: Flow bound to .java files
- **WHEN** a flow declares `scenario_bindings: [".java"]` and user right-clicks a `.java` file in FileTree
- **THEN** a menu item with the flow's name appears under a "Business Flows" submenu

#### Scenario: No matching flows
- **WHEN** user right-clicks a `.png` file and no flow is bound to `.png`
- **THEN** no "Business Flows" submenu appears

### Requirement: Dynamic menu lifecycle
Context menu items SHALL be dynamically updated when flows are saved, deleted, or have their bindings modified. The menu SHALL update without requiring application restart.

#### Scenario: New flow adds menu item
- **WHEN** user saves a new flow with binding `[".java"]`
- **THEN** right-clicking a `.java` file immediately shows the new flow's menu item

#### Scenario: Deleted flow removes menu item
- **WHEN** user deletes a flow
- **THEN** the corresponding context menu item is immediately removed

### Requirement: Flow execution from context menu
Clicking a flow's context menu item SHALL open the run configuration panel with the right-clicked file pre-loaded as a material input. The flow's output directory and settings are pre-populated from the flow definition.

#### Scenario: Execute flow from context menu
- **WHEN** user right-clicks `UserService.java`, selects "Code Review Flow" from Business Flows submenu, and clicks Confirm in the run config panel
- **THEN** the flow executes with `UserService.java` as material input

### Requirement: Business Flows submenu grouping
All flow-triggered context menu items SHALL be grouped under a "Business Flows" parent menu item. If only one flow matches, the submenu SHALL still be used for consistency.

#### Scenario: Multiple matching flows
- **WHEN** 3 flows are bound to `.java` files and user right-clicks a `.java` file
- **THEN** a "Business Flows" submenu appears containing 3 items, one per flow
