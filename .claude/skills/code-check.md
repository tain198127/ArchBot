---
name: code-check
description: |
  Pre-edit code quality gate. Before every Edit/Write on source code, runs fast
  checks (ESLint, jscpd, complexity, clippy, cargo audit) and deep review
  (architecture, security, performance, anti-tamper) for large changes.
  Auto-fixes what it can, then presents unfixable issues for user confirmation.
globs:
  - "src/**/*.ts"
  - "src/**/*.vue"
  - "src-tauri/src/**/*.rs"
---

# Code Check Skill — Pre-Edit Quality Gate

## Rule: Check before you edit

Before making ANY Edit/Write to a source file, run the quality gate. This skill
is auto-triggered by the PreToolUse hook AND can be invoked manually via `/arch-review`.

## Thresholds

| Metric | Threshold | Level |
|--------|-----------|-------|
| Function length | 200 lines | ⚠️ warn |
| Cyclomatic complexity | 10 | ⚠️ warn |
| Duplicate code | 20 lines | ⚠️ warn |
| Security issues | ANY | ❌ block |
| Rust clippy errors | ANY | ❌ block |
| Rust vulnerabilities | ANY | ❌ block |

## Workflow

### Step 1: Check skip conditions

Check in order:
1. If `SKIP_CODE_CHECK=1` is in environment → skip all checks
2. If the file or surrounding context contains `// skip-checks: <reason>` → skip
3. Otherwise, proceed to Step 2

### Step 2: Run fast checks

```bash
.claude/scripts/code-check.sh --file <file-path> --changed-lines <N>
```

Parse the JSON output. The exit code tells you:
- `0` = all pass → proceed to Step 4
- `1` = block-level failures → go to Step 3
- `2` = warn-level only → proceed to Step 4

### Step 3: Auto-fix loop

The shell script already runs up to 3 rounds of auto-fix internally. After it
finishes, re-check the JSON output:

- If `status` is `"pass"` → proceed to Step 4
- If `status` is `"block"` → remaining issues are NOT auto-fixable

For remaining issues, check if the deep review can fix them (Step 5).

### Step 4: Determine if deep review is needed

Deep review is needed if ANY of:
- `changed_lines >= 20`
- Multiple files are being modified
- The shell script output contains `DEEP_CHECK_NEEDED=true`
- This is a `/arch-review` invocation

If deep review is NOT needed → Edit proceeds.

### Step 5: Deep review (Claude Agent)

Invoke the code-check agent:

```
Use the Agent tool with subagent_type="code-check" to review the file at <file-path>.
The agent receives the diff/context and returns findings as JSON.
```

After receiving the agent's report:

**For block-level findings (❌):**
- If `auto_fixable: true` → apply the fix, re-check
- If `auto_fixable: false` → present to user for confirmation

**For warn-level findings (⚠️):**
- If `auto_fixable: true` → apply the fix silently
- If `auto_fixable: false` → mention to user but don't block

### Step 6: User confirmation (only when unfixable issues remain)

Present the report with color indicators:

```
❌ [BLOCK] Security: XSS risk in Login.vue:42 — user input inserted via innerHTML
⚠️ [WARN]  Architecture: Login.vue directly calls Tauri command (use store)
✅ [PASS] Complexity: ok
✅ [PASS] Naming: ok
```

Then ask the user:

> **Code check found issues:**
> - ❌ 1 blocking, ⚠️ 1 warning
>
> Choose action: **A) Block this edit**, **B) Allow this edit**, **C) Allow all edits this session**

- **A**: Stop — do NOT make the Edit/Write call
- **B**: Proceed with this one edit, don't ask again for this file
- **C**: Set `SKIP_CODE_CHECK=1` for the rest of this session, proceed

### Step 7: Proceed

If checks pass or user approves → make the Edit/Write call.

## Quick Reference

```
SKIP_CODE_CHECK=1         → skip all
// skip-checks: <reason>  → skip this file/section
/arch-review <file>       → manual deep review
```

## Integration with other skills

- **function-comments**: This skill's complexity check uses the same
  `check-complexity.cjs` script. When adding/updating comments, follow
  `function-comments.md` format.
- **prd-sync**: Security fixes don't require prd.yml update. New features
  uncovered by architecture review may.
- **i18n-sync**: If auto-fix adds new user-visible strings, remember to
  sync i18n keys.
