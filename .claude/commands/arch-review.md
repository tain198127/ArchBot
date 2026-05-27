---
name: "Code Review: Arch Review"
description: Manual code review — runs fast + deep checks on a file
category: Code Quality
tags: [review, quality, security, architecture]
argument: "<file-path>"
---

# /arch-review — Manual Code Review

Run a complete code quality review on the specified file: fast checks (ESLint,
jscpd, complexity, clippy, cargo audit) + deep review (architecture, security,
performance, anti-tamper).

**Arguments:**
- `<file-path>` — Path to the file to review (required)
- `--fast-only` — Only run fast checks, skip deep review

**What it does:**

1. Runs `.claude/scripts/code-check.sh --file <file-path> --deep`
2. If deep review is needed, invokes the code-check agent for architecture,
   security, performance, and anti-tamper analysis
3. Auto-fixes what it can (ESLint --fix, clippy --fix, missing comments)
4. Presents a graded report with ❌ block / ⚠️ warn / ✅ pass for each dimension
5. Allows interactive choice: block / allow this edit / allow all this session

**Example:**
```
/arch-review src/components/Login.vue
/arch-review src-tauri/src/main.rs
```
