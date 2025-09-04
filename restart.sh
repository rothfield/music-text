#!/bin/bash

# restart.sh - Smart restart script for music-text web server
# Only rebuilds if Rust source files have changed since last build

set -e  # Exit on any error

echo "🔄 Smart restart for music-text web server..."

# Define paths
RUST_BINARY="target/release/cli"
RUST_SOURCES="src/"
SERVER_PID_FILE=".server.pid"

# Function to check if rebuild is needed
needs_rebuild() {
    # If binary doesn't exist, definitely need to rebuild
    if [ ! -f "$RUST_BINARY" ]; then
        echo "🔧 Rust binary not found - rebuild needed"
        return 0
    fi
    
    # Check if any Rust source files are newer than the binary
    if [ -d "$RUST_SOURCES" ]; then
        local newer_files=$(find "$RUST_SOURCES" -name "*.rs" -newer "$RUST_BINARY" 2>/dev/null)
        if [ -n "$newer_files" ]; then
            echo "🔧 Rust source files have changed - rebuild needed:"
            echo "$newer_files" | sed 's/^/  /'
            return 0
        fi
    fi
    
    # Check if Cargo.toml is newer
    if [ -f "Cargo.toml" ] && [ "Cargo.toml" -nt "$RUST_BINARY" ]; then
        echo "📋 Cargo.toml has changed - rebuild needed"
        return 0
    fi
    
    echo "✅ Rust binary is up to date"
    return 1
}

# Function to stop existing server
stop_server() {
    if [ -f "$SERVER_PID_FILE" ]; then
        local pid=$(cat "$SERVER_PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            echo "🛑 Stopping existing server (PID: $pid)..."
            kill "$pid"
            sleep 1
            # Force kill if still running
            if kill -0 "$pid" 2>/dev/null; then
                echo "⚡ Force killing server..."
                kill -9 "$pid"
            fi
        fi
        rm -f "$SERVER_PID_FILE"
    fi
}

# Function to start server
start_server() {
    echo "🚀 Starting unified web server..."
    
    # Start the unified Rust server (serves both API and web assets)
    ./target/release/cli server &
    
    local server_pid=$!
    echo $server_pid > "$SERVER_PID_FILE"
    echo "✅ Unified server started with PID: $server_pid"
    echo "📍 Web UI: http://localhost:3000"
    echo "📍 API: http://localhost:3000/api/parse"
}

# Main execution
main() {
    # Stop existing server first
    stop_server
    
    # Rebuild Rust backend if necessary
    if needs_rebuild; then
        echo "🔨 Rebuilding Rust backend..."
        cargo build --release
        echo "✅ Rust rebuild complete"
    else
        echo "⚡ Skipping rebuild - using existing binary"
    fi
    
    # Start the server
    start_server
    
    echo ""
    echo "🎵 Unified Notation Parser server restarted!"
    echo "📱 Open http://localhost:3000 in your browser"
    echo "🔍 Server PID stored in: $SERVER_PID_FILE"
    echo ""
    echo "💡 To stop the server: kill \$(cat $SERVER_PID_FILE)"
    echo "📝 To view logs: tail -f server.log (if available)"
}

# Run main function
main "$@"