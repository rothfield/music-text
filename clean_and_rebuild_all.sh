#!/bin/bash
set -e

echo "🧹 Clean and Rebuild All Components"
echo "=================================="

# Stop any running servers
echo "🛑 Stopping HTTP server..."
kill $(cat .server.pid 2>/dev/null) 2>/dev/null || true
rm -f .server.pid

# Clean all build artifacts
echo "🗑️  Cleaning build artifacts..."
cargo clean
rm -rf pkg/

# Rebuild WASM package
echo "🔧 Building WASM package..."
wasm-pack build --target web --out-dir pkg

# Rebuild Rust binaries (skip data_generator due to missing deps)
echo "🦀 Building Rust binaries..."
cargo build --release --bin cli --bin get_vexflow_fsm

# Start HTTP server
echo "🌐 Starting HTTP server on port 8000..."
python3 -m http.server 8000 > /dev/null 2>&1 & echo $! > .server.pid

echo "✅ Rebuild complete!"
echo "🔗 Test at: http://localhost:8000/test_vexflow_simple.html"
echo "📊 Server PID: $(cat .server.pid)"