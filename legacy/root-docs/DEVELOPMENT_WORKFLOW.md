# Development Workflow & Environment Documentation

## System Environment

### Operating System & Desktop
- **OS**: Arch Linux (6.16.3-arch1-1 kernel)
- **Display Server**: Wayland (`XDG_SESSION_TYPE=wayland`)
- **Window Manager**: Sway (Wayland compositor)
  - Socket: `/run/user/1000/sway-ipc.1000.116803.sock`
- **Architecture**: x86_64

### Terminal Multiplexer: Zellij

**Primary Development Environment**: Zellij terminal multiplexer with custom layout

#### Zellij Configuration
- **Layout File**: `music-text-dev.kdl` (in project root)
- **Running Process**: `zellij --layout music-text-dev.kdl`
- **Server**: `/usr/bin/zellij --server /run/user/1000/zellij/0.43.1/charming-cowbell`

#### Pane Layout Structure

```
┌─────────────────┬─────────────────┐
│                 │                 │
│                 │   cargo-watch   │
│                 │   (web server)  │
│   Claude CLI    │                 │
│   (AI Command   ├─────────────────┤
│    Center)      │                 │
│                 │   Shell Pane    │
│                 │   (commands)    │
│                 │                 │
└─────────────────┴─────────────────┘
       65%              35%
```

#### Pane Details

**Main Pane (65% width): "AI Command Center"**
- **Template**: `claude`
- **Command**: `make claude-cli`
- **Purpose**: Primary interaction with Claude Code assistant
- **Working Directory**: `/home/john/projects/music-text`

**Right Top Pane (17.5% of screen): "Auto-Rebuild Server"**
- **Template**: `watch`
- **Command**: `cargo-watch -c -x "run -- --web"`
- **Purpose**: Automatically rebuilds and restarts web server on file changes
- **Working Directory**: `/home/john/projects/music-text`
- **Key Feature**: Clear screen (`-c`) for clean output on rebuilds

**Right Bottom Pane (17.5% of screen): "Manual Shell"**
- **Template**: `shell`
- **Purpose**: Manual command execution, testing, git operations
- **Working Directory**: `/home/john/projects/music-text`

### Zellij Layout Configuration

```kdl
layout {
    default_tab_template {
        children
        pane size=1 borderless=true {
            plugin location="zellij:status-bar"
        }
    }
    
    pane_template name="claude" {
        command "make"
        args "claude-cli"
        start_in_place true
    }
    pane_template name="watch" {
        command "cargo-watch"
        args "-c" "-x" "run -- --web"
        start_in_place true
        cwd "/home/john/projects/music-text"
    }
    pane_template name="shell" {
        start_in_place true
    }

    tab name="AI Command Center" focus=true {
        pane split_direction="vertical" {
            claude
            pane size="35%" {
                pane split_direction="horizontal" {
                    watch
                    shell
                }
            }
        }
    }
}
```

## Development Workflow

### Primary Development Pattern

1. **Start Environment**: `zellij --layout music-text-dev.kdl`
   - Automatically launches Claude CLI in main pane
   - Auto-starts web server watcher in top-right pane
   - Shell ready for manual commands in bottom-right pane

2. **Code Development Cycle**:
   - **Main Pane**: Interact with Claude for code changes, debugging, analysis
   - **Watch Pane**: Automatically rebuilds on file saves (visual feedback)
   - **Shell Pane**: Manual testing, git operations, Playwright tests

3. **Testing Workflow**:
   - Web server runs continuously at `http://localhost:3000`
   - Browser testing via `npx playwright test` in shell pane
   - Rust unit tests via `cargo test` in shell pane

### Key Advantages of This Setup

#### Persistent Development State
- **No Manual Server Management**: Cargo-watch handles automatic rebuilds
- **Immediate Feedback**: File changes trigger rebuilds instantly
- **Context Preservation**: All panes maintain working directory and history

#### Multi-Tasking Efficiency
- **Parallel Operations**: AI assistance + auto-rebuild + manual commands simultaneously
- **Visual Status**: Can see build status, server logs, and command output at once
- **Quick Context Switching**: Alt+Tab between panes without losing focus

