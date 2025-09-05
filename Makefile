# Music-Text Makefile
# Ensures consistent builds and testing across CLI and web server

.PHONY: all build test clean consistency-test help grammar

# Default target
all: build test

# Regenerate Pest grammar from templates
grammar:
	@echo "🎼 Regenerating Pest grammar from templates..."
	cargo run --bin generate-grammar
	@echo "✅ Grammar regenerated successfully"

# Build music-txt binary
build:
	@echo "🔨 Building music-txt binary..."
	cargo build --release --bin music-txt

# Run all tests
test: test-rust test-consistency

# Run Rust unit and integration tests
test-rust:
	@echo "🧪 Running Rust tests..."
	cargo test --release

# Run cross-platform consistency tests
test-consistency: build
	@echo "🔍 Running cross-platform consistency tests..."
	cargo test cross_platform_tests --release
	@if command -v curl &> /dev/null && curl -s -f "http://localhost:3000" >/dev/null 2>&1; then \
		./test_cross_platform_consistency.sh; \
	else \
		echo "ℹ️  Server not running, skipping API tests"; \
	fi

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	rm -rf test_output/
	rm -rf temp_server_tests/
	rm -f *.ly *.outline *.yaml *.json *.clr server_test_*


# Install dependencies
install:
	@echo "📥 Installing dependencies..."
	cargo fetch
	npm install

# Run linting and formatting
lint:
	@echo "🔧 Running linting..."
	cargo fmt --check
	cargo clippy -- -D warnings

# Format code
format:
	@echo "📝 Formatting code..."
	cargo fmt

# Update cache-busting versions for web assets
cache-bust:
	@echo "🔄 Updating cache-busting versions..."
	@cd webapp && ./cache-bust.sh

# Main development command - build first, then watch (most common)
dev: build dev-server

# Full tmux development environment
dev-tmux: build
	@echo "🚀 Starting tmux development environment..."
	@./dev.sh

# Simple development setup (no tmux)
dev-simple:
	@echo "📋 Simple development setup..."
	@./dev-simple.sh

# Watch and auto-restart web server
dev-server:
	@echo "🌐 Starting web server with auto-restart..."
	@echo "💡 Edit any Rust file to see it restart automatically"
	@cargo watch --quiet --postpone --watch src --watch Cargo.toml --watch grammar/notation.pest.template --watch grammar/systems.json --exec 'run --bin music-txt -- --web'

# Watch and auto-run tests  
dev-test:
	@echo "🧪 Starting test watcher..."
	@cargo watch --quiet --postpone --watch src --watch Cargo.toml --watch grammar/notation.pest.template --watch grammar/systems.json --exec test

# Quiet development - minimal output
dev-quiet:
	@echo "🤫 Starting quiet development server..."
	@cargo watch --quiet --postpone --watch src --watch Cargo.toml --watch grammar/notation.pest.template --watch grammar/systems.json --exec 'run --bin music-txt -- --web' 2>/dev/null

# Full CI pipeline (what GitHub Actions runs)
ci: clean build test lint

# Quick consistency check (for development)
quick-check: build
	@echo "⚡ Quick consistency check..."
	cargo test cross_platform_tests --release --quiet

# Help target
help:
	@echo "Available targets:"
	@echo "  all              - Build and test everything (default)"
	@echo "  grammar          - Regenerate Pest grammar from templates"
	@echo "  build            - Build music-txt binary"
	@echo "  test             - Run all tests"
	@echo "  test-rust        - Run Rust tests only"
	@echo "  test-consistency - Run cross-platform consistency tests"
	@echo "  clean            - Clean build artifacts"
	@echo ""
	@echo "Development:"
	@echo "  dev              - Build and start development server (most common)"
	@echo "  dev-server       - Start development server with auto-restart"
	@echo "  dev-tmux         - Full tmux development environment"
	@echo "  dev-simple       - Show manual development setup"
	@echo "  dev-test         - Run tests with auto-restart"
	@echo "  dev-quiet        - Quiet development server"
	@echo ""
	@echo "Utilities:"
	@echo "  install          - Install dependencies"
	@echo "  lint             - Run linting"
	@echo "  format           - Format code"
	@echo "  cache-bust       - Update web asset versions"
	@echo "  ci               - Full CI pipeline"
	@echo "  quick-check      - Quick consistency check"
	@echo "  help             - Show this help"