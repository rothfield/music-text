.PHONY: help build clean run web test test-cli test-web logs dev mtx zellij-clean

# PERMANENT DEVELOPMENT MODE: Always use fastest debug build with warnings suppressed
# Philosophy: We are ALWAYS in dev mode for fastest iteration
RUST_BUILD_FLAGS := RUSTFLAGS="-A warnings"

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-15s %s\n", $$1, $$2}'

build: ## Build fast debug binary with all features (permanent dev mode)
	$(RUST_BUILD_FLAGS) cargo build --features gui

clean: ## Clean build artifacts and logs
	cargo clean
	rm -f development.log

run: build ## Start web server on port 3000
	./target/debug/music-text --web

web: build ## Start web server on port 3000 (alias for run)
	./target/debug/music-text --web

gui: build ## Launch GUI editor
	./target/debug/music-text gui

repl: build ## Start interactive REPL for musical notation
	./target/debug/music-text repl

perf: build ## Run parser performance benchmarks
	./target/debug/music-text perf

cli-test: build ## Test CLI with example input
	./target/debug/music-text pest "|1 2 3"

kill: ## Stop the web server
	@pkill -f "music-text --web" && echo "✓ Web server stopped" || echo "✗ No web server running"

status: ## Check if web server is running
	@lsof -i :3000 > /dev/null 2>&1 && echo "✓ Web server is running on http://localhost:3000" || echo "✗ Web server is not running"

test: ## Run all tests
	cargo test

test-cli: build ## Test CLI with various inputs
	@echo "Testing pest output..."
	@./target/debug/music-text pest "|1 2 3" > /dev/null && echo "✓ pest works"
	@echo "Testing document output..."
	@./target/debug/music-text document "|S R G M|" > /dev/null && echo "✓ document works"
	@echo "Testing lilypond output..."
	@./target/debug/music-text full-lily "|1-2-3|" > /dev/null && echo "✓ lilypond works"

test-web: ## Run Playwright browser tests
	npx playwright test

test-web-headed: ## Run Playwright tests with visible browser
	npx playwright test --headed

logs: ## Tail the development log
	tail -f development.log

dev: build ## Start development server and watch logs in split terminal
	@echo "Starting server on http://localhost:3000"
	@echo "Press Ctrl+C to stop"
	@./target/debug/music-text --web

fresh: clean build ## Clean and rebuild everything
	@echo "Fresh build complete!"

install-test-deps: ## Install Playwright for browser testing
	npm init -y
	npm install --save-dev @playwright/test
	npx playwright install

install-completions: build ## Install shell completions for fish
	./target/debug/music-text completions fish > ~/.config/fish/completions/music-text.fish
	@echo "✓ Fish completions installed"

# Quick development shortcuts
b: build
c: clean
w: web
k: kill
t: test
l: logs

# --- AI Development ---
.PHONY: claude-cli
claude-cli: ## Run the Claude CLI for AI-driven development
	@echo "Starting Claude CLI..."
	claude --dangerously-skip-permissions

mtx: ## Start Zellij with music-text development layout
	zellij --layout music-text-dev.kdl

zellij-clean: ## Clean up old exited Zellij sessions
	@echo "Cleaning up old Zellij sessions..."
	@zellij list-sessions | grep EXITED | cut -d' ' -f1 | xargs -r -I{} zellij delete-session {} && echo "✓ Old sessions cleaned" || echo "✓ No old sessions to clean"
