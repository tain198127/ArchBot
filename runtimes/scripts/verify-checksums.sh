#!/bin/bash
# 校验 cache/ 下所有预打包产物的 SHA256 与 manifest.yml 一致性
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CACHE_DIR="${SCRIPT_DIR}/../cache"
MANIFEST="${SCRIPT_DIR}/../manifest.yml"

ERRORS=0
CHECKED=0

for runtime_dir in "${CACHE_DIR}"/*/; do
  runtime=$(basename "${runtime_dir}")
  for version_dir in "${runtime_dir}"/*/; do
    version=$(basename "${version_dir}")
    for platform_dir in "${version_dir}"/*/; do
      platform=$(basename "${platform_dir}")

      SHAFILE=$(ls "${platform_dir}"/*.sha256 2>/dev/null | head -1 || true)
      if [ -z "${SHAFILE}" ]; then
        echo "⚠️  Missing sha256: ${runtime} ${version} ${platform}"
        continue
      fi

      ACTUAL=$(cat "${SHAFILE}")

      case "${runtime}" in
        claude-code) PKG_NAME="claude-code-${version}-${platform}.tar.gz" ;;
        hermes)      PKG_NAME="hermes-agent-${version}-${platform}.tar.gz" ;;
        opencode)    PKG_NAME="opencode-${version}-${platform}.tar.gz" ;;
        openclaw)    PKG_NAME="openclaw-${version}-${platform}.tar.gz" ;;
        *) echo "⚠️  Unknown runtime: ${runtime}"; continue ;;
      esac

      PKG="${platform_dir}/${PKG_NAME}"
      if [ ! -f "${PKG}" ]; then
        echo "⚠️  Missing package: ${PKG_NAME}"
        continue
      fi

      EXPECTED=$(shasum -a 256 "${PKG}" | awk '{print $1}')
      if [ "${ACTUAL}" != "${EXPECTED}" ]; then
        echo "❌ Mismatch: ${runtime} ${version} ${platform}"
        echo "   Expected: ${EXPECTED}"
        echo "   Recorded: ${ACTUAL}"
        ERRORS=$((ERRORS + 1))
      else
        echo "✅ ${runtime} ${version} ${platform}"
      fi
      CHECKED=$((CHECKED + 1))
    done
  done
done

echo ""
echo "Checked: ${CHECKED}, Errors: ${ERRORS}"

if [ ${ERRORS} -gt 0 ]; then
  echo "❌ Verification failed!"
  exit 1
fi
echo "✅ All checksums verified."
