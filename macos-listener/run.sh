#!/bin/bash

# macOS Network Connection Monitor - Run Script
# This script builds and runs the macOS Network Connection Monitor

set -e

echo "🔍 macOS Network Connection Monitor"
echo "=================================="

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

# Check if required system commands are available
if ! command -v netstat &> /dev/null; then
    echo "❌ netstat command not found. This is required for network monitoring."
    exit 1
fi

if ! command -v lsof &> /dev/null; then
    echo "❌ lsof command not found. This is required for process information."
    exit 1
fi

echo "✅ System requirements met"
echo ""

# Build the application
echo "🔨 Building application..."
if cargo build --release; then
    echo "✅ Build successful"
else
    echo "❌ Build failed"
    exit 1
fi

echo ""
echo "🚀 Starting macOS Network Connection Monitor..."
echo "   Press Ctrl+C to stop the application"
echo ""

# Run the application
cargo run --release