#### Wayland/Sway Integration
- **Native Wayland**: Full compatibility with modern display server
- **Tiling Window Management**: Sway provides efficient window organization
- **Resource Efficiency**: Wayland's lower overhead compared to X11

### Development Commands Available

#### In Any Pane
- `make dev-server` - Start web server with auto-restart
- `make dev-test` - Auto-run tests on file changes
- `make test` - Run all tests (Rust + Playwright)
- `cargo build --release` - Production build
- `npx playwright test --headed` - Visual browser testing

#### Specific to Shell Pane
- Git operations: `git status`, `git commit`, `git push`
- Manual testing: `curl http://localhost:3000/health`
- Cache management: `make cache-bust`
- Clean builds: `make clean && make build`

## Architecture Integration

### File Change Detection
- **cargo-watch** monitors `src/` directory for Rust changes
- Automatic rebuild triggers on any `.rs` file modification
- Web server restarts automatically on successful build
- Clear screen output provides clean visual feedback

### Web Development Flow
```
File Edit → cargo-watch detects → Rebuild → Server restart → Test in browser
     ↑                                                           ↓
     └─────────────── Feedback loop via shell pane ──────────────┘
```

### Cross-Pane Communication
- **Build Status**: Watch pane shows compilation success/failure
- **Server Status**: Watch pane shows web server startup/shutdown
- **Testing**: Shell pane for browser automation and manual verification
- **AI Assistance**: Main pane for code analysis, debugging, feature implementation

## Tool Integration

### Core Development Stack
- **Zellij**: Terminal multiplexer with custom layouts
- **Sway**: Wayland compositor for window management
- **Cargo**: Rust build system and package manager
- **cargo-watch**: File watching and auto-rebuild
- **Playwright**: Browser automation and testing
- **Claude Code**: AI-powered development assistance

### Quality Assurance Tools
- **rustfmt**: Code formatting (via `make format`)
- **clippy**: Rust linting (via `make lint`) 
- **Playwright**: End-to-end browser testing
- **Custom Tests**: Cross-platform consistency testing

## Troubleshooting Common Issues

### Zellij Session Management
```bash
# Attach to existing session
zellij attach

# List sessions
zellij ls

# Kill session if needed
zellij kill-session
```

### Cargo-Watch Issues
```bash
# If watch gets stuck, restart from shell pane:
pkill cargo-watch
cargo-watch -c -x "run -- --web"
```

### Sway Window Management
- **Switch Panes**: `Alt + Arrow Keys` or `Alt + hjkl`
- **Focus Terminal**: Click on desired pane
- **Resize**: `Alt + Shift + Arrow Keys`

### Port Conflicts
```bash
# Check what's using port 3000
lsof -i :3000

# Kill conflicting processes
pkill -f "cargo.*web"
```

## Performance Characteristics

### Resource Usage
- **Zellij**: Minimal overhead terminal multiplexer
- **Sway**: Efficient Wayland compositor
- **cargo-watch**: Moderate CPU during builds, idle otherwise
- **Rust Debug Builds**: Fast compilation for development iteration

### Build Times
- **Development Builds**: ~2-5 seconds for incremental changes
- **Full Rebuilds**: ~10-30 seconds depending on changes
- **Release Builds**: ~1-2 minutes (used for production testing)

### Memory Footprint
- **Development Environment**: ~100-200MB for all tools
- **Web Server**: ~20-50MB during operation
- **Browser Tests**: ~200-500MB during Playwright execution

## Future Improvements

### Potential Enhancements
1. **Hot Reload**: Frontend asset hot-reloading for faster UI iteration
2. **Test Integration**: Auto-run Playwright tests on successful builds
3. **Notification System**: Desktop notifications for build success/failure
4. **Multi-Project**: Extend layout for multiple project development

### Known Limitations
1. **Manual Browser Refresh**: No hot-reload for frontend changes
2. **Single Server Instance**: Cannot run multiple server configurations simultaneously
3. **Build Dependencies**: Full rebuild required for grammar template changes

---

**Environment**: Arch Linux + Sway + Wayland + Zellij  
**Last Updated**: 2025-09-07  
**Layout Version**: music-text-dev.kdl v1.0