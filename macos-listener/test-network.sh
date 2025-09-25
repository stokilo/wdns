#!/bin/bash

# Test script for macOS Network Connection Monitor
# This script creates some network activity to test the monitor

echo "🧪 Testing macOS Network Connection Monitor"
echo "=========================================="

# Check if the application is running
if pgrep -f "macos-listener" > /dev/null; then
    echo "✅ macOS Listener is running"
else
    echo "❌ macOS Listener is not running. Please start it first:"
    echo "   cd macos-listener && cargo run"
    exit 1
fi

echo ""
echo "🌐 Creating test network activity..."

# Create some network connections to monitor
echo "📡 Making HTTP requests..."
curl -s httpbin.org/ip > /dev/null &
curl -s google.com > /dev/null &
curl -s github.com > /dev/null &

# Start a simple HTTP server
echo "🚀 Starting test HTTP server on port 8080..."
python3 -m http.server 8080 > /dev/null 2>&1 &
SERVER_PID=$!

# Wait a moment for connections to establish
sleep 2

echo "✅ Test network activity created:"
echo "   - HTTP requests to external sites"
echo "   - Local HTTP server on port 8080"
echo ""
echo "🔍 Check the macOS Listener application to see these connections!"
echo ""
echo "Press Enter to clean up test connections..."
read

# Clean up
echo "🧹 Cleaning up..."
kill $SERVER_PID 2>/dev/null
pkill -f "curl.*httpbin.org" 2>/dev/null
pkill -f "curl.*google.com" 2>/dev/null
pkill -f "curl.*github.com" 2>/dev/null

echo "✅ Test completed!"
