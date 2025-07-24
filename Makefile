.PHONY: help install dev build test test-unit test-unit-watch test-rust test-property test-property-intensive check lint format clean setup-tools preview security-audit test-coverage build-analyze deploy-dry-run ci-check clippy fmt fmt-check

# Default target - show help
help:
	@echo "Available commands:"
	@echo ""
	@echo "Setup & Installation:"
	@echo "  make setup-tools  - Install global tools (wasm-pack)"
	@echo "  make install      - Install project dependencies"
	@echo ""
	@echo "Development:"
	@echo "  make dev          - Start development server"
	@echo "  make build        - Build production static site"
	@echo "  make preview      - Preview production build"
	@echo ""
	@echo "Testing & Quality:"
	@echo "  make test         - Run all tests (frontend + Rust)"
	@echo "  make test-unit    - Run frontend unit tests only"
	@echo "  make test-unit-watch - Run frontend unit tests in watch mode"
	@echo "  make test-rust    - Run Rust/WASM unit tests"
	@echo "  make test-property - Run Rust property/fuzz tests (100 cases each)"
	@echo "  make test-property-intensive - Run intensive property tests (1000+ cases each)"
	@echo "  make test-coverage - Run tests with coverage report"
	@echo "  make check        - Type check the code"
	@echo "  make lint         - Run linter checks"
	@echo "  make format       - Auto-format code"
	@echo "  make clippy       - Run Rust clippy linter"
	@echo "  make fmt          - Format Rust code"
	@echo "  make fmt-check    - Check Rust formatting"
	@echo "  make security-audit - Check for security vulnerabilities"
	@echo ""
	@echo "Analysis & CI:"
	@echo "  make build-analyze - Analyze bundle size"
	@echo "  make ci-check     - Run all CI checks locally"
	@echo "  make deploy-dry-run - Test deployment without publishing"
	@echo ""
	@echo "Maintenance:"
	@echo "  make clean        - Clean build artifacts"

# Install global tools
setup-tools:
	@echo "Installing global tools..."
	npm install -g wasm-pack

# Install dependencies
install:
	@echo "Installing dependencies..."
	cd frontend/svelte && npm install

# Start development server
dev:
	@echo "Starting development server..."
	cd frontend/svelte && npm run dev

# Build production site
build:
	@echo "Building production site..."
	cd frontend/svelte && npm run build

# Preview production build
preview:
	@echo "Starting preview server..."
	cd frontend/svelte && npm run preview

# Run all tests (frontend + Rust)
test: test-rust
	@echo "Running all frontend tests..."
	cd frontend/svelte && npm test

# Run frontend unit tests only
test-unit:
	@echo "Running frontend unit tests..."
	cd frontend/svelte && npm run test:unit

# Run frontend unit tests in watch mode
test-unit-watch:
	@echo "Running frontend unit tests in watch mode..."
	cd frontend/svelte && npm run test:unit:watch

# Run Rust/WASM unit tests
test-rust:
	@echo "Running Rust unit tests..."
	cd frontend/rust-wasm && cargo test

# Run Rust property/fuzz tests (default 100 cases each)
test-property:
	@echo "Running Rust property-based tests..."
	cd frontend/rust-wasm && cargo test property_tests --release

# Run intensive property tests (1000+ cases each)
test-property-intensive:
	@echo "Running intensive property-based tests (1000+ cases each)..."
	cd frontend/rust-wasm && PROPTEST_CASES=1000 cargo test property_tests --release

# Run tests with coverage report
test-coverage:
	@echo "Running tests with coverage..."
	cd frontend/svelte && npm run test:unit -- --coverage

# Type check
check:
	@echo "Running type check..."
	cd frontend/svelte && npm run check

# Run linter
lint:
	@echo "Running linter..."
	cd frontend/svelte && npm run lint

# Format code
format:
	@echo "Formatting code..."
	cd frontend/svelte && npm run format
	@echo "Formatting Rust code..."
	cd frontend/rust-wasm && cargo fmt

# Security audit
security-audit:
	@echo "Running security audit..."
	cd frontend/svelte && npm audit
	@echo "Running Rust security audit..."
	cd frontend/rust-wasm && cargo audit || echo "Install cargo-audit with: cargo install cargo-audit"

# Analyze bundle size
build-analyze:
	@echo "Building and analyzing bundle..."
	cd frontend/svelte && npm run build
	@echo "Bundle analysis complete. Check build output above."

# CI checks - run all quality checks
ci-check: lint check test test-property
	@echo "All CI checks completed successfully!"

# Dry run deployment
deploy-dry-run: build
	@echo "Performing deployment dry run..."
	cd frontend/svelte && echo "Built files ready for deployment in ./build directory"
	@ls -la frontend/svelte/build/

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	rm -rf frontend/svelte/build
	rm -rf frontend/svelte/.svelte-kit
	rm -rf frontend/svelte/node_modules/.vite
	rm -rf frontend/rust-wasm/target
	rm -rf frontend/rust-wasm/pkg

# Run Rust clippy linter
clippy:
	@echo "Running Rust clippy..."
	cd frontend/rust-wasm && cargo clippy --all-targets --all-features -- -D warnings

# Format Rust code
fmt:
	@echo "Formatting Rust code..."
	cd frontend/rust-wasm && cargo fmt

# Check Rust formatting
fmt-check:
	@echo "Checking Rust formatting..."
	cd frontend/rust-wasm && cargo fmt --check

# Clean everything including node_modules
clean-all: clean
	@echo "Cleaning all dependencies..."
	rm -rf frontend/svelte/node_modules
	rm -rf node_modules

# Watch for changes and rebuild WASM
watch-wasm:
	@echo "Watching Rust files for changes..."
	cd frontend/rust-wasm && cargo watch -x "build --target wasm32-unknown-unknown"

# Development with WASM watching
dev-with-wasm: 
	@echo "Starting development with WASM watching..."
	@echo "Note: Run 'make watch-wasm' in another terminal for automatic WASM rebuilds"
	cd frontend/svelte && npm run dev