#!/bin/bash
#
# code-check.sh — ArchBot 快速代码检查通道
#
# 由 PreToolUse Hook 或 /arch-review 命令调用。
# 检查 ESLint / jscpd / complexity / cargo clippy / cargo audit。
# 输出 JSON 格式结果到 stdout，退出码 0=通过, 1=阻断, 2=警告。
#
# 用法:
#   .claude/scripts/code-check.sh --file <path> [--changed-lines <N>] [--deep]
#
# 参数:
#   --file <path>       要检查的文件路径（必需）
#   --changed-lines <N> 改动行数（用于决定是否触发深度通道）
#   --deep              强制执行深度通道（用于 /arch-review 命令）

set -euo pipefail

FILE=""
CHANGED_LINES=0
DEEP_FLAG=false

# ── 参数解析 ──────────────────────────────────────────────
# Hook 模式: 从 stdin 读取 JSON, 提取 file_path
if [ ! -t 0 ] && [ $# -eq 0 ]; then
  RAW="$(cat)"
  FILE=$(echo "$RAW" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('file_path',''))" 2>/dev/null || true)
  if [ -z "$FILE" ]; then
    echo '{"status":"error","message":"Could not parse file_path from stdin JSON"}'
    exit 2
  fi
  # 改动行数从 new_string/old_string/content 估算
  NEW_STR=$(echo "$RAW" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('new_string','')+d.get('content',''))" 2>/dev/null || true)
  CHANGED_LINES=$(echo "$NEW_STR" | wc -l | tr -d ' ')
fi

# 命令行模式: /arch-review 或手动调用
while [[ $# -gt 0 ]]; do
  case "$1" in
    --file) FILE="$2"; shift 2 ;;
    --changed-lines) CHANGED_LINES="$2"; shift 2 ;;
    --deep) DEEP_FLAG=true; shift ;;
    *) shift ;;
  esac
done

if [ -z "$FILE" ]; then
  echo '{"status":"error","message":"--file is required (or stdin JSON with file_path)"}'
  exit 2
fi

PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
RESULTS=()

# ── 跳过检测 ──────────────────────────────────────────────
if [ "${SKIP_CODE_CHECK:-}" = "1" ]; then
  echo '{"status":"pass","message":"SKIP_CODE_CHECK=1","checks":[]}'
  exit 0
fi

if grep -q "skip-checks:" "$FILE" 2>/dev/null; then
  echo '{"status":"pass","message":"skip-checks comment found","checks":[]}'
  exit 0
fi

EXT="${FILE##*.}"

# ── 辅助函数 ──────────────────────────────────────────────
add_result() {
  local check="$1" level="$2" status="$3" detail="$4"
  RESULTS+=("{\"check\":\"$check\",\"level\":\"$level\",\"status\":\"$status\",\"detail\":\"$detail\"}")
}

has_block() {
  for r in "${RESULTS[@]}"; do
    [[ "$r" == *'"level":"block"'* ]] && [[ "$r" == *'"status":"fail"'* ]] && return 0
  done
  return 1
}

output_json() {
  local overall="pass"
  if has_block; then overall="block"; fi
  local checks_json
  checks_json=$(printf ',%s' "${RESULTS[@]}")
  checks_json="[${checks_json:1}]"
  echo "{\"status\":\"$overall\",\"file\":\"$FILE\",\"checks\":$checks_json}"
}

# ── TypeScript / Vue 检查 ──────────────────────────────────
run_ts_checks() {
  # 1. ESLint
  if command -v npx &>/dev/null; then
    local eslint_out
    eslint_out=$(cd "$PROJECT_ROOT" && npx eslint "$FILE" --format json 2>&1) || true
    if echo "$eslint_out" | grep -q '"fatal"'; then
      add_result "eslint" "block" "fail" "ESLint returned fatal error"
    elif echo "$eslint_out" | grep -q '"severity":2'; then
      local err_count
      err_count=$(echo "$eslint_out" | grep -o '"severity":2' | wc -l | tr -d ' ')
      add_result "eslint" "block" "fail" "$err_count error(s). Run: npx eslint $FILE"
    elif echo "$eslint_out" | grep -q '"severity":1'; then
      local warn_count
      warn_count=$(echo "$eslint_out" | grep -o '"severity":1' | wc -l | tr -d ' ')
      add_result "eslint" "warn" "warn" "$warn_count warning(s)"
    else
      add_result "eslint" "block" "pass" "ESLint passed"
    fi
  else
    add_result "eslint" "warn" "skip" "npx not available"
  fi

  # 2. jscpd (重复代码, ≥20 行)
  if command -v npx &>/dev/null; then
    local jscpd_out
    jscpd_out=$(cd "$PROJECT_ROOT" && npx jscpd --min-lines 20 --min-tokens 50 "$FILE" --silent 2>&1) || true
    if echo "$jscpd_out" | grep -qi "clone found"; then
      add_result "jscpd" "warn" "warn" "Duplicate code detected (>=20 lines). Consider extracting."
    else
      add_result "jscpd" "warn" "pass" "No duplicates >=20 lines"
    fi
  else
    add_result "jscpd" "warn" "skip" "jscpd not available"
  fi

  # 3. Complexity (已有脚本)
  local comp_out
  comp_out=$(cd "$PROJECT_ROOT" && node .claude/scripts/check-complexity.cjs "$FILE" 2>&1) || true
  if echo "$comp_out" | grep -q "complexity > 10\|over threshold\|require documentation"; then
    add_result "complexity" "warn" "warn" "Functions exceed complexity threshold (10). See above."
  elif echo "$comp_out" | grep -q "All functions are within"; then
    add_result "complexity" "warn" "pass" "Complexity within limits"
  else
    add_result "complexity" "warn" "pass" "Complexity check completed"
  fi
}

# ── Rust 检查 ──────────────────────────────────────────────
run_rust_checks() {
  local rust_dir="$PROJECT_ROOT/src-tauri"

  # 4. cargo clippy
  if command -v cargo &>/dev/null; then
    local clippy_out
    clippy_out=$(cd "$rust_dir" && cargo clippy -- -D warnings 2>&1) || true
    if echo "$clippy_out" | grep -q "error:"; then
      add_result "clippy" "block" "fail" "Clippy errors found. Run: cargo clippy --fix"
    elif echo "$clippy_out" | grep -q "warning:"; then
      add_result "clippy" "warn" "warn" "Clippy warnings. Run: cargo clippy --fix"
    else
      add_result "clippy" "block" "pass" "Clippy passed"
    fi
  else
    add_result "clippy" "warn" "skip" "cargo not available"
  fi

  # 5. cargo audit
  if command -v cargo &>/dev/null && cargo audit --version &>/dev/null; then
    local audit_out
    audit_out=$(cd "$rust_dir" && cargo audit 2>&1) || true
    if echo "$audit_out" | grep -q "vulnerability"; then
      add_result "cargo-audit" "block" "fail" "Security vulnerabilities found. Run: cargo audit"
    else
      add_result "cargo-audit" "block" "pass" "No known vulnerabilities"
    fi
  else
    add_result "cargo-audit" "warn" "skip" "cargo-audit not installed. Run: cargo install cargo-audit"
  fi

  # 6. Rust complexity (已有脚本)
  local comp_out
  comp_out=$(cd "$PROJECT_ROOT" && node .claude/scripts/check-complexity.cjs "$FILE" 2>&1) || true
  if echo "$comp_out" | grep -q "complexity > 10\|over threshold"; then
    add_result "complexity" "warn" "warn" "Functions exceed complexity threshold (10)"
  else
    add_result "complexity" "warn" "pass" "Complexity within limits"
  fi
}

# ── 主逻辑 ──────────────────────────────────────────────────
case "$EXT" in
  ts|tsx|vue) run_ts_checks ;;
  rs)         run_rust_checks ;;
  *)
    add_result "all" "warn" "skip" "No checks configured for .$EXT files"
    output_json
    exit 0
    ;;
esac

# ── 自动修复 ────────────────────────────────────────────────
MAX_FIX_ROUNDS=3
for round in $(seq 1 $MAX_FIX_ROUNDS); do
  if ! has_block; then break; fi

  # 执行自动修复
  if [ "$EXT" = "ts" ] || [ "$EXT" = "tsx" ] || [ "$EXT" = "vue" ]; then
    (cd "$PROJECT_ROOT" && npx eslint "$FILE" --fix 2>&1) || true
  elif [ "$EXT" = "rs" ]; then
    (cd "$PROJECT_ROOT/src-tauri" && cargo clippy --fix --allow-dirty 2>&1) || true
  fi

  # 重新检查
  RESULTS=()
  case "$EXT" in
    ts|tsx|vue) run_ts_checks ;;
    rs)         run_rust_checks ;;
  esac
done

# ── 判断是否需要深度通道 ──────────────────────────────────
if [ "$DEEP_FLAG" = true ] || [ "$CHANGED_LINES" -ge 20 ]; then
  echo "DEEP_CHECK_NEEDED=true" >> "$PROJECT_ROOT/.claude/.code-check-state"
fi

output_json

if has_block; then
  exit 1
fi
exit 0
