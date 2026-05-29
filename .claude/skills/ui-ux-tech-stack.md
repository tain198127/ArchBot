---
name: ui-ux-tech-stack
description: |
  Enforces the ArchBot UI/UX technology stack specification. Prohibits banned
  libraries, enforces Tailwind-only styling, PrimeVue as sole component library,
  lazy loading for heavy components, WCAG 2.2 AA accessibility, and mandatory
  Storybook stories for all reusable components. Auto-triggered on all frontend
  file modifications.
globs:
  - "src/**/*.vue"
  - "src/**/*.ts"
  - "src/**/*.css"
  - "package.json"
  - "vite.config.ts"
---

# UI/UX Tech Stack Skill

## Rule: One Stack, One Way

ArchBot has exactly ONE approved UI technology stack. Every frontend file must comply. Deviations are blocked unless explicitly approved.

---

## Part 1: Banned Libraries (BLOCK)

The following libraries MUST NOT appear in `package.json`, imports, or anywhere in the codebase:

| Banned | Reason |
|--------|--------|
| `element-plus` | Replaced by PrimeVue |
| `ant-design-vue` | Conflicts with PrimeVue |
| `vuetify` | Conflicts with PrimeVue |
| `quasar` | Conflicts with PrimeVue |
| `@nuxt/ui` | Not a Nuxt project |
| `ag-grid-enterprise` | Commercial license risk |
| `ag-grid-community` | PrimeVue DataTable covers same needs |
| Any icon library other than `lucide-vue-next` | Single icon source |

**Check on every change**: If a new dependency is added to `package.json`, verify it is NOT in the banned list.

---

## Part 2: Styling — Tailwind CSS v4 Only (BLOCK)

### Rule

**All styling MUST use Tailwind CSS utility classes. Custom CSS/SCSS files and `<style>` blocks are forbidden.**

### Allowed

```vue
<template>
  <!-- Tailwind classes directly in template -->
  <div class="flex items-center gap-2 px-4 py-2 bg-surface text-on-surface">
    <span class="text-sm font-medium">Label</span>
  </div>
</template>
```

### Blocked

```vue
<!-- BLOCKED: <style> block -->
<style scoped>
.my-container {
  display: flex;
  padding: 1rem;
}
</style>

<!-- BLOCKED: inline styles -->
<div style="display: flex; padding: 1rem;">
```

### Exception

Tailwind `@layer` directives in a single `src/tailwind.css` entry file are permitted for base styles, theme tokens, and Volt component overrides. This is the ONLY `.css` file allowed.

### Dark Theme (BLOCK if missing)

Every component that renders visible UI MUST support dark theme using Tailwind's `dark:` variant:

```vue
<div class="bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100">
```

Components without `dark:` classes on color-related utilities are incomplete.

---

## Part 3: Component Library — PrimeVue Only (BLOCK)

### Rule

**PrimeVue (Unstyled mode) is the sole UI component library. Volt provides the Tailwind-wrapped styled variants.**

### Import Pattern

```typescript
// CORRECT: named import from primevue
import { Button } from 'primevue'
import { DataTable } from 'primevue'
import { Dialog } from 'primevue'

// CORRECT: Volt styled component (Tailwind-wrapped PrimeVue)
import { VButton } from '@/components/base/VButton.vue'

// BLOCKED: any other component library
import { ElButton } from 'element-plus'
import { AButton } from 'ant-design-vue'
import { VBtn } from 'vuetify'
```

### Icon Pattern

```typescript
// CORRECT: only lucide-vue-next
import { Search, Settings, ChevronRight } from 'lucide-vue-next'

// BLOCKED: any other icon source
import { Icon } from '@iconify/vue'
import { SearchIcon } from '@heroicons/vue'
```

---

## Part 4: Heavy Component Lazy Loading (BLOCK)

### Rule

**Monaco Editor, Vue Flow, and ECharts MUST be dynamically imported. They must never appear in the initial bundle.**

### Required Pattern

