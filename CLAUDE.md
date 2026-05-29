# ArchBot

## Project Skills

- **i18n-sync** (`.claude/skills/i18n-sync.md`) — All UI text must use the i18n system. When modifying any `.vue` or UI config file, always update both `src/i18n/zh-CN.ts` and `src/i18n/en-US.ts`. Default language is **English**. No hardcoded user-visible strings allowed.
- **prd-sync** (`.claude/skills/prd-sync.md`) — When adding/removing/changing product-level features (menu items, panels, config options, modes), update `prd.yml` to match. Bug fixes, refactoring, and style tweaks do NOT require updates.
- **function-map-sync** (`function-map.yml`) — When modifying code, check `function-map.yml` first. If aligned, proceed. If deviation is reasonable, update function-map.yml alongside the code. If unreasonable, raise concern with alternatives.
- **function-comments** (`.claude/skills/function-comments.md`) — When creating/modifying functions, sync comments. Functions > 20 lines or complexity > 5 (McCabe) MUST have comments. Use `node .claude/scripts/check-complexity.cjs <file>` to check.
- **ui-ux-tech-stack** (`.claude/skills/ui-ux-tech-stack.md`) — **MANDATORY pre-commit check.** Every frontend change must comply with: Tailwind CSS v4 only (no `<style>` blocks), PrimeVue Unstyled as sole component library, `dark:` variants required, heavy components lazy-loaded, all text via i18n, keyboard accessible, no banned libraries. See skill file for full checklist.

## Tech Stack

- Frontend: Vue 3 + TypeScript + PrimeVue (Unstyled) + Tailwind CSS v4
- Icons: @lucide/vue
- Backend: Tauri 2 (Rust)
- Build: Vite + @tailwindcss/vite
- Design system: Storybook 10 + @storybook/addon-a11y
- Fonts: Inter (UI), JetBrains Mono (code)

## Component Architecture

```
src/components/
├── base/        # Reusable Tailwind-wrapped PrimeVue components (VButton, VInput, VDialog, ...)
├── domain/      # Business feature components (DataStandardEditor, SettingsPanel, ...)
└── layout/      # Layout components (MenuBar, SplitPanel)
```

## Styling Rules (BLOCK if violated)

- **Only `src/tailwind.css`** may contain CSS. All component styling must use Tailwind utility classes.
- **No `<style>` or `<style scoped>`** blocks in `.vue` files.
- **Dark theme required** — every component that renders visible UI must support `dark:` variants.
- Legacy CSS variables (`--bg-*`, `--text-*`) are removed. Use `bg-surface-*`, `text-text-*` Tailwind tokens.

## i18n

- Library: vue-i18n v11 (Composition API mode)
- Resource files: `src/i18n/zh-CN.ts` (中文), `src/i18n/en-US.ts` (English)
- Default locale: `en-US` (North American market primary)
- Legacy access: `t.value.section.key` (computed ref)
- New access: `tt('section.key')` (vue-i18n function)
- Type-safe: `en-US.ts` must mirror `zh-CN.ts` key structure exactly
