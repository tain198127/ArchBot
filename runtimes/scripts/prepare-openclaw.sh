#!/bin/bash
# 预打包 OpenClaw
# 用法: ./prepare-openclaw.sh <version> <platform>
# 示例: ./prepare-openclaw.sh 2026.3.13 darwin-arm64
set -euo pipefail

VERSION="${1:-2026.3.13}"
PLATFORM="${2:-darwin-arm64}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CACHE_DIR="${SCRIPT_DIR}/../cache/openclaw/${VERSION}/${PLATFORM}"

mkdir -p "${CACHE_DIR}"

echo "=== Preparing OpenClaw ${VERSION} for ${PLATFORM} ==="

TMPDIR=$(mktemp -d)
cd "${TMPDIR}"

# 下载 OpenClaw npm 包
npm init -y > /dev/null 2>&1
npm pack "openclaw@${VERSION}" --pack-destination ./downloads 2>&1
TARBALL=$(ls ./downloads/*.tgz | head -1)

mkdir -p ./extracted
tar -xzf "${TARBALL}" -C ./extracted

OUTPUT="${CACHE_DIR}/openclaw-${VERSION}-${PLATFORM}.tar.gz"
tar -czf "${OUTPUT}" -C ./extracted/package .

shasum -a 256 "${OUTPUT}" | awk '{print $1}' > "${CACHE_DIR}/openclaw-${VERSION}-${PLATFORM}.sha256"
ACTUAL=$(cat "${CACHE_DIR}/openclaw-${VERSION}-${PLATFORM}.sha256")

cat > "${CACHE_DIR}/openclaw-${VERSION}-${PLATFORM}.metadata.json" << EOF
{
  "runtime": "openclaw",
  "version": "${VERSION}",
  "platform": "${PLATFORM}",
  "sha256": "${ACTUAL}",
  "prepared_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

rm -rf "${TMPDIR}"
echo "✅ Prepared: ${OUTPUT}"
echo "   SHA256: ${ACTUAL}"