```typescript
// CORRECT: dynamic import, loaded on demand
const MonacoEditor = defineAsyncComponent(() => import('@/components/base/MonacoEditor.vue'))
const FlowChart = defineAsyncComponent(() => import('@/components/domain/FlowChart.vue'))
const ChartPanel = defineAsyncComponent(() => import('@/components/domain/ChartPanel.vue'))

// CORRECT: route-level lazy loading
const routes = [
  { path: '/code-view', component: () => import('@/views/CodeView.vue') },
  { path: '/graph-view', component: () => import('@/views/GraphView.vue') },
]

// BLOCKED: static import of heavy component
import MonacoEditor from '@/components/base/MonacoEditor.vue'
import EditorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker'
```

### For Read-Only Code Snippets

Prefer Shiki over Monaco:

```typescript
// CORRECT: use Shiki for syntax highlighting (read-only)
import { codeToHtml } from 'shiki'
```

---

## Part 5: Data Tables — PrimeVue DataTable (BLOCK)

### Rule

**All data tables must use PrimeVue DataTable or TreeTable by default.**

```vue
<!-- CORRECT: PrimeVue DataTable -->
<DataTable :value="items" paginator :rows="25" stripedRows>
  <Column field="name" header="Name" />
  <Column field="status" header="Status" />
</DataTable>

<!-- BLOCKED: AG Grid (any edition) -->
<AgGridVue :rowData="items" :columnDefs="columns" />
```

### TanStack Table Exception

TanStack Table is permitted ONLY when PrimeVue DataTable cannot satisfy a highly custom requirement, and ONLY with:
1. Tailwind CSS styling (no custom CSS)
2. A complete Storybook story
3. Explicit approval noted in the component file as `// ui-stack: tanstack-table-approved <reason>`

---

## Part 6: Graph Visualization — Vue Flow Layered Loading (BLOCK)

### Rule

**All graph/flow views must implement layered loading with max 150 nodes per view.**

### Required Layers

| Layer | Content | Default |
|-------|---------|---------|
| 1 | System / Business Domain | Visible |
| 2 | Modules | Visible |
| 3 | Interfaces | Hidden (expand on click) |
| 4 | Evidence Details | Hidden (expand on click) |

### Required Pattern

```typescript
// Component must track expanded state per layer
const expandedLayers = ref(new Set([1, 2])) // Only layers 1-2 visible initially

// Node count guard
const visibleNodes = computed(() => {
  const nodes = graphNodes.value.filter(n => expandedLayers.value.has(n.layer))
  if (nodes.length > 150) {
    console.warn(`Node limit exceeded: ${nodes.length} > 150`)
    return nodes.slice(0, 150)
  }
  return nodes
})
```

---

## Part 7: Accessibility — WCAG 2.2 AA (BLOCK)

### Rule

**All interactive elements must be keyboard-operable. All components must pass Storybook a11y checks.**

### Checklist (per component)

- [ ] All buttons, links, inputs reachable via `Tab`
- [ ] Focus indicators visible (`focus-visible:ring-2` or equivalent Tailwind)
- [ ] Modals trap focus and restore on close
- [ ] Dropdown menus navigable with arrow keys
- [ ] `aria-label` on icon-only buttons
- [ ] `role` attributes on custom interactive elements
- [ ] Color contrast ≥ 4.5:1 (AA) in both light and dark themes
- [ ] Storybook addon-a11y passes with zero violations

### Keyboard Pattern

```vue
<!-- CORRECT: keyboard-accessible interactive element -->
<button
  class="focus-visible:ring-2 focus-visible:ring-primary rounded"
  @click="handleAction"
  :aria-label="t.common.settings"
>
  <Settings :size="18" />
</button>

<!-- BLOCKED: non-focusable click target -->
<div @click="handleAction">
  <Settings :size="18" />
</div>
```

---

## Part 8: i18n — No Hardcoded Strings (BLOCK)

### Rule

**All user-visible text MUST use Vue I18n keys. Default language is English.**

This extends the existing `i18n-sync` skill with one addition:

