# Tech Note: Zellij + Cargo-Watch Development Environment

## Overview

Integration patterns for using Zellij terminal multiplexer with cargo-watch for automated Rust builds in the music-text project.

## Zellij vs TMux for Rust Development

### Zellij Advantages
- **Modern UI**: Better visual pane management and status display
- **Rust-native**: Built in Rust, good ecosystem fit
- **Floating panes**: Can overlay terminal output when needed  
- **Built-in layouts**: Easy to define and switch development layouts
- **Session persistence**: Survives network disconnections

### TMux Advantages  
- **Mature ecosystem**: More plugins, integrations, documentation
- **Universal availability**: Pre-installed on most systems
- **Shell integration**: Better integration with various shells
- **Performance**: Lower resource usage for simple setups

## Cargo-Watch Integration Patterns

### Pattern 1: Dedicated Build Pane
```bash
# Layout: Editor | Build Output
# Pane 1: Editor/Claude development  
# Pane 2: cargo watch -x "build --features gui"
```

**Pros:**
- Immediate feedback on compile errors
- Continuous validation during development
- Clear separation of concerns

**Cons:**  
- Noisy output during rapid edits
- May interfere with RAM disk space during parallel builds

### Pattern 2: On-Demand Building
```bash
# Layout: Editor | Manual Commands
# Pane 1: Editor/Claude development
# Pane 2: Manual `make build`, `make test` commands
```

**Pros:**
- Developer controls when builds happen
- Cleaner output, easier to read errors
- Better for large refactoring sessions

**Cons:**
- Easy to forget to build after changes
- Slower feedback cycle

### Pattern 3: Hybrid Approach
```bash
# Start: cargo-watch enabled
# During major refactoring: Ctrl+C to pause
# Resume: cargo watch for quick iteration
```

## Claude Code Integration

### Current Limitations
- **No real-time pane monitoring**: Claude cannot "watch" live terminal output
- **Snapshot-only access**: Can dump pane contents with `zellij action dump-screen`
- **File-based coordination**: Claude works through file system, not terminal

### Workflow Recommendations

#### For Claude-led Development:
1. **Pause cargo-watch** during automated refactoring
2. **Let Claude run explicit builds** (`make build`)
3. **Resume cargo-watch** for manual editing

#### For Manual Development:
1. **Keep cargo-watch running** continuously  
2. **Monitor build pane** for immediate feedback
3. **Use Claude for complex changes** with cargo-watch paused

## Recommended Zellij Layout

```kdl
layout {
    pane_template name="build_pane" {
        command "cargo"
        args "watch" "-x" "build --features gui"
        start_suspended true
    }
    
    tab name="dev" {
        pane size=70 {
            // Main development pane
        }
        pane size=30 {
            build_pane
        }
    }
    
    tab name="test" {  
        pane {
            command "cargo"
            args "watch" "-x" "test"
        }
    }
    
    tab name="web" {
        pane {
            command "make" 
            args "web"
        }
    }
}
```

## RAM Disk Considerations

### With 4GB RAM Disk:
- **Single cargo-watch**: Usually fine (2.5GB peak usage)
- **Multiple watchers**: Risk of space exhaustion
- **Parallel builds**: Can spike to 3-4GB temporarily

### Monitoring Commands:
```bash
# Watch RAM disk usage during builds
watch -n 2 'df -h /mnt/rust_ramdisk'

# Monitor active cargo processes  
ps aux | grep cargo

# Check Zellij session status
zellij list-sessions
```

## Best Practices

1. **Use `start_suspended true`** in layouts to control when watchers start
2. **Monitor RAM disk usage** during development
3. **Pause watchers during large refactoring** to prevent resource contention
4. **Use dedicated panes** for different types of builds (debug/test/release)
5. **Configure shell aliases** for common watch patterns:
   ```bash
   alias watch-build='cargo watch -x "build --features gui"'
   alias watch-test='cargo watch -x test'
   alias watch-check='cargo watch -x check'
   ```

## Troubleshooting

### Cargo-Watch Not Responding
- Check if multiple instances are running: `ps aux | grep cargo`
- Restart with fresh terminal: `Ctrl+C` then restart
- Clear target directory if corrupted: `cargo clean`

### Zellij Pane Issues
- Dump pane contents: `zellij action dump-screen`
- Kill stuck panes: `zellij action close-pane`
- Reload layout: `zellij action new-tab --layout <name>`

### RAM Disk Full
- Check usage: `df -h /mnt/rust_ramdisk`
- Clean builds: `cargo clean`
- Kill parallel watchers to reduce load

## Conclusion

Zellij + cargo-watch provides excellent development experience for music-text, with the caveat that Claude integration requires manual coordination. The hybrid approach (pause during automation, resume for iteration) works best for mixed human/AI development workflows.