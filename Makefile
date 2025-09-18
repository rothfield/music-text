.PHONY: help build clean run web web-watch kill kill-watch kill-all restart test test-cli test-web logs dev mtx zellij-clean tui direct-test

# PERMANENT DEVELOPMENT MODE: Always use fastest debug build with warnings suppressed
# Philosophy: We are ALWAYS in dev mode for fastest iteration
RUST_BUILD_FLAGS := RUSTFLAGS="-A warnings"

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-15s %s\n", $$1, $$2}'

build: ## Build fast debug binary (permanent dev mode)
	$(RUST_BUILD_FLAGS) cargo build

build-quiet: ## Build quietly without output
	@$(RUST_BUILD_FLAGS) cargo build > /dev/null 2>&1

clean: ## Clean build artifacts and logs
	cargo clean
	rm -f development.log

run: build ## Start web server on port 3000
	./target/debug/music-text --web

web: build ## Start web server on port 3000 (alias for run)
	./target/debug/music-text --web

web-watch: ## Watch for changes and auto-restart web server
	@echo "🔄 Starting web-watch mode (robust, never dies)..."
	@echo "Watching src/ and public/ for changes"
	@echo "Uses same build flags as 'make build': RUSTFLAGS=\"-A warnings\""
	@echo "Press Ctrl+C to stop"
	@echo "🔧 Initial build and server start..."
	@if $(RUST_BUILD_FLAGS) cargo build 2>/dev/null; then \
		echo "✅ Initial build successful, starting server..."; \
		lsof -ti:3000 | xargs -r kill 2>/dev/null || true; \
		sleep 1; \
		./target/debug/music-text --web > .web-server.log 2>&1 & \
		echo $$! > .web-server.pid; \
		echo "🚀 Server started on http://localhost:3000 (PID: $$!)"; \
	else \
		echo "❌ Initial build failed, will retry on changes..."; \
	fi
	@touch .last-build
	@echo $$$$ > .web-watch.pid; \
	trap 'echo "🛑 Shutting down web-watch..."; lsof -ti:3000 | xargs -r kill 2>/dev/null || true; rm -f .web-watch.pid .web-server.pid .web-server.log; exit 0' INT TERM; \
	while true; do \
		{ \
			if find src public -type f \( -name "*.rs" -o -name "*.html" -o -name "*.js" \) -newer .last-build 2>/dev/null | grep -q .; then \
				echo "📝 Changes detected at $$(date '+%H:%M:%S'), rebuilding..."; \
				if $(RUST_BUILD_FLAGS) cargo build 2>/dev/null; then \
					echo "✅ Build successful, restarting server..."; \
					touch .last-build; \
					lsof -ti:3000 | xargs -r kill 2>/dev/null || true; \
					sleep 1; \
					./target/debug/music-text --web > .web-server.log 2>&1 & \
					echo $$! > .web-server.pid; \
					echo "🚀 Server restarted on http://localhost:3000 (PID: $$!)"; \
				else \
					echo "❌ Build failed at $$(date '+%H:%M:%S'), retrying in 5 seconds..."; \
					sleep 3; \
				fi; \
			fi; \
		} 2>/dev/null || { \
			echo "⚠️ Error occurred, continuing watch..."; \
		}; \
		sleep 2; \
	done


tui: build ## Start TUI REPL
	@./target/debug/music-text repl

perf: build ## Run parser performance benchmarks
	./target/debug/music-text perf

cli-test: build ## Test CLI with example input
	./target/debug/music-text --input "|1 2 3"

direct-test: build ## Test new direct parsing pipeline
	@echo "Testing direct parsing pipeline..."
	@echo "Old pipeline:"
	@echo "1 2 3 | 4 - 5 6 |" | ./target/debug/music-text --output debug | head -10
	@echo "\nNew direct pipeline:"
	@echo "1 2 3 | 4 - 5 6 |" | ./target/debug/music-text --direct --output debug | head -10

kill: ## Stop the web server
	@pkill -f "music-text --web" && echo "✓ Web server stopped" || echo "✗ No web server running"

kill-watch: ## Stop the web-watch process
	@if [ -f .web-watch.pid ]; then \
		WATCH_PID=$$(cat .web-watch.pid 2>/dev/null); \
		if [ -n "$$WATCH_PID" ] && kill -0 $$WATCH_PID 2>/dev/null; then \
			echo "🛑 Stopping web-watch process (PID: $$WATCH_PID)..."; \
			kill -TERM $$WATCH_PID 2>/dev/null || true; \
			sleep 1; \
			kill -KILL $$WATCH_PID 2>/dev/null || true; \
			rm -f .web-watch.pid .web-server.pid .web-server.log; \
			pkill -f "\\./target/debug/music-text --web" 2>/dev/null || true; \
			echo "✅ Web-watch stopped and cleaned up"; \
		else \
			echo "❌ Web-watch process not found or already dead"; \
			rm -f .web-watch.pid .web-server.pid .web-server.log; \
		fi; \
	else \
		echo "❌ No web-watch PID file found"; \
		pkill -f "make.*web-watch" 2>/dev/null && echo "🧹 Killed any remaining web-watch processes" || true; \
		pkill -f "\\./target/debug/music-text --web" 2>/dev/null && echo "🧹 Killed any remaining web servers" || true; \
	fi

kill-all: ## Stop both web server and web-watch
	@$(MAKE) kill-watch
	@$(MAKE) kill

restart: ## Restart the web server
	@$(MAKE) kill || true
	@sleep 1
	@$(MAKE) web

status: ## Check if web server is running
	@lsof -i :3000 > /dev/null 2>&1 && echo "✓ Web server is running on http://localhost:3000" || echo "✗ Web server is not running"

test: ## Run all tests
	cargo test

test-cli: build ## Test CLI with various inputs
	@echo "Testing parser output..."
	@./target/debug/music-text --input "|1 2 3" > /dev/null && echo "✓ parser works"
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

cli-spec: ## View CLI specification document
	@if command -v bat > /dev/null 2>&1; then \
		bat specs/cli-specification.md --language markdown; \
	elif command -v less > /dev/null 2>&1; then \
		less specs/cli-specification.md; \
	else \
		cat specs/cli-specification.md; \
	fi

# Quick development shortcuts
b: build
c: clean
w: web
ww: web-watch
k: kill
kw: kill-watch
ka: kill-all
r: restart
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
