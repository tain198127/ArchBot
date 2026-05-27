---
name: i18n-sync
description: |
  Enforces i18n synchronization for all UI text changes in the ArchBot project.
  Automatically triggered when any .vue component or UI-related .ts file is modified.
  Ensures zh-CN and en-US resource files stay in sync with page content.
globs:
  - "src/**/*.vue"
  - "src/config/**/*.ts"
  - "src/i18n/**/*.ts"
---

# i18n Sync Skill

## Rule: No Hardcoded UI Text

**Every user-visible string in .vue templates and UI config files MUST use the i18n system.**

This is non-negotiable. Never write a raw Chinese or English string in a template or component.

## Project i18n Structure

```
src/i18n/
├── index.ts       # useI18n() composable, locale switching
├── zh-CN.ts       # Chinese resource (source of truth for keys)
└── en-US.ts       # English resource (must mirror zh-CN keys exactly)
```

- Type safety: `en-US.ts` must have the **exact same key structure** as `zh-CN.ts`
- The type `Messages` is derived from `typeof zhCN`, so `zh-CN.ts` defines the shape
- Usage in components: `const { t } = useI18n()` then `t.value.section.key`

## Mandatory Workflow

When modifying ANY `.vue` file or UI config file:

### Step 1: Identify all user-visible text
Scan for:
- Template text content (between tags)
- Placeholder attributes
- Title/tooltip attributes
- Label props on Element Plus components
- Error messages shown to users
- Confirmation dialog text
- Notification/message text

### Step 2: Choose or create i18n key
- Use existing keys from `zh-CN.ts` when meaning matches
- Create new keys following the naming convention:
  - Group by feature area: `menu`, `menuFile`, `menuEdit`, `menuConfig`, `menuRun`, `panel`, `editor`, `model`, `bottom`
  - For new features, create a new top-level group (e.g., `dialog`, `notification`, `settings`)
  - Use camelCase for key names
  - Key name should describe the content's purpose, not its value

### Step 3: Update BOTH resource files
- Add the key to `src/i18n/zh-CN.ts` with the Chinese text
- Add the **same key** to `src/i18n/en-US.ts` with the English translation
- Both files must always have identical key structures

### Step 4: Use in component
```vue
<script setup lang="ts">
import { useI18n } from '../i18n'  // adjust path as needed
const { t } = useI18n()
</script>

<template>
  <!-- Direct usage -->
  <span>{{ t.section.key }}</span>
  
  <!-- Attribute binding -->
  <input :placeholder="t.section.inputPlaceholder" />
  
  <!-- Element Plus props -->
  <el-option :label="t.section.optionLabel" />
</template>
```

### Step 5: Verify key sync
After all changes, verify:
- Every key in `zh-CN.ts` exists in `en-US.ts`
- Every key in `en-US.ts` exists in `zh-CN.ts`
- No orphaned keys (keys that exist in resource files but are never referenced)
- Run `npx vue-tsc --noEmit` — TypeScript will catch mismatched structures

## Key Naming Convention

| Area | Prefix | Example |
|------|--------|---------|
| Top-level menu labels | `menu.*` | `menu.file`, `menu.edit` |
| File menu items | `menuFile.*` | `menuFile.newProject` |
| Edit menu items | `menuEdit.*` | `menuEdit.find` |
| Config menu items | `menuConfig.*` | `menuConfig.ai` |
| Run menu items | `menuRun.*` | `menuRun.genCode` |
| Panel labels | `panel.*` | `panel.project` |
| Editor area | `editor.*` | `editor.welcome` |
| Model area | `model.*` | `model.chatMode` |
| Bottom panel | `bottom.*` | `bottom.log` |
| Dialogs/modals | `dialog.*` | `dialog.confirmDelete` |
| Common actions | `common.*` | `common.confirm`, `common.cancel` |
| Errors | `error.*` | `error.networkFailed` |

## Anti-Patterns (DO NOT)

```vue
<!-- WRONG: Hardcoded Chinese -->
<span>保存成功</span>
<el-button>确认</el-button>

<!-- WRONG: Hardcoded English -->
<span>Save successful</span>

<!-- WRONG: String in script without i18n -->
const msg = '操作失败'

<!-- CORRECT -->
<span>{{ t.common.saveSuccess }}</span>
<el-button>{{ t.common.confirm }}</el-button>
```

## Checklist (run mentally before completing any UI task)

- [ ] No hardcoded user-visible strings in templates
- [ ] No hardcoded user-visible strings in `<script>` that reach the UI
- [ ] New keys added to BOTH `zh-CN.ts` and `en-US.ts`
- [ ] Key structures match exactly between both files
- [ ] `npx vue-tsc --noEmit` passes (catches structural mismatches)
- [ ] Existing i18n keys reused where meaning matches (no duplicates)
