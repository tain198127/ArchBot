---
name: prd-sync
description: |
  Keeps prd.yml in sync with code changes. When features are added, modified, or removed
  at the PRD level (new menu items, new panels, new config options, layout changes,
  new modes, workflow changes), update prd.yml to reflect the current state.
  Bug fixes, refactoring, and style tweaks do NOT require prd.yml updates.
globs:
  - "src/**/*.vue"
  - "src/**/*.ts"
  - "src-tauri/src/**/*.rs"
  - "prd.yml"
---

# PRD Sync Skill

## Rule: Keep prd.yml consistent with implemented features

When development work **adds, removes, or changes product-level functionality**, update `prd.yml` to reflect the current state of the product.

## What counts as PRD-level change (MUST update prd.yml)

- New menu items or menu categories added/removed
- New panels, areas, or layout regions
- New configuration options (e.g., system settings page)
- New modes or workflows (e.g., a new model panel mode)
- New iteration scope items completed or descoped
- Tech stack changes (new major dependency)
- Window behavior changes (e.g., frameless → framed)
- New user-facing capabilities or features

## What does NOT require prd.yml update (skip)

- Bug fixes
- CSS/styling tweaks
- Refactoring without functional change
- Performance optimization
- Code comments or documentation
- Internal implementation details (file structure, helper functions)
- i18n resource additions (unless they reflect a new feature)
- Test additions

## How to update prd.yml

1. Read the current `prd.yml`
2. Identify which section is affected by the change
3. Update **only** the relevant section — do not rewrite the entire file
4. Keep the YAML structure consistent with existing format
5. If a new section is needed, follow the existing naming conventions

## Sections in prd.yml and when to update them

| Section | Update when... |
|---------|---------------|
| `project` | Tech stack changes, project description changes |
| `window` | Window chrome/behavior changes |
| `layout` | Panels added/removed, layout structure changes |
| `menu_bar` | Menu items added/removed/reorganized |
| `theme` | Theme options change |
| `iteration.current.scope` | New feature implemented that was in scope |
| `iteration.current.excluded` | Item moves from excluded to implemented |

## Checklist (mental check before completing a feature task)

- [ ] Did I add a new user-visible feature? → Update prd.yml
- [ ] Did I add a new menu item? → Add it to `menu_bar.items`
- [ ] Did I add a new panel or layout area? → Update `layout.panels`
- [ ] Did I add new config options? → Update relevant section
- [ ] Was this just a bug fix / refactor / style change? → Skip prd.yml update

## Example

If you add a "system settings" menu item and settings page:

```yaml
# Add to menu_bar.items → config → groups:
- items:
    - name: 系统配置
      shortcut: null
      action: config.system

# Add new section or update layout.panels if it opens in a new area
```
