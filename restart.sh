#!/bin/bash

# restart.sh - Smart restart script for notation parser web server
# Only rebuilds WASM if Rust source files have changed since last build

set -e  # Exit on any error

echo "🔄 Smart restart for notation parser web server..."

# Define paths
WASM_PKG="pkg/notation_parser.js"
RUST_SOURCES="src/"
SERVER_PID_FILE=".server.pid"

# Function to check if WASM needs rebuilding
needs_rebuild() {
    # If WASM package doesn't exist, definitely need to rebuild
    if [ ! -f "$WASM_PKG" ]; then
        echo "📦 WASM package not found - rebuild needed"
        return 0
    fi
    
    # Check if any Rust source files are newer than the WASM package
    if [ -d "$RUST_SOURCES" ]; then
        local newer_files=$(find "$RUST_SOURCES" -name "*.rs" -newer "$WASM_PKG" 2>/dev/null)
        if [ -n "$newer_files" ]; then
            echo "🔧 Rust source files have changed - rebuild needed:"
            echo "$newer_files" | sed 's/^/  /'
            return 0
        fi
    fi
    
    # Check if Cargo.toml is newer
    if [ -f "Cargo.toml" ] && [ "Cargo.toml" -nt "$WASM_PKG" ]; then
        echo "📋 Cargo.toml has changed - rebuild needed"
        return 0
    fi
    
    echo "✅ WASM package is up to date"
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
    echo "🚀 Starting web server..."
    
    # Check if we have a preferred server script
    if [ -f "run_webserver.sh" ]; then
        bash run_webserver.sh &
    elif [ -f "server.js" ] && command -v node >/dev/null 2>&1; then
        node server.js &
    elif command -v python3 >/dev/null 2>&1; then
        python3 -m http.server 8000 &
    elif command -v python >/dev/null 2>&1; then
        python -m SimpleHTTPServer 8000 &
    else
        echo "❌ No suitable web server found (need node, python3, or python)"
        exit 1
    fi
    
    local server_pid=$!
    echo $server_pid > "$SERVER_PID_FILE"
    echo "✅ Server started with PID: $server_pid"
}

# Main execution
main() {
    # Stop existing server first
    stop_server
    
    # Rebuild WASM if necessary
    if needs_rebuild; then
        echo "🔨 Rebuilding WASM package..."
        bash rebuild_wasm.sh
        echo "✅ WASM rebuild complete"
    else
        echo "⚡ Skipping rebuild - using existing WASM package"
    fi
    
    # Start the server
    start_server
    
    echo ""
    echo "🎵 Notation Parser web server restarted!"
    echo "📱 Open http://localhost:8000 in your browser"
    echo "🔍 Server PID stored in: $SERVER_PID_FILE"
    echo ""
    echo "💡 To stop the server: kill \$(cat $SERVER_PID_FILE)"
    echo "📝 To view logs: tail -f server.log (if available)"
}

# Run main function
main "$@"