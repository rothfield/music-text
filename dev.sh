#!/bin/bash
# Development environment setup script
# Creates a tmux session with multiple panes for efficient development

echo "🚀 Setting up development environment..."

# Check if tmux is available
if ! command -v tmux &> /dev/null; then
    echo "❌ tmux is not installed. Please install tmux first:"
    echo "   sudo apt install tmux  # Ubuntu/Debian"
    echo "   brew install tmux      # macOS"
    exit 1
fi

# Kill existing dev session if it exists
tmux kill-session -t dev 2>/dev/null || true

# Create new tmux session
echo "📱 Creating tmux session 'dev'..."
tmux new-session -d -s dev -n main

# Pane 0: Web Server with auto-restart
echo "🌐 Setting up web server pane..."
tmux send-keys -t dev:main "echo '🌐 Web Server (auto-restart)'" Enter
tmux send-keys -t dev:main "cargo watch --quiet --postpone --watch src --watch Cargo.toml --watch grammar/notation.pest.template --watch grammar/systems.json --exec 'run --bin music-txt -- --web'" Enter

# Split horizontally for tests
tmux split-window -h -t dev:main

# Pane 1: Test watcher  
echo "🧪 Setting up test watcher pane..."
tmux send-keys -t dev:main.1 "echo '🧪 Test Watcher'" Enter
tmux send-keys -t dev:main.1 "cargo watch --quiet --postpone --watch src --watch Cargo.toml --watch grammar/notation.pest.template --watch grammar/systems.json --exec test" Enter

# Split the right pane vertically for browser tests
tmux split-window -v -t dev:main.1

# Pane 2: Browser tests
echo "🌍 Setting up browser test pane..."
tmux send-keys -t dev:main.2 "echo '🌍 Browser Tests'" Enter
tmux send-keys -t dev:main.2 "echo 'Run: npx playwright test --headed'" Enter
tmux send-keys -t dev:main.2 "echo 'Or: npx playwright test --debug'" Enter

# Create a new window for manual commands
tmux new-window -t dev -n cmd
tmux send-keys -t dev:cmd "echo '💻 Manual Commands Window'" Enter
tmux send-keys -t dev:cmd "echo 'Available commands:'" Enter
tmux send-keys -t dev:cmd "echo '  make cache-bust  - Update asset versions'" Enter  
tmux send-keys -t dev:cmd "echo '  cargo build      - Build project'" Enter
tmux send-keys -t dev:cmd "echo '  cargo test       - Run tests'" Enter

# Select the main window and adjust pane sizes
tmux select-window -t dev:main
tmux resize-pane -t dev:main.0 -x 60  # Make server pane wider

# Attach to the session
echo "✅ Development environment ready!"
echo ""
echo "📋 Layout:"
echo "  🌐 Left pane:    Web server (auto-restart on file changes)"
echo "  🧪 Top right:    Tests (auto-run on file changes)" 
echo "  🌍 Bottom right: Browser tests (manual)"
echo "  💻 'cmd' tab:    Manual commands"
echo ""
echo "🔧 Usage:"
echo "  • Edit files → server & tests auto-restart"
echo "  • Ctrl+B, C → new window"
echo "  • Ctrl+B, arrow keys → switch panes"
echo "  • Ctrl+B, D → detach (keeps running)"
echo "  • tmux attach -t dev → reattach later"
echo ""
echo "🚀 Attaching to tmux session..."

tmux attach -t dev