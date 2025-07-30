#!/bin/bash
set -e

echo "🌿 Setting up Ellex Language Development Environment"
echo "=================================================="

# Make this script executable
chmod +x "$0"

# Check prerequisites
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

echo "Checking prerequisites..."

if ! command_exists rustc; then
    echo "❌ Rust not found. Please install from https://rustup.rs/"
    exit 1
fi

if ! command_exists elixir; then
    echo "❌ Elixir not found. Please install from https://elixir-lang.org/install.html"
    exit 1
fi

echo "✅ Prerequisites found"

# Build Rust components
echo "🦀 Building Rust components..."
cd crates && cargo build && cd ..

# Build Elixir backend
echo "💧 Building Elixir backend..."
cd elixir_backend && mix deps.get && mix compile && cd ..

echo "🎉 Ellex setup complete!"
echo ""
echo "Try these commands:"
echo "  cargo run --bin ellex_cli repl    # Start interactive mode"
echo "  cargo run --bin ellex_cli serve   # Start web playground"
echo ""
