#!/bin/bash

# rebuild_and_restart.sh
# Rebuilds all binaries, WASM module and restarts the web server

set -e

echo "🔄 Starting rebuild at $(date '+%Y-%m-%d %H:%M:%S')..."
echo ""
echo "📦 Building CLI binary..."
cargo build --release --bin cli

echo "📦 Building VexFlow FSM binary..."
cargo build --release --bin get_vexflow_fsm

echo "📦 Building data generator binary (if deps available)..."
cargo build --release --bin data_generator || echo "  ⚠️  Data generator skipped (missing dependencies)"

echo "🔄 Rebuilding WASM module..."
./rebuild_wasm.sh

echo "🛑 Stopping existing web server..."
pkill -f "node server.js" || true

echo "⏳ Waiting for server to stop..."
sleep 2

echo "🚀 Starting web server..."
node server.js

echo "⏳ Waiting for server to start..."
sleep 3

echo "✅ Checking server status..."
if ps aux | grep -q "[n]ode server.js"; then
    echo "🎉 Web server is running at http://localhost:3000"
    echo "📁 Server log: server.log"
    echo "✨ Rebuild completed at $(date '+%Y-%m-%d %H:%M:%S')"
else
    echo "❌ Server failed to start. Check server.log for details."
    exit 1
fi
