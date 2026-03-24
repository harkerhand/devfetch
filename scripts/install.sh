#!/usr/bin/env bash
set -euo pipefail

BIN_NAME="devfetch"
INSTALL_DIR="${DEVFETCH_INSTALL_DIR:-${HOME}/.local/bin}"
REPO="${DEVFETCH_REPO:-harkerhand/devfetch}"
VERSION="${DEVFETCH_VERSION:-latest}"

uname_s="$(uname -s)"
case "${uname_s}" in
    Linux*) platform="linux" ;;
    Darwin*) platform="macos" ;;
    *)
        echo "[install] 不支持的平台: ${uname_s}"
        exit 1
        ;;
esac

asset="${BIN_NAME}-${platform}"
if [[ "${VERSION}" == "latest" ]]; then
    download_url="https://github.com/${REPO}/releases/latest/download/${asset}"
else
    download_url="https://github.com/${REPO}/releases/download/${VERSION}/${asset}"
fi

mkdir -p "${INSTALL_DIR}"
tmp_bin="$(mktemp)"
trap 'rm -f "${tmp_bin}"' EXIT

echo "[install] 下载 ${download_url}"
curl -fL "${download_url}" -o "${tmp_bin}"

cp "${tmp_bin}" "${INSTALL_DIR}/${BIN_NAME}"
chmod +x "${INSTALL_DIR}/${BIN_NAME}"

echo "[install] 安装完成: ${INSTALL_DIR}/${BIN_NAME}"
if [[ ":${PATH}:" != *":${INSTALL_DIR}:"* ]]; then
    echo "[install] 提示: ${INSTALL_DIR} 不在 PATH 中，请手动加入。"
fi
