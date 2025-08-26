#!/bin/bash
# Script to start the notation parser web server

echo "🎵 Starting Notation Parser Web Server..."

# Check if node is installed
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed. Please install Node.js first."
    exit 1
fi

# Check if webapp directory exists
if [ ! -d "webapp" ]; then
    echo "❌ webapp directory not found. Please run this script from the project root."
    exit 1
fi

# Check if WASM files exist, build if missing
if [ ! -f "webapp/pkg/notation_parser_bg.wasm" ]; then
    echo "⚠️  WASM files not found. Building WASM module..."
    ./rebuild_wasm.sh
    if [ $? -ne 0 ]; then
        echo "❌ Failed to build WASM module"
        exit 1
    fi
fi

# Start the server
echo "🚀 Starting server on http://localhost:3000"
cd webapp && node server.js