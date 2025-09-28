# Zellij Development Environment for Music Text Parser

## Overview

This project uses **Zellij** as the terminal multiplexer for development. Zellij provides a modern alternative to tmux with better defaults and configuration.

## Quick Start

```bash
# Start the development environment
zellij --layout music-text-dev.kdl
```

## Zellij Layout Configuration

The development environment is defined in `music-text-dev.kdl` with three key areas:

### Layout Structure

```
┌─────────────────────────────────────────────────────────┐
│                  Claude CLI (Main)                      │
├─────────────────────────────────┬───────────────────────┤
│          Server Logs            │       Shell           │
│     (Upper Right - TAIL)        │   (Lower Right)       │
│                                 │                       │
│ cargo-watch with tail output    │  Manual commands      │
└─────────────────────────────────┴───────────────────────┘
```

### Pane Breakdown

1. **Claude CLI Pane (Main/Left)**
   - Full-height left pane
   - Runs `make claude-cli` for AI assistance
   - Primary workspace for development tasks

2. **Server Logs Pane (Upper Right)**
   - **CRITICAL: This pane runs tail output for development logs**
   - Automatically starts cargo-watch for live rebuilds
   - Monitors `src/` directory changes
   - Displays server logs with timestamps
   - Shows compilation errors and warnings in real-time

3. **Shell Pane (Lower Right)**
   - General purpose shell for manual commands
   - Used for git operations, testing, debugging
   - Available for any ad-hoc development tasks

## Tail Command Usage in Development

### Server Logs Pane (Upper Right)

The upper right pane automatically runs cargo-watch which provides tail-like output:

```bash
# What runs automatically in server logs pane:
cargo-watch -w src/ -c -x 'run -- --web'
```

This provides continuous monitoring of:
- Compilation status
- Server startup/shutdown
- HTTP request logs
- Error messages
- Performance metrics

### Manual Tail Commands

For additional log monitoring in the shell pane:

```bash
# Monitor development log file
tail -f development.log

# Monitor system logs
tail -f /var/log/syslog

# Follow multiple files
tail -f development.log /tmp/music-text-debug.log
```

### Tail Options Reference

```bash
# Follow file (most common for development)
tail -f filename

# Show last N lines and follow
tail -n 50 -f development.log

# Follow with colored output (if supported)
tail -f development.log | grep --color=always "ERROR\|WARN"
```

## Zellij Key Bindings

### Basic Navigation
- `Alt + h/j/k/l` - Navigate between panes (vim-style)
- `Alt + [/]` - Switch between tabs
- `Alt + n` - Create new tab
- `Alt + x` - Close current pane

### Copy Mode (Text Selection)
1. `Ctrl + s + [` - Enter copy mode
2. Use arrow keys or vim bindings to navigate
3. `Space` - Start selection
4. `Enter` - Copy selection to clipboard
5. `Esc` - Exit copy mode

### Pane Management
- `Alt + +/-` - Resize current pane
- `Alt + z` - Toggle pane fullscreen
- `Ctrl + s + d` - Detach from session
- `Ctrl + s + q` - Quit Zellij

## Development Workflow

### Standard Development Session

1. **Start Environment**
   ```bash
   zellij --layout music-text-dev.kdl
   ```

2. **Monitor Server Logs (Upper Right Pane)**
   - Automatically shows cargo-watch output
   - Watch for compilation errors
   - Monitor HTTP requests at http://localhost:3000

3. **Use Claude CLI (Main Pane)**
   - AI assistance for development tasks
   - Code generation and debugging help

4. **Manual Commands (Lower Right Shell)**
   ```bash
   # Test endpoints
   curl http://localhost:3000/api/parse?input="1-2-3"
   
   # Check processes
   lsof -i :3000
   
   # Kill stuck server
   pkill -f "music-text --web"
   
   # Run tests
   cargo test
   ```

### Log Monitoring Best Practices

1. **Always check the upper right pane first** - it shows live compilation and server status
2. **Use tail -f for persistent logs** - development.log accumulates over time
3. **Monitor multiple sources** - server logs, compilation output, and HTTP access logs
4. **Color-code important messages** - use grep with --color for ERROR/WARN highlighting

### Common Development Tasks

#### Server Management
```bash
# Start development server (automatic in upper right pane)
cargo run -- --web

# Check server status
curl http://localhost:3000/

# Kill and restart
pkill -f "music-text --web" && cargo run -- --web
```

#### Log Analysis
```bash
# Find recent errors
tail -100 development.log | grep ERROR

# Monitor real-time
tail -f development.log | grep -E "(ERROR|WARN|INFO)"

# Check server startup issues
tail -50 development.log | grep "server\|bind\|port"
```

## Troubleshooting

### Port 3000 Already in Use
```bash
# Find process using port 3000
lsof -i :3000

# Kill the process
pkill -f "music-text --web"

# Or kill by PID
kill $(lsof -t -i :3000)
```

### Zellij Session Issues
```bash
# List active sessions
zellij list-sessions

# Attach to existing session
zellij attach session-name

# Kill all sessions
zellij kill-all-sessions
```

### Log File Issues
```bash
# Clear development log if it gets too large
> development.log

# Rotate logs manually
mv development.log development.log.old
touch development.log
```

## Configuration Files

### music-text-dev.kdl Layout
- Defines the three-pane development setup
- Configures automatic cargo-watch in server logs pane
- Sets working directory to `/home/john/projects/music-text`

### Key Features of the Layout
1. **Automatic server startup** with cargo-watch
2. **Live reload** on source code changes
3. **Integrated logging** with tail-like output in upper right pane
4. **Clean separation** of AI assistance, monitoring, and manual tasks

## Advanced Usage

### Custom Layouts
Create additional layouts for specific tasks:

```bash
# Create a testing layout
cp music-text-dev.kdl music-text-test.kdl
# Edit to add test runners, coverage tools, etc.
```

### Session Management
```bash
# Start named session
zellij --session dev-session --layout music-text-dev.kdl

# Detach and reattach later
zellij attach dev-session
```

### Log Aggregation
```bash
# Combine multiple log sources in shell pane
tail -f development.log /var/log/nginx/access.log | grep music-text
```

The upper right pane provides continuous tail-like monitoring of the development server, making it easy to track changes, errors, and performance in real-time during development.