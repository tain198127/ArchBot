#!/bin/bash
# 预打包 Hermes
# 用法: ./prepare-hermes.sh <version> <platform>
# 示例: ./prepare-hermes.sh 0.14.0 darwin-arm64
set -euo pipefail

VERSION="${1:-0.14.0}"
PLATFORM="${2:-darwin-arm64}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CACHE_DIR="${SCRIPT_DIR}/../cache/hermes/${VERSION}/${PLATFORM}"

mkdir -p "${CACHE_DIR}"

echo "=== Preparing Hermes ${VERSION} for ${PLATFORM} ==="

PYTHON_BIN="${PYTHON_BIN:-python3.12}"

TMPDIR=$(mktemp -d)
VENV_DIR="${TMPDIR}/hermes-venv"

"${PYTHON_BIN}" -m venv "${VENV_DIR}"
source "${VENV_DIR}/bin/activate" 2>/dev/null || source "${VENV_DIR}/Scripts/activate" 2>/dev/null
pip install --quiet "hermes-agent==${VERSION}"

OUTPUT="${CACHE_DIR}/hermes-agent-${VERSION}-${PLATFORM}.tar.gz"
tar -czf "${OUTPUT}" -C "${TMPDIR}" hermes-venv

shasum -a 256 "${OUTPUT}" | awk '{print $1}' > "${CACHE_DIR}/hermes-agent-${VERSION}-${PLATFORM}.sha256"
ACTUAL=$(cat "${CACHE_DIR}/hermes-agent-${VERSION}-${PLATFORM}.sha256")

cat > "${CACHE_DIR}/hermes-agent-${VERSION}-${PLATFORM}.metadata.json" << EOF
{
  "runtime": "hermes",
  "version": "${VERSION}",
  "platform": "${PLATFORM}",
  "python_version": "$(${PYTHON_BIN} --version)",
  "sha256": "${ACTUAL}",
  "prepared_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

rm -rf "${TMPDIR}"
echo "✅ Prepared: ${OUTPUT}"
echo "   SHA256: ${ACTUAL}"
