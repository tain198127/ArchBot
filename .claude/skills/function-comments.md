---
name: function-comments
description: |
  Enforces function-level documentation for complex functions. When creating or modifying
  functions (parameters, logic changes), sync comments accordingly. Functions over 20 lines
  or with cyclomatic complexity > 5 MUST have documentation comments.
  Uses a Node.js script to calculate McCabe cyclomatic complexity.
globs:
  - "src/**/*.ts"
  - "src/**/*.vue"
  - "src-tauri/src/**/*.rs"
---

# Function Comments Skill

## Rule: Keep function comments in sync with implementation

When you **create** or **modify** a function (change parameters, return type, or logic), you must:

1. **Check** if the function requires a comment (see criteria below)
2. **Add** a comment if one is required but missing
3. **Update** the existing comment if the function's behavior changed

## When comments are REQUIRED

A function **MUST** have a documentation comment if ANY of these are true:

- Function body exceeds **20 lines** (excluding blank lines and closing braces)
- Cyclomatic complexity is **greater than 5**
- Function has **4 or more parameters**

## When comments are OPTIONAL (but update if present)

- Functions under 20 lines with complexity ≤ 5
- If a comment already exists, keep it accurate when the function changes
- If no comment exists and the function is simple, don't add one

## How to measure complexity

Run the following Node.js script to calculate McCabe cyclomatic complexity:

```bash
node .claude/scripts/check-complexity.cjs <file-path> [function-name]
```

The script counts decision points:
- `if`, `else if`, `?:` (ternary)
- `for`, `while`, `do...while`
- `case` (each case in switch)
- `&&`, `||` (logical operators in conditions)
- `catch`
- Rust: `match` arms (each `=>` except the last/default)

**McCabe formula**: `M = E - N + 2P` simplified to `M = decision_points + 1`

## Comment format by language

### TypeScript/JavaScript
```typescript
/**
 * Brief description of what the function does
 *
 * Business logic:
 * 1. Step one
 * 2. Step two
 * 3. Step three
 */
function complexFunction(param1: string, param2: number): Result {
```

### Rust
```rust
/// Brief description of what the function does
///
/// Business logic:
/// 1. Step one
/// 2. Step two
/// 3. Step three
fn complex_function(param1: &str, param2: u32) -> Result<T, E> {
```

### Vue `<script setup>` functions
Same as TypeScript format, placed directly above the function.

## Comment content requirements

For functions that require comments:

1. **First line**: What the function does (one sentence)
2. **Business logic section** (if complexity > 5 or lines > 20):
   - Numbered steps explaining the high-level flow
   - Focus on WHY, not WHAT (the code shows what)
   - Mention non-obvious side effects
   - Note important error handling paths

## What NOT to put in comments

- Parameter type descriptions (TypeScript/Rust type system handles this)
- Return type descriptions (same reason)
- Obvious one-to-one code narration
- `@param` / `@returns` JSDoc tags (unless it's a public library API)
- References to tickets, PRs, or dates

## Workflow

When you create or modify a function:

1. Count the lines (body only, exclude signature and closing brace)
2. If > 20 lines OR you suspect high complexity, run the complexity checker
3. If complexity > 5 or lines > 20: add/update the comment
4. If the function already has a comment and you changed its behavior: update the comment
5. If the function is simple and has no comment: leave it alone

## Checklist

- [ ] New function with > 20 lines? → Add comment with business logic steps
- [ ] New function with complexity > 5? → Add comment with business logic steps
- [ ] Modified function parameters? → Update comment if one exists
- [ ] Changed function logic? → Update comment if one exists
- [ ] Function is simple (< 20 lines, complexity ≤ 5) and has no comment? → Skip
