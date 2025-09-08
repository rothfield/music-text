#!/bin/bash
# Simple development setup without tmux
# For when you prefer separate terminal windows

echo "🚀 Simple development setup"
echo ""
echo "Run these commands in separate terminals:"
echo ""
echo "📍 Terminal 1 - Web Server (auto-restart):"
echo "  cargo watch -x 'run -- --web'"
echo ""
echo "📍 Terminal 2 - Tests (auto-run):"
echo "  cargo watch -x test"
echo ""
echo "📍 Terminal 3 - Browser Tests (manual):"
echo "  npx playwright test --headed"
echo ""
echo "📍 Manual commands:"
echo "  make cache-bust    # Update cache versions"
echo "  cargo build        # Manual build"
echo "  cargo clippy       # Linting"
echo ""

# Offer to start the web server watcher
read -p "🌐 Start web server watcher now? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🚀 Starting web server with auto-restart..."
    echo "💡 Edit any Rust file and save to see it restart automatically"
    cargo watch -x 'run -- --web'
fi