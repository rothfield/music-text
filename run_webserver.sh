#!/bin/bash
# Script to start the unified music-text web server

echo "🎵 Starting Unified Notation Parser Web Server..."

# Check if webapp directory exists
if [ ! -d "webapp" ]; then
    echo "❌ webapp directory not found. Please run this script from the project root."
    exit 1
fi

# Check if Rust backend is built
if [ ! -f "target/release/cli" ]; then
    echo "⚠️  Rust backend not found. Building..."
    cargo build --release
    if [ $? -ne 0 ]; then
        echo "❌ Failed to build Rust backend"
        exit 1
    fi
fi

# Update cache-busting versions for web assets
echo "🔄 Updating cache-busting versions..."
cd webapp && node update-cache-bust.js && cd ..

# Start the unified Rust server (serves both API and web assets)
echo "🚀 Starting unified server on http://localhost:3000"
echo "📍 Web UI: http://localhost:3000"
echo "📍 API: http://localhost:3000/api/parse"
./target/release/cli --web