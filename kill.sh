#!/usr/bin/env fish

# Kill music-text server processes
# Works in fish shell

echo "🛑 Killing music-text servers..."

# Kill by process name
set -l killed_any false

# Kill any running cargo processes (dev server)
set cargo_pids (pgrep -f "cargo.*notation")
if test (count $cargo_pids) -gt 0
    echo "  Killing cargo processes: $cargo_pids"
    kill $cargo_pids
    set killed_any true
end

# Kill Rust web server processes
set rust_pids (pgrep -f "notation.*server|web_server")
if test (count $rust_pids) -gt 0
    echo "  Killing Rust server processes: $rust_pids"
    kill $rust_pids
    set killed_any true
end

# Kill Node.js server processes (webapp)
set node_pids (pgrep -f "node.*server")
if test (count $node_pids) -gt 0
    echo "  Killing Node.js processes: $node_pids"
    kill $node_pids
    set killed_any true
end

# Kill any HTTP servers on port 3000
set port_pids (lsof -ti:3000 2>/dev/null)
if test (count $port_pids) -gt 0
    echo "  Killing processes on port 3000: $port_pids"
    kill $port_pids
    set killed_any true
end

# Kill any Python HTTP servers (from old build scripts)
set python_pids (pgrep -f "python.*http.server")
if test (count $python_pids) -gt 0
    echo "  Killing Python HTTP servers: $python_pids"
    kill $python_pids
    set killed_any true
end

# Check for .server.pid file
if test -f .server.pid
    set pid_from_file (cat .server.pid)
    if test -n "$pid_from_file"
        echo "  Killing process from .server.pid: $pid_from_file"
        kill $pid_from_file 2>/dev/null
        set killed_any true
    end
    rm -f .server.pid
    echo "  Removed .server.pid"
end

if test "$killed_any" = "true"
    echo "✅ Server processes killed"
    # Wait a moment for processes to shut down
    sleep 1
    echo "🔍 Checking remaining processes..."
    
    # Show any remaining processes
    set remaining (pgrep -f "notation|server.*3000")
    if test (count $remaining) -gt 0
        echo "⚠️  Some processes still running: $remaining"
        echo "  Use 'kill -9 $remaining' if needed"
    else
        echo "✅ All server processes stopped"
    end
else
    echo "ℹ️  No server processes found running"
end