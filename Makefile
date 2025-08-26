# Notation Parser Makefile
# Ensures consistent builds and testing across CLI and WASM

.PHONY: all build test clean consistency-test help

# Default target
all: build test

# Build all targets
build: build-cli build-wasm

# Build CLI binary
build-cli:
	@echo "🔨 Building CLI binary..."
	cargo build --release --bin cli

# Build WASM module
build-wasm:
	@echo "📦 Building WASM module..."
	wasm-pack build --target web --out-dir web/pkg

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
	rm -rf web/pkg/
	rm -rf test_output/
	rm -rf temp_server_tests/
	rm -f *.ly *.outline *.yaml *.json *.clr server_test_*

# Start development server
dev-server:
	@echo "🚀 Starting development server..."
	npm start

# Run development workflow (build + start server)
dev: build dev-server

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

# Full CI pipeline (what GitHub Actions runs)
ci: clean build test lint

# Quick consistency check (for development)
quick-check: build-wasm
	@echo "⚡ Quick consistency check..."
	cargo test cross_platform_tests --release --quiet

# Help target
help:
	@echo "Available targets:"
	@echo "  all              - Build and test everything (default)"
	@echo "  build            - Build CLI and WASM"
	@echo "  build-cli        - Build CLI binary only"
	@echo "  build-wasm       - Build WASM module only"
	@echo "  test             - Run all tests"
	@echo "  test-rust        - Run Rust tests only"
	@echo "  test-consistency - Run cross-platform consistency tests"
	@echo "  clean            - Clean build artifacts"
	@echo "  dev-server       - Start development server"
	@echo "  dev              - Build and start development server"
	@echo "  install          - Install dependencies"
	@echo "  lint             - Run linting"
	@echo "  format           - Format code"
	@echo "  ci               - Full CI pipeline"
	@echo "  quick-check      - Quick consistency check"
	@echo "  help             - Show this help"