#!/bin/bash

#Unused script for running project
set -e


cargo build -p prepare --release 
cargo r -p prepare

cargo build -p core --target wasm32-wasip1 --release
cargo build -p runner --release

cargo run -p runner

# Command to run some binary with wasmtime
# wasmtime run \
#   -S preview2=n \
#   -S threads=y \
#   ./target/wasm32-wasip1-threads/release/core_bin.wasm \
#   --invoke run

