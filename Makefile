.PHONY: all build release test clean fmt lint doc install run help

# Default target
all: fmt lint test build

# Build targets
build:
	@echo "Building debug version..."
	@cargo build

release:
	@echo "Building release version..."
	@cargo build --release

# Testing
test:
	@echo "Running tests..."
	@cargo test --all-features

test-verbose:
	@echo "Running tests with output..."
	@cargo test --all-features -- --nocapture

# Code quality
fmt:
	@echo "Formatting code..."
	@cargo fmt --all

lint:
	@echo "Running clippy..."
	@cargo clippy --all-targets --all-features -- -D warnings

check:
	@echo "Checking code..."
	@cargo check --all-features

# Documentation
doc:
	@echo "Building documentation..."
	@cargo doc --all-features --no-deps --open

# Benchmarks
bench:
	@echo "Running benchmarks..."
	@cargo bench --all-features

# Coverage
coverage:
	@echo "Generating coverage report..."
	@cargo tarpaulin --out Html --output-dir coverage

# Clean
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean
	@rm -rf coverage/

# Development
watch:
	@echo "Starting development watch mode..."
	@cargo watch -x 'run'

watch-test:
	@echo "Starting test watch mode..."
	@cargo watch -x 'test'

# Installation
install:
	@echo "Installing RustIRC..."
	@cargo install --path .

# Run
run:
	@echo "Running RustIRC..."
	@cargo run

run-release:
	@echo "Running RustIRC (release)..."
	@cargo run --release

# Security
audit:
	@echo "Running security audit..."
	@cargo audit

# Dependencies
update:
	@echo "Updating dependencies..."
	@cargo update

# Help
help:
	@echo "RustIRC Development Commands:"
	@echo "  make all         - Format, lint, test, and build"
	@echo "  make build       - Build debug version"
	@echo "  make release     - Build release version"
	@echo "  make test        - Run tests"
	@echo "  make fmt         - Format code"
	@echo "  make lint        - Run clippy linter"
	@echo "  make doc         - Build and open documentation"
	@echo "  make bench       - Run benchmarks"
	@echo "  make coverage    - Generate coverage report"
	@echo "  make clean       - Clean build artifacts"
	@echo "  make watch       - Watch mode for development"
	@echo "  make install     - Install RustIRC locally"
	@echo "  make run         - Run RustIRC"
	@echo "  make audit       - Run security audit"
	@echo "  make help        - Show this help message"