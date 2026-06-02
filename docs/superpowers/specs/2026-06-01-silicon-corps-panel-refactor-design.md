# Silicon Corps Panel Refactoring — Design Spec

**Date**: 2026-06-01
**Status**: Draft — awaiting user review

---

## 1. Overview

Refactor the 硅基军团 (Silicon Corps) panel to streamline editing, remove unused
sections, and replace the hardcoded "默认操作" (Default Operation) dropdown with
a dynamic "默认能力" (Default Capability) dropdown populated from installed skill
packages and custom skills.

### Goals

1. **Click = Edit**: Single-click any list row → enter edit mode directly (no view mode)
2. **Remove dead sections**: Handoff rules, Agent picker, MCP picker
3. **Dynamic capabilities**: Replace hardcoded operation list with skills from
   installed agent runtime packages + custom DB skills
4. **Grouped display**: Capabilities grouped by source package; custom skills as "游击队"
5. **Full i18n**: All new UI text internationalized (zh-CN + en-US)

---

## 2. Architecture

### 2.1 Data Flow

```
┌─────────────────────┐     ┌──────────────────────┐
│  Rust Backend        │     │  Vue Frontend         │
│                      │     │                       │
│  agent_list_skill_   │────▶│  loadCapabilities()   │
│  commands(runtime)   │     │                       │
│                      │     │  ┌─────────────────┐  │
│  skills/ dir walk    │     │  │ Grouped options  │  │
│  → SkillCommand[]    │     │  │ for VSelect      │  │
│                      │     │  └─────────────────┘  │
│  db_find_all         │────▶│                       │
│  (table: skills)     │     │  + i18n name mapping  │
│                      │     │                       │
│  de_save(employee)   │◀────│  form.default_        │
│                      │     │  capability ← select  │
└─────────────────────┘     └──────────────────────┘
```

### 2.2 Component Tree

```
DigitalEmployeePanel.vue
├── List View (editMode === 'list')
│   ├── Search bar (VInput)
│   ├── Action buttons: [新增] [编辑]  ← 删除 [查看]
│   └── Employee table (click row → edit)
│
└── Edit View (editMode === 'edit')  ← 删除 view 分支
    ├── Basic Info fieldset
    ├── Personality fieldset
    ├── Capabilities fieldset
    │   ├── Focus Areas (checkboxes)
    │   ├── Deliverable Groups (checkboxes)
    │   └── Default Capability (grouped VSelect)  ← renamed from defaultOp
    ├── Skill Stack fieldset
    │   └── Skills (multi-select)  ← only Skills, no Agent/MCP
    └── Footer actions: [保存] [复制] [删除] [取消]

Removed:
  ✗ Handoff Rules fieldset (entire block)
  ✗ Agent row in Skill Stack
  ✗ MCP row in Skill Stack
  ✗ View mode (all :disabled="editMode === 'view'" guards)
```

---

## 3. Data Model

### 3.1 Database Change

```sql
-- Add new column, keep old for migration
ALTER TABLE digital_employees ADD COLUMN default_capability TEXT NOT NULL DEFAULT '';
-- Migrate existing data: default_capability = default_op (can be left as-is or mapped)
```

### 3.2 Rust Types

```rust
/// A single skill command discovered from an installed skill package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCommand {
    /// Parent skill package name (e.g., "superpowers", "gstack")
    pub package: String,
    /// Skill directory name (e.g., "brainstorming", "qa")
    pub skill_name: String,
    /// The slash command (e.g., "/brainstorming", "/qa")
    pub command: String,
    /// English display name from SKILL.md or fallback
    pub display_name_en: String,
}
```

### 3.3 New Tauri Command

```rust
/// Enumerate all skill commands from installed skill packages.
/// Walks {skills_dir}/{package}/skills/*/SKILL.md to discover commands.
#[tauri::command]
pub fn agent_list_skill_commands(runtime: String) -> Result<Vec<SkillCommand>, String>
```

