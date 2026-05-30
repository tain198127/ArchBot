#!/bin/bash
# 只准备当前平台的 Runtime 包（开发用）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

case "$(uname -s)" in
  Darwin)
    ARCH=$(uname -m)
    case "${ARCH}" in
      arm64) PLATFORM="darwin-arm64" ;;
      x86_64) PLATFORM="darwin-x64" ;;
      *) echo "Unknown arch: ${ARCH}"; exit 1 ;;
    esac
    ;;
  Linux)
    PLATFORM="linux-x64"
    ;;
  MINGW*|MSYS*|CYGWIN*)
    PLATFORM="win32-x64"
    ;;
  *)
    echo "Unknown OS: $(uname -s)"
    exit 1
    ;;
esac

echo "Detected platform: ${PLATFORM}"
echo ""

bash "${SCRIPT_DIR}/prepare-claude-code.sh" 2.1.128 "${PLATFORM}"
bash "${SCRIPT_DIR}/prepare-hermes.sh" 0.14.0 "${PLATFORM}"
bash "${SCRIPT_DIR}/prepare-opencode.sh" 1.15.10 "${PLATFORM}"
bash "${SCRIPT_DIR}/prepare-openclaw.sh" 2026.3.13 "${PLATFORM}"

echo ""
echo "=== Verifying ==="
bash "${SCRIPT_DIR}/verify-checksums.sh"
