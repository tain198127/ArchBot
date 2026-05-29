# ArchBot UI/UX Tech Stack Specification

**Date**: 2026-05-29
**Status**: Approved
**Scope**: Full UI/UX technology framework for the ArchBot Tauri desktop application

## 1. Motivation

ArchBot is a professional desktop tool for enterprise requirement analysts, architects, developers, and testers. The UI must:

- Present a technology-forward aesthetic attractive to investors
- Be compliant for North American market launch
- Avoid commercial licensing risks (notably AG Grid Enterprise)
- Enforce engineering best practices (Storybook, accessibility, i18n)
- Support dark/light themes with full WCAG 2.2 AA compliance

## 2. Base Stack

| Layer | Technology | Version | Notes |
|-------|-----------|---------|-------|
| Desktop Shell | Tauri | 2.0 | Rust backend |
| Frontend Framework | Vue 3 | latest | Composition API + `<script setup>` |
| Build Tool | Vite | latest | Code splitting, dynamic imports |
| Language | TypeScript | latest | Strict mode, all code typed |
| Styling | Tailwind CSS | v4 | Only allowed CSS approach |
| Dark Theme | Tailwind `dark:` variant or CSS variables | — | No custom CSS files |

## 3. UI Component Architecture

### 3.1 Component Base: PrimeVue Unstyled + Volt

- **PrimeVue** in **Unstyled mode** as the component foundation
- **Volt** component set (Tailwind-wrapped, copyable components) for styled variants
- All styling applied via Tailwind classes on PrimeVue components

### 3.2 Icons

- **Lucide Vue Next** as the sole icon library
- No other icon libraries permitted

### 3.3 Prohibited Libraries

The following are **explicitly banned** to prevent dependency conflicts and stylistic inconsistency:

- Nuxt UI
- Element Plus
- Ant Design
- Vuetify
- Quasar
- Any other full component library besides PrimeVue

## 4. Data Tables

### 4.1 Default: PrimeVue DataTable / TreeTable

- Free, feature-sufficient for all standard table needs
- Use for all default data display grids

### 4.2 Prohibited: AG Grid Enterprise

- **Strictly forbidden** unless written redistribution license from AG Grid is obtained
- The free AG Grid Community edition is also discouraged — PrimeVue covers the same use case without license ambiguity

### 4.3 Future Option: TanStack Table + TanStack Virtual

- Permitted only for highly customized table requirements that PrimeVue cannot satisfy
- Must be paired with Tailwind CSS styling and Storybook stories
- No commercial restrictions

## 5. Code Display

### 5.1 Code Editor: Monaco Editor

- **Mandatory lazy loading**: dynamic `import()` only when code/diff panels are opened
- Never bundled in the initial chunk

### 5.2 Syntax Highlighting: Shiki

- Use for read-only code snippets
- Lightweight alternative that avoids loading the full Monaco bundle

## 6. Graph & Flow Visualization: Vue Flow

- **Layered loading strategy** (mandatory):
  - Layer 1: System or business domain
  - Layer 2: Modules
  - Layer 3: Interfaces
  - Layer 4: Evidence details
- Default: display only layers 1-2; expand on click
- Node limit: 150 nodes per view (performance tested)

## 7. Charts: ECharts

- Standard charting library for all data visualization
- Dynamic import required (heavy library)

## 8. Internationalization: Vue I18n

- Default language: **English** (market priority: North America)
- Secondary language: Chinese
- All user-visible strings must use i18n keys
- Follows existing `i18n-sync` skill conventions

## 9. Accessibility (WCAG 2.2 AA)

- All interactive elements must be keyboard-operable
- Focus management for modals, dropdowns, and panels
- Color contrast ratios meeting AA minimums in both themes
- Tested via Storybook accessibility addon (`@storybook/addon-a11y`)

## 10. Design System: Storybook

- **Mandatory**: every reusable component must have a Story
- Each Story must demonstrate:
  - Light theme
  - Dark theme
  - Chinese long-text scenario
  - English long-text scenario
  - Keyboard navigation
  - Accessibility check results
- Before creating a new page or component:
  1. Check for existing reusable Storybook components
  2. If none exists, create the component WITH its Story

## 11. Typography

| Usage | Font |
|-------|------|
| Interface text | Inter |
| Code display | JetBrains Mono |

## 12. Project Structure

```
src/components/
├── base/        # Reusable base components (buttons, inputs, cards, etc.)
├── domain/      # Business domain components (requirement panels, arch views, etc.)
└── layout/      # Layout components (shell, sidebar, toolbar, etc.)

stories/         # Storybook files (or co-located with components)
```

## 13. Performance Strategy

### 13.1 Code Splitting

- Route-level splitting via Vite dynamic imports
- Heavy component lazy loading:
  - Monaco Editor: `() => import('monaco-editor')`
  - Vue Flow: `() => import('@vue-flow/core')`
  - ECharts: `() => import('echarts')`

### 13.2 Tree Shaking

- PrimeVue: import only used components (no full registry import)
- All libraries: prefer named imports over wildcard imports

## 14. Mandatory Code Generation Rules

These rules are programmatically enforced by the `ui-ux-tech-stack` skill:

1. All styling must use Tailwind classes only — no `<style>` blocks, no `.css`/`.scss` files
2. Dark theme must be implemented for every component via `dark:` variants
3. Heavy components (Monaco, Vue Flow, ECharts) must NOT be loaded at app startup
4. Data tables must default to PrimeVue DataTable
5. All user-visible text must use i18n keys
6. Interactive elements must support keyboard operation
7. Only one component library (PrimeVue) — no mixing
8. Graph views must implement layered expansion
9. Complex interactions should prefer existing PrimeVue components
10. TanStack Table allowed only with Tailwind styling + Storybook story

## 15. Migration Note

This specification replaces the current Element Plus-based UI stack. Migration must be incremental — each component converted individually with its Storybook story, i18n keys, and accessibility verification completed before moving to the next.
