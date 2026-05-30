#!/bin/bash
# 预打包 OpenCode
# 用法: ./prepare-opencode.sh <version> <platform>
# 示例: ./prepare-opencode.sh 1.15.10 darwin-arm64
set -euo pipefail

VERSION="${1:-1.15.10}"
PLATFORM="${2:-darwin-arm64}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CACHE_DIR="${SCRIPT_DIR}/../cache/opencode/${VERSION}/${PLATFORM}"

mkdir -p "${CACHE_DIR}"

echo "=== Preparing OpenCode ${VERSION} for ${PLATFORM} ==="

case "${PLATFORM}" in
  darwin-arm64)  FILE="opencode-darwin-arm64.tar.gz" ;;
  darwin-x64)    FILE="opencode-darwin-x64.tar.gz" ;;
  linux-x64)     FILE="opencode-linux-x64.tar.gz" ;;
  win32-x64)     FILE="opencode-win32-x64.zip" ;;
  *) echo "Unknown platform: ${PLATFORM}"; exit 1 ;;
esac

URL="https://github.com/anomalyco/opencode/releases/download/v${VERSION}/${FILE}"

TMPDIR=$(mktemp -d)
curl -fsSL "${URL}" -o "${TMPDIR}/${FILE}"

OUTPUT="${CACHE_DIR}/opencode-${VERSION}-${PLATFORM}.tar.gz"

if [[ "${FILE}" == *.zip ]]; then
  unzip -q "${TMPDIR}/${FILE}" -d "${TMPDIR}/extracted"
  tar -czf "${OUTPUT}" -C "${TMPDIR}/extracted" .
else
  cp "${TMPDIR}/${FILE}" "${OUTPUT}"
fi

shasum -a 256 "${OUTPUT}" | awk '{print $1}' > "${CACHE_DIR}/opencode-${VERSION}-${PLATFORM}.sha256"
ACTUAL=$(cat "${CACHE_DIR}/opencode-${VERSION}-${PLATFORM}.sha256")

cat > "${CACHE_DIR}/opencode-${VERSION}-${PLATFORM}.metadata.json" << EOF
{
  "runtime": "opencode",
  "version": "${VERSION}",
  "platform": "${PLATFORM}",
  "sha256": "${ACTUAL}",
  "prepared_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

rm -rf "${TMPDIR}"
echo "✅ Prepared: ${OUTPUT}"
echo "   SHA256: ${ACTUAL}"
