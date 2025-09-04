#!/bin/bash
# Quick start script for the unified music-text web server

echo "🎵 Quick starting unified server..."

# Build if needed
if [ ! -f "target/release/cli" ]; then
    echo "🔨 Building Rust server..."
    cargo build --release
fi

# Start the unified Rust server
echo "🚀 Starting on http://localhost:3000"
./target/release/cli --web
