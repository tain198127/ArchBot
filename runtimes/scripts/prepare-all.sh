#!/bin/bash
# 全平台预打包（CI 用）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PLATFORMS=("darwin-arm64" "darwin-x64" "linux-x64" "win32-x64")

for PLATFORM in "${PLATFORMS[@]}"; do
  echo "=== Building for ${PLATFORM} ==="
  bash "${SCRIPT_DIR}/prepare-claude-code.sh" 2.1.128 "${PLATFORM}"
  bash "${SCRIPT_DIR}/prepare-hermes.sh" 0.14.0 "${PLATFORM}"
  bash "${SCRIPT_DIR}/prepare-opencode.sh" 1.15.10 "${PLATFORM}"
  bash "${SCRIPT_DIR}/prepare-openclaw.sh" 2026.3.13 "${PLATFORM}"
done

echo ""
echo "=== Verifying All ==="
bash "${SCRIPT_DIR}/verify-checksums.sh"
