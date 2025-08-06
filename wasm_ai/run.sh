#!/bin/bash
set -e

if ! command -v wasmedge &> /dev/null
then
    echo "Installing WasmEdge"
    ./install_wasmedge.sh
else
    echo "WasmEdge already installed"
fi

cargo clean

[ -f "./core_bin.wasm" ] && rm "./core_bin.wasm"

curl -z tinyllama-1.1b-chat-v1.0.Q5_K_M.gguf -LO https://huggingface.co/second-state/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/tinyllama-1.1b-chat-v1.0.Q5_K_M.gguf


RUSTFLAGS="--cfg=wasmedge --cfg=tokio_unstable" cargo build -p core --target wasm32-wasip1 --release

cp "./target/wasm32-wasip1/release/core_bin.wasm" "./"

# Paths
model_file="./tinyllama-1.1b-chat-v1.0.Q5_K_M.gguf"
wasm_file="./core_bin.wasm"
output_file="./core_bin.wasm"

# Ensure model exists
[[ -f "$model_file" ]] || { echo "Model file not found: $model_file"; exit 1; }

# Copy if different
[[ "$wasm_file" != "$output_file" ]] && cp "$wasm_file" "$output_file"

# Run

wasmedge --dir .:. --nn-preload tinyllama:GGML:AUTO:$model_file "$output_file" --ctx-size 32000 --max-memory-pages 65536 --enable-all
