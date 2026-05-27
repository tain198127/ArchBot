# ArchBot

## Project Skills

- **i18n-sync** (`.claude/skills/i18n-sync.md`) — All UI text must use the i18n system. When modifying any `.vue` or UI config file, always update both `src/i18n/zh-CN.ts` and `src/i18n/en-US.ts`. No hardcoded user-visible strings allowed.
- **prd-sync** (`.claude/skills/prd-sync.md`) — When adding/removing/changing product-level features (menu items, panels, config options, modes), update `prd.yml` to match. Bug fixes, refactoring, and style tweaks do NOT require updates.

## Tech Stack

- Frontend: Vue 3 + TypeScript + Element Plus
- Backend: Tauri 2 (Rust)
- Build: Vite

## i18n

- Resource files: `src/i18n/zh-CN.ts` (中文), `src/i18n/en-US.ts` (English)
- Usage: `const { t } = useI18n()` → `t.value.section.key`
- Type-safe: `en-US.ts` must mirror `zh-CN.ts` key structure exactly
