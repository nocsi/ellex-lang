.PHONY: help setup build test clean dev docs install release fmt clippy check bench

help: ## Show this help message
	@echo "🌿 Ellex Language Development Commands"
	@echo "====================================="
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

setup: ## Set up the development environment
	@echo "🔧 Setting up development environment..."
	@./scripts/setup.sh

build: ## Build all components
	@echo "🔨 Building all components..."
	@cd crates && cargo build --release --bin el
	@cd elixir_backend && mix compile

test: ## Run all tests
	@echo "🧪 Running tests..."
	@cd crates && cargo test --all-features
	@cd elixir_backend && mix test

dev: ## Start development REPL
	@echo "🌿 Starting Ellex REPL..."
	@cd crates && cargo run --bin el repl

clean: ## Clean build artifacts
	@echo "🧹 Cleaning..."
	@cd crates && cargo clean
	@cd elixir_backend && mix clean

docs: ## Generate documentation
	@echo "📚 Generating documentation..."
	@cd crates && cargo doc --no-deps --open

install: build ## Install the el binary locally
	@echo "📦 Installing el binary..."
	@cd crates && cargo install --path ellex_cli --bin el --force

release: ## Build optimized release binary
	@echo "🚀 Building release binary..."
	@cd crates && cargo build --release --bin el
	@echo "✅ Release binary built: crates/target/release/el"

fmt: ## Format code
	@echo "🎨 Formatting code..."
	@cd crates && cargo fmt --all

clippy: ## Run clippy lints
	@echo "📎 Running clippy..."
	@cd crates && cargo clippy --all-targets --all-features -- -D warnings

check: ## Check code without building
	@echo "🔍 Checking code..."
	@cd crates && cargo check --all-features

bench: ## Run benchmarks
	@echo "⚡ Running benchmarks..."
	@cd crates && cargo bench --all-features
	@cd benchmarks && python3 scripts/direct_benchmark.py