**Discovery logic**:
1. Resolve `{isolated_home}/.claude/skills/` for the given runtime
2. List all subdirectories (each is a skill package)
3. For each package, list `{package}/skills/` subdirectories
4. For each skill dir, read `SKILL.md` frontmatter to extract `name` and command
5. Return flat `Vec<SkillCommand>` with package grouping info

### 3.4 Frontend Types

```typescript
interface CapabilityOption {
  value: string        // "superpowers/brainstorming" or "guerrillas/custom-xyz"
  label: string        // "头脑风暴 (/brainstorming)" — i18n mapped
  group: string        // "Super Power" or "游击队"
  command: string      // "/brainstorming"
}
```

---

## 4. i18n Design

### 4.1 Name Mapping Table (Built-in)

Two TypeScript constants provide the mapping layer:

```typescript
// Package display names — used as group headers in the dropdown
const PACKAGE_DISPLAY_NAMES: Record<string, string> = {
  'superpowers':            'Super Power',
  'gstack':                 'Gstack',
  'everything-claude-code': 'Everything Claude Code',
  'super-claude':           'Super Claude',
  'claude-official-skills': 'OpenSpec',
}
```

A `SKILL_NAME_MAP` maps `package/skill_name` → i18n key:

```typescript
// In DigitalEmployeePanel.vue
const SKILL_NAME_MAP: Record<string, string> = {
  // ── superpowers ──
  'superpowers/brainstorming':              'de.cap.brainstorming',
  'superpowers/writing-plans':              'de.cap.writingPlans',
  'superpowers/executing-plans':            'de.cap.executingPlans',
  'superpowers/test-driven-development':    'de.cap.testDrivenDevelopment',
  'superpowers/systematic-debugging':       'de.cap.systematicDebugging',
  'superpowers/subagent-driven-development':'de.cap.subagentDrivenDev',
  'superpowers/verification-before-completion':'de.cap.verificationCheck',
  'superpowers/requesting-code-review':     'de.cap.requestingCodeReview',
  'superpowers/dispatching-parallel-agents':'de.cap.dispatchingAgents',
  'superpowers/using-git-worktrees':       'de.cap.gitWorktrees',
  'superpowers/graphify':                  'de.cap.graphify',
  // ── gstack ──
  'gstack/browse':              'de.cap.browse',
  'gstack/qa':                  'de.cap.qa',
  'gstack/ship':                'de.cap.ship',
  'gstack/plan-eng-review':     'de.cap.planEngReview',
  'gstack/cso':                 'de.cap.cso',
  'gstack/document-generate':   'de.cap.documentGenerate',
  'gstack/office-hours':        'de.cap.officeHours',
  'gstack/review':              'de.cap.gstackReview',
  'gstack/land-and-deploy':     'de.cap.landAndDeploy',
  'gstack/design-consultation': 'de.cap.designConsultation',
  'gstack/investigate':         'de.cap.investigate',
  'gstack/retro':               'de.cap.retro',
  // ── everything-claude-code ──
  'everything-claude-code/code-review':      'de.cap.codeReview',
  'everything-claude-code/arch-review':      'de.cap.archReview',
  'everything-claude-code/security-review':  'de.cap.securityReview',
  'everything-claude-code/sc:design':        'de.cap.scDesign',
  'everything-claude-code/sc:implement':     'de.cap.scImplement',
  'everything-claude-code/sc:analyze':       'de.cap.scAnalyze',
  'everything-claude-code/e2e':              'de.cap.e2e',
  'everything-claude-code/deep-research':    'de.cap.deepResearch',
  'everything-claude-code/orchestrate':      'de.cap.orchestrate',
  'everything-claude-code/tdd':              'de.cap.tdd',
  'everything-claude-code/docs':             'de.cap.docs',
  'everything-claude-code/plan':             'de.cap.plan',
  'everything-claude-code/loop':             'de.cap.loop',
  // ── super-claude ──
  'super-claude/*': 'de.cap.superClaudeFallback',  // TBD after inspecting repo
  // ── claude-official-skills (OpenSpec etc) ──
  'claude-official-skills/openspec-propose':       'de.cap.openspecPropose',
  'claude-official-skills/openspec-apply-change':  'de.cap.openspecApply',
  'claude-official-skills/openspec-explore':       'de.cap.openspecExplore',
  'claude-official-skills/openspec-new-change':    'de.cap.openspecNewChange',
  'claude-official-skills/openspec-archive-change':'de.cap.openspecArchive',
  'claude-official-skills/openspec-verify-change': 'de.cap.openspecVerify',
}
```