- **Default language is English** (`en-US.ts` is the primary source when designing new features)
- Chinese (`zh-CN.ts`) is the secondary language
- UX copy design starts from English, then translates to Chinese

---

## Part 9: Storybook — Mandatory Stories (BLOCK)

### Rule

**Every reusable component in `src/components/base/` and `src/components/domain/` MUST have a corresponding Story.**

### Story Requirements

Each Story must include:

```typescript
// ComponentName.stories.ts
import type { Meta, StoryObj } from '@storybook/vue3'
import ComponentName from './ComponentName.vue'

const meta: Meta<typeof ComponentName> = {
  title: 'Base/ComponentName',  // or 'Domain/ComponentName'
  component: ComponentName,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof ComponentName>

// REQUIRED: Light theme
export const Light: Story = { args: { /* ... */ } }

// REQUIRED: Dark theme
export const Dark: Story = {
  args: { /* ... */ },
  parameters: { backgrounds: { default: 'dark' } },
}

// REQUIRED: Chinese long text
export const ChineseLongText: Story = {
  args: { label: '这是一个非常长的中文字符串用于测试组件的文本溢出行为和换行效果' },
}

// REQUIRED: English long text
export const EnglishLongText: Story = {
  args: { label: 'This is an extremely long English string designed to test overflow wrapping and truncation' },
}

// REQUIRED: Keyboard interaction test
export const KeyboardInteraction: Story = {
  args: { /* ... */ },
  play: async ({ canvasElement }) => {
    // Keyboard navigation test
  },
}
```

### Pre-Creation Check

Before creating a new page or component:
1. Check `src/components/base/` and `src/components/domain/` for reusable matches
2. Check existing Storybook for similar Stories
3. If nothing matches, create the component AND its Story together

---

## Part 10: Typography

| Usage | Font | Tailwind Config |
|-------|------|-----------------|
| Interface | Inter | `font-sans: 'Inter', ...` |
| Code | JetBrains Mono | `font-mono: 'JetBrains Mono', ...` |

```vue
<!-- Interface text -->
<span class="font-sans">Panel Title</span>

<!-- Code display -->
<pre class="font-mono text-sm"><code>const x = 1</code></pre>
```

---

## Part 11: Project Structure Enforcement

```
src/components/
├── base/        # Reusable base components — for ALL shared UI elements
├── domain/      # Business domain components — feature-specific panels, views
└── layout/      # Layout components — shell, sidebar, toolbar, split panes
```

New components must be placed in the correct directory:
- Shared/simple components → `base/`
- Feature-specific complex components → `domain/`
- App shell, navigation, structural → `layout/`

---

## Part 12: PrimeVue Import Strategy

### Rule: Import only what you use

```typescript
// CORRECT: individual named imports
import { Button, DataTable, Column, Dialog, InputText } from 'primevue'

// BLOCKED: full registry import
import PrimeVue from 'primevue/config'
import * as PrimeVueComponents from 'primevue'
```

---

## Quick Reference Checklist

Before completing ANY frontend change, verify:

- [ ] No banned libraries in imports or package.json
- [ ] All styling uses Tailwind classes — no `<style>` blocks, no `.css`/`.scss` files
- [ ] Dark theme `dark:` variants present on all color utilities
- [ ] Only PrimeVue components used (no Element Plus, Ant Design, etc.)
- [ ] Only Lucide icons used
- [ ] Monaco/Vue Flow/ECharts use dynamic `import()` or `defineAsyncComponent`
- [ ] Data tables use PrimeVue DataTable (not AG Grid)
- [ ] Graph views implement layered loading (max 150 nodes, layers 1-2 default)
- [ ] All interactive elements keyboard-accessible with visible focus indicators
- [ ] All user-visible text uses i18n keys (English default)
- [ ] Reusable components have complete Storybook Stories (light, dark, zh, en, keyboard, a11y)
- [ ] Component placed in correct directory (base/domain/layout)
- [ ] Font usage: Inter for UI text, JetBrains Mono for code
- [ ] PrimeVue imports are named, not full registry
