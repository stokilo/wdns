#!/bin/bash

# Build script for macOS Network Connection Monitor

set -e

echo "🔨 Building macOS Network Connection Monitor"
echo "==========================================="

# Check if we're on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "❌ This application is designed for macOS only"
    exit 1
fi

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Clean previous build
echo "🧹 Cleaning previous build..."
cargo clean

# Build the application
echo "🔨 Building application..."
if cargo build --release; then
    echo "✅ Build successful"
    echo ""
    echo "🚀 To run the application:"
    echo "   cargo run"
    echo "   # or"
    echo "   ./target/release/macos-listener"
    echo ""
    echo "🧪 To test network monitoring:"
    echo "   ./test-network.sh"
else
    echo "❌ Build failed"
    exit 1
fi
