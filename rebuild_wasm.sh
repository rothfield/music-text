#!/bin/bash
# This script rebuilds the WebAssembly package.
BUILD_TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
echo "🔄 Rebuilding Wasm package at $BUILD_TIMESTAMP..."

# Set build timestamp environment variables for Rust
export BUILD_DATE=$(date '+%Y-%m-%d')
export BUILD_TIME=$(date '+%H:%M:%S')

# Build to webapp/pkg directory (where server serves it from)
wasm-pack build --target web --out-dir webapp/pkg --release

echo "✅ Wasm package rebuilt successfully at $(date '+%Y-%m-%d %H:%M:%S')"