### 4.2 i18n Keys (zh-CN.ts)

```typescript
digitalEmployee: {
  // ... existing keys ...
  // Changed:
  defaultCapability: '默认能力',  // was defaultOp: '默认操作'
  // Removed:
  // view, viewEmployee, handoffRules, addHandoff
  // Added:
  guerrillas: '游击队',
  cap: {
    // superpowers
    brainstorming: '头脑风暴',
    writingPlans: '编写计划',
    executingPlans: '执行计划',
    testDrivenDevelopment: '测试驱动开发',
    systematicDebugging: '系统调试',
    subagentDrivenDev: '子代理驱动开发',
    verificationCheck: '完成前验证',
    requestingCodeReview: '请求代码评审',
    dispatchingAgents: '并行代理调度',
    gitWorktrees: 'Git工作树',
    graphify: '知识图谱',
    // gstack
    browse: '网页浏览',
    qa: 'QA测试',
    ship: '部署发布',
    planEngReview: '工程评审',
    cso: '安全审计',
    documentGenerate: '文档生成',
    officeHours: '办公时间',
    gstackReview: '代码评审',
    landAndDeploy: '部署上线',
    designConsultation: '设计咨询',
    investigate: '问题调查',
    retro: '回顾总结',
    // everything-claude-code
    codeReview: '代码评审',
    archReview: '架构评审',
    securityReview: '安全审查',
    scDesign: '系统设计',
    scImplement: '代码实现',
    scAnalyze: '代码分析',
    e2e: '端到端测试',
    deepResearch: '深度研究',
    orchestrate: '多智能体编排',
    tdd: '测试驱动开发',
    docs: '文档生成',
    plan: '制定计划',
    loop: '自主循环',
    // openspec
    openspecPropose: '创建变更提案',
    openspecApply: '实施变更',
    openspecExplore: '需求探索',
    openspecNewChange: '新建变更',
    openspecArchive: '归档变更',
    openspecVerify: '验证变更',
  },
}
```

### 4.3 i18n Keys (en-US.ts)

```typescript
digitalEmployee: {
  // ... existing keys ...
  defaultCapability: 'Default Capability',
  guerrillas: 'Guerrillas',
  cap: {
    brainstorming: 'Brainstorming',
    writingPlans: 'Writing Plans',
    // ... (English equivalents)
  },
}
```

### 4.4 Fallback Strategy

If a skill has no mapping in `SKILL_NAME_MAP`:
1. Try to use `display_name_en` from `SkillCommand` (read from SKILL.md)
2. Fallback: show `skill_name` as-is
3. Always show `(/command)` suffix

---

## 5. Component Changes (DigitalEmployeePanel.vue)

### 5.1 State Changes

```diff
- const editMode = ref<'list' | 'edit' | 'view'>('list')
+ const editMode = ref<'list' | 'edit'>('list')

- const editAgents = ref<string[]>([])
- const editMcps = ref<string[]>([])
- const agentPick = ref('')
- const mcpPick = ref('')
- const editHandoffs = ref<any[]>([])

- function onPickAgent() {...}
- function onPickMcp() {...}

+ // Capability options loaded from backend
+ const capabilityOptions = ref<CapabilityOption[]>([])
+ const capabilityLoading = ref(false)

- function handleView() {...}
```

### 5.2 Method Changes

```diff
  function handleRowClick(row: any) {
    selectedIds.value = new Set([row.id])
-   isNew.value = false; editMode.value = 'view'
+   isNew.value = false; editMode.value = 'edit'
    editForm.value = { ...row }
    editSkills.value = parseJsonArray(row.skills)
-   editAgents.value = parseJsonArray(row.agents)
-   editMcps.value = parseJsonArray(row.mcps)
-   editHandoffs.value = ...
+   editForm.value.default_capability = row.default_capability || ''
  }

+ async function loadCapabilities() {
+   capabilityLoading.value = true
+   try {
+     // Load installed skill commands from runtime
+     const commands = await invoke<SkillCommand[]>('agent_list_skill_commands', { runtime: 'claude_code' })
+     // Load custom skills from DB
+     const customSkills = await invoke<any>('db_find_all', { table: 'skills', ... })
+     capabilityOptions.value = buildGroupedOptions(commands, customSkills)
+   } finally { capabilityLoading.value = false }
+ }
+
+ function buildGroupedOptions(commands: SkillCommand[], customSkills: any[]): CapabilityOption[] {
+   const options: CapabilityOption[] = []
+   // Group by package
+   const grouped = new Map<string, SkillCommand[]>()
+   for (const cmd of commands) {
+     if (!grouped.has(cmd.package)) grouped.set(cmd.package, [])
+     grouped.get(cmd.package)!.push(cmd)
+   }
+   for (const [pkg, cmds] of grouped) {
+     const pkgName = PACKAGE_DISPLAY_NAMES[pkg] || pkg
+     for (const cmd of cmds) {
+       const mapKey = `${cmd.package}/${cmd.skill_name}`
+       const i18nKey = SKILL_NAME_MAP[mapKey]
+       const displayName = i18nKey ? tt(i18nKey) : (cmd.display_name_en || cmd.skill_name)
+       options.push({
+         value: mapKey,
+         label: `${displayName} (${cmd.command})`,
+         group: pkgName,
+         command: cmd.command,
+       })
+     }
+   }
+   // Custom skills → 游击队
+   for (const cs of customSkills) {
+     options.push({
+       value: `guerrillas/${cs.code || cs.id}`,
+       label: `${cs.name || cs.label} (/${cs.code || cs.id})`,
+       group: tt('digitalEmployee.guerrillas'),
+       command: `/${cs.code || cs.id}`,
+     })
+   }
+   return options
+ }
```

### 5.3 Template Changes

```diff
- <VButton size="sm" variant="ghost" @click="handleView">{{ de.view || '查看' }}</VButton>
+ <!-- removed -->

- <!-- Edit / View -->
+ <!-- Edit -->

- <span>{{ isNew ? ... : (editMode === 'view' ? (de.viewEmployee) : (de.editEmployee)) }}</span>
+ <span>{{ isNew ? de.newEmployee : de.editEmployee }}</span>

  <!-- All :disabled="editMode === 'view'" removed -->
- :disabled="editMode === 'view'"

  <!-- Handoff Rules fieldset → removed entirely -->

  <!-- Agent row → removed -->
  <!-- MCP row → removed -->

  <!-- Default Op → Default Capability -->
- <VSelect v-model="editForm.default_op" :options="opOptions" ... />
+ <VSelect v-model="editForm.default_capability" :options="capabilityOptions"
+   option-group-label="group" option-label="label" option-value="value"
+   :loading="capabilityLoading" ... />
```

---

## 6. Backend Changes

### 6.1 New Module: `skill_discovery.rs`

```
src-tauri/src/agent_runtime/
├── skill_installer.rs      (existing)
├── skill_discovery.rs      (NEW)
└── mod.rs                  (register new module)
```

