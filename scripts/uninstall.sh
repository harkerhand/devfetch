#!/usr/bin/env bash
set -euo pipefail

BIN_NAME="devfetch"
INSTALL_DIR="${DEVFETCH_INSTALL_DIR:-${HOME}/.local/bin}"
TARGET="${INSTALL_DIR}/${BIN_NAME}"

if [[ -f "${TARGET}" ]]; then
    rm -f "${TARGET}"
    echo "[uninstall] 已删除 ${TARGET}"
else
    echo "[uninstall] 未找到 ${TARGET}"
fi
