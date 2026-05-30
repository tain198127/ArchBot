#!/bin/bash
# 交叉编译全部 Adapter
# 用法: ./build-adapters.sh [target]
# 示例: ./build-adapters.sh aarch64-apple-darwin
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TARGET="${1:-}"

ADAPTERS=(
  "archbot-adapter-claude-code"
  "archbot-adapter-hermes"
  "archbot-adapter-opencode"
  "archbot-adapter-openclaw"
)

DEFAULT_TARGETS=(
  "aarch64-apple-darwin"
  "x86_64-apple-darwin"
  "x86_64-pc-windows-msvc"
  "x86_64-unknown-linux-gnu"
)

build_adapter() {
  local adapter="$1"
  local target="$2"
  local cache_dir="${SCRIPT_DIR}/../cache/${adapter}/1.0.0/${target}"

  echo "Building ${adapter} for ${target}..."

  # Adapter 代码位于 adapters/<name>/ 目录
  local adapter_dir="${SCRIPT_DIR}/../${adapter}"
  if [ ! -d "${adapter_dir}" ]; then
    echo "⚠️  Adapter directory not found: ${adapter_dir} (skipping)"
    return
  fi

  cd "${adapter_dir}"
  cargo build --release --target "${target}"

  mkdir -p "${cache_dir}"

  local ext=""
  [[ "${target}" == *"windows"* ]] && ext=".exe"

  local src="target/${target}/release/${adapter}${ext}"
  local dest="${cache_dir}/${adapter}-1.0.0-${target}${ext}"

  cp "${src}" "${dest}"
  shasum -a 256 "${dest}" | awk '{print $1}' > "${dest}.sha256"

  echo "  -> ${dest}"
}

if [ -n "${TARGET}" ]; then
  for adapter in "${ADAPTERS[@]}"; do
    build_adapter "${adapter}" "${TARGET}"
  done
else
  for target in "${DEFAULT_TARGETS[@]}"; do
    for adapter in "${ADAPTERS[@]}"; do
      build_adapter "${adapter}" "${target}"
    done
  done
fi

echo "✅ All adapters built."