### 6.2 `skill_discovery.rs` Responsibilities

1. **Walk skill directories**: `{skills_dir}/{package}/skills/{skill_name}/SKILL.md`
2. **Parse SKILL.md**: Extract frontmatter (YAML between `---` markers) for `name` and command trigger
3. **Return `Vec<SkillCommand>`**: Flat list with package grouping info

### 6.3 Tauri Command Registration

In `lib.rs`:
```rust
pub fn agent_list_skill_commands(runtime: String) -> Result<Vec<SkillCommand>, String>
```

### 6.4 Digital Employee Entity Update

In `digital_employee.rs` entity:
```diff
- pub default_op: String,
+ pub default_capability: String,
```

---

## 7. Files Changed Summary

| File | Change Type | Description |
|------|------------|-------------|
| `src/components/domain/DigitalEmployeePanel.vue` | Major | Remove view mode, handoffs, agent/mcp; rename defaultOp→defaultCapability with grouped dynamic options |
| `src/i18n/zh-CN.ts` | Medium | Add ~40 capability name keys; rename defaultOp→defaultCapability; remove view/handoff keys |
| `src/i18n/en-US.ts` | Medium | Mirror zh-CN changes in English |
| `src-tauri/src/agent_runtime/skill_discovery.rs` | **New** | Walk installed skill packages, enumerate commands |
| `src-tauri/src/agent_runtime/mod.rs` | Minor | Register skill_discovery module |
| `src-tauri/src/lib.rs` | Minor | Register `agent_list_skill_commands` Tauri command |
| `src-tauri/src/db/entities/digital_employee.rs` | Minor | `default_op` → `default_capability` |
| `src-tauri/src/digital_employee.rs` | Minor | Update field references |
| `src-tauri/src/handlers/de_handler.rs` | Minor | Update field references |
| `src/api/types.ts` | Minor | Update `DigitalEmployee` interface |
| `prd.yml` | Minor | Reflect feature changes |
| `function-map.yml` | Minor | Reflect capability changes |

---

## 8. Migration Plan

1. Add `default_capability` column (nullable, default `''`)
2. Deploy backend + frontend changes
3. For existing employees, `default_capability` starts empty — user can set it on next edit
4. Old `default_op` column kept for backward compatibility, removed in future cleanup

---

## 9. Assumptions & Risks

| # | Assumption | Risk | Mitigation |
|---|-----------|------|------------|
| A1 | Skill packages follow `{package}/skills/{skill_name}/SKILL.md` directory convention | Medium — some packages may differ | Verify against actual superpowers/gstack/ecc repos during implementation; add fallback to walk for any `.md` |
| A2 | VSelect component can render grouped options (`option-group-label` prop) | Low — may need to extend base VSelect | Check VSelect.vue current API; if missing, add grouped mode support |
| A3 | Skills are already installed via Agent Config before capabilities are loaded | Low — dropdown may be empty | Show "No capabilities loaded — install skills first" message when empty |
| A4 | SuperClaude Framework repo structure matches other skill packages | Medium — repo unexamined | SuperClaude mappings marked TBD; will inspect repo during implementation |
| A5 | `skills` DB table stores custom user-defined skills | Low — table exists but may need schema changes | Verify schema; add `command` field if missing |

---

## 10. Testing Checklist

- [ ] Click any list row → enters edit mode directly
- [ ] No "查看" button visible
- [ ] No handoff rules section in edit form
- [ ] No Agent/MCP pickers in skill stack
- [ ] Default capability dropdown shows grouped options
- [ ] Options load from installed skill packages
- [ ] Custom skills appear under "游击队" group
- [ ] Each option displays as "中文名 (/command)"
- [ ] Selecting an option saves correctly
- [ ] i18n switches correctly between zh-CN and en-US
- [ ] Backward compat: existing employees with empty default_capability work
