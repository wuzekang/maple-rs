#!/bin/bash
source ../emsdk/emsdk_env.sh

# Set environment for emscripten
# EMCC_CFLAGS for C/C++ code
export EMCC_CFLAGS="-s USE_SDL=3 -s FETCH=1"

# Build for wasm32-unknown-emscripten
# cargo build --target wasm32-unknown-emscripten --release --example future_demo -p web-fetch
cargo build --target wasm32-unknown-emscripten --release --bin client