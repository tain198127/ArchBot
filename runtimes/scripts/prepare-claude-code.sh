#!/bin/bash
# 预打包 Claude Code
# 用法: ./prepare-claude-code.sh <version> <platform>
# 示例: ./prepare-claude-code.sh 2.1.128 darwin-arm64
set -euo pipefail

VERSION="${1:-2.1.128}"
PLATFORM="${2:-darwin-arm64}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CACHE_DIR="${SCRIPT_DIR}/../cache/claude-code/${VERSION}/${PLATFORM}"

mkdir -p "${CACHE_DIR}"

echo "=== Preparing Claude Code ${VERSION} for ${PLATFORM} ==="

TMPDIR=$(mktemp -d)
cd "${TMPDIR}"

case "${PLATFORM}" in
  darwin-arm64) NPM_PKG="@anthropic-ai/claude-code-darwin-arm64@${VERSION}" ;;
  darwin-x64)   NPM_PKG="@anthropic-ai/claude-code-darwin-x64@${VERSION}" ;;
  linux-x64)    NPM_PKG="@anthropic-ai/claude-code-linux-x64@${VERSION}" ;;
  win32-x64)    NPM_PKG="@anthropic-ai/claude-code-win32-x64@${VERSION}" ;;
  *) echo "Unknown platform: ${PLATFORM}"; exit 1 ;;
esac

npm init -y > /dev/null 2>&1
npm pack "${NPM_PKG}" --pack-destination ./downloads 2>&1
TARBALL=$(ls ./downloads/*.tgz | head -1)

mkdir -p ./extracted
tar -xzf "${TARBALL}" -C ./extracted

OUTPUT="${CACHE_DIR}/claude-code-${VERSION}-${PLATFORM}.tar.gz"
tar -czf "${OUTPUT}" -C ./extracted/package .

shasum -a 256 "${OUTPUT}" | awk '{print $1}' > "${CACHE_DIR}/claude-code-${VERSION}-${PLATFORM}.sha256"
ACTUAL=$(cat "${CACHE_DIR}/claude-code-${VERSION}-${PLATFORM}.sha256")

cat > "${CACHE_DIR}/claude-code-${VERSION}-${PLATFORM}.metadata.json" << EOF
{
  "runtime": "claude_code",
  "version": "${VERSION}",
  "platform": "${PLATFORM}",
  "sha256": "${ACTUAL}",
  "prepared_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

rm -rf "${TMPDIR}"
echo "✅ Prepared: ${OUTPUT}"
echo "   SHA256: ${ACTUAL}"
