#!/bin/bash

# rebuild.sh
# Rebuilds all Rust binaries and WASM module (without restarting server)

set -e

echo "🔄 Rebuilding all Rust binaries..."
echo "  📦 Building CLI binary..."
cargo build --release --bin cli

echo "  📦 Building VexFlow FSM binary..."
cargo build --release --bin get_vexflow_fsm

echo "  📦 Building data generator binary (if deps available)..."
cargo build --release --bin data_generator || echo "  ⚠️  Data generator skipped (missing dependencies)"

echo "🔄 Rebuilding WASM module..."
./rebuild_wasm.sh

echo "✅ All binaries and WASM module rebuilt successfully!"
echo "📁 Binaries available in target/release/"
echo "   - cli"
echo "   - get_vexflow_fsm" 
echo "   - data_generator (if built)"