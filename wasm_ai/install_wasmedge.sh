#!/bin/bash

#ChatGPT-generated script for installing correct version of Wasmedge runtime

set -e

curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install_v2.sh | bash -s -- -v 0.14.1

source $HOME/.wasmedge/env

echo "[+] WasmEdge installed at: $(which wasmedge)"
wasmedge --version

echo "[+] Checking for wasi-nn plugin..."
plugin_path=$(find ~/.wasmedge/plugin -name "libwasmedgePluginWasiNN.*" | head -n 1)

if [[ -f "$plugin_path" ]]; then
  echo "[✓] wasi-nn_ggml plugin installed: $plugin_path"
else
  echo "[✗] wasi-nn_ggml plugin NOT found!"
  echo "    => You may need to install it manually from:"
  echo "       https://github.com/WasmEdge/WasmEdge/releases"
  exit 1
fi
