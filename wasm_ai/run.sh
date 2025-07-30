#!/bin/bash
set -e

echo "🔧 Building WASM..."
cargo build --target wasm32-unknown-unknown --release

wasm-bindgen ./target/wasm32-unknown-unknown/release/core.wasm \
  --out-dir ./front/pkg \
  --target web

miniserve ./front --index "index.html" -p 8080   