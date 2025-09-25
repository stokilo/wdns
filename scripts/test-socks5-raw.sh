#!/bin/bash

# Test SOCKS5 with raw socket connection
# This helps identify protocol issues

set -e

PROXY_HOST="127.0.0.1"
PROXY_PORT="9702"

echo "🧪 Raw SOCKS5 Protocol Test"
echo "==========================="

# Test 1: Send proper SOCKS5 greeting
echo "📡 Test 1: Sending proper SOCKS5 greeting..."
echo -ne '\x05\x01\x00' | nc "$PROXY_HOST" "$PROXY_PORT" 2>/dev/null &
NC_PID=$!
sleep 1
kill $NC_PID 2>/dev/null || true
echo "✅ SOCKS5 greeting sent"

# Test 2: Send HTTP request to SOCKS5 port (should fail gracefully)
echo "📡 Test 2: Sending HTTP request to SOCKS5 port (should fail)..."
echo -e "GET / HTTP/1.1\r\nHost: example.com\r\n\r\n" | nc "$PROXY_HOST" "$PROXY_PORT" 2>/dev/null &
NC_PID=$!
sleep 1
kill $NC_PID 2>/dev/null || true
echo "✅ HTTP request sent (should be rejected)"

# Test 3: Send malformed data
echo "📡 Test 3: Sending malformed data..."
echo -ne '\xFF\x01\x00' | nc "$PROXY_HOST" "$PROXY_PORT" 2>/dev/null &
NC_PID=$!
sleep 1
kill $NC_PID 2>/dev/null || true
echo "✅ Malformed data sent (should be rejected)"

echo ""
echo "🔍 Check the service logs to see how it handled these tests:"
echo "   The service should log detailed information about each connection attempt"
echo "   Look for messages like:"
echo "   - 'Received X bytes from SOCKS5 client'"
echo "   - 'SOCKS5 greeting bytes: [...]'"
echo "   - 'Invalid SOCKS version: X'"
echo "   - 'Client sent HTTP request instead of SOCKS5'"
