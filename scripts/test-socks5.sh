#!/bin/bash

# Test SOCKS5 proxy functionality
# This script tests the SOCKS5 proxy server

set -e

PROXY_HOST="127.0.0.1"
PROXY_PORT="9702"
TEST_URL="http://httpbin.org/ip"

echo "🧦 Testing SOCKS5 Proxy Server"
echo "================================"

# Check if service is running
echo "📡 Checking if WDNS service is running..."
if ! curl -s http://127.0.0.1:9700/health > /dev/null; then
    echo "❌ WDNS service is not running. Please start it first:"
    echo "   ./target/release/wdns-service"
    exit 1
fi

echo "✅ WDNS service is running"

# Check if SOCKS5 proxy is enabled
echo "🔍 Checking SOCKS5 proxy configuration..."
response=$(curl -s http://127.0.0.1:9700/)
socks5_enabled=$(echo "$response" | jq -r '.socks5_enabled // false')
socks5_port=$(echo "$response" | jq -r '.socks5_port // null')

if [ "$socks5_enabled" != "true" ]; then
    echo "❌ SOCKS5 proxy is not enabled"
    echo "   Enable it in config.json:"
    echo '   "socks5_enabled": true'
    exit 1
fi

echo "✅ SOCKS5 proxy is enabled on port $socks5_port"

# Test SOCKS5 connection
echo "🧪 Testing SOCKS5 connection..."
if curl --socks5 "$PROXY_HOST:$PROXY_PORT" --connect-timeout 10 "$TEST_URL" > /dev/null 2>&1; then
    echo "✅ SOCKS5 proxy connection successful"
else
    echo "❌ SOCKS5 proxy connection failed"
    echo "   Make sure the SOCKS5 server is running on $PROXY_HOST:$PROXY_PORT"
    exit 1
fi

# Test with different tools
echo "🔧 Testing with different tools..."

# Test with curl
echo "  📡 Testing with curl..."
curl_result=$(curl --socks5 "$PROXY_HOST:$PROXY_PORT" -s "$TEST_URL" | jq -r '.origin' 2>/dev/null || echo "failed")
if [ "$curl_result" != "failed" ] && [ -n "$curl_result" ]; then
    echo "  ✅ curl test passed - IP: $curl_result"
else
    echo "  ❌ curl test failed"
fi

# Test with wget (if available)
if command -v wget >/dev/null 2>&1; then
    echo "  📡 Testing with wget..."
    if wget --quiet --proxy=on --proxy-type=socks5 --proxy-host="$PROXY_HOST" --proxy-port="$PROXY_PORT" -O- "$TEST_URL" > /dev/null 2>&1; then
        echo "  ✅ wget test passed"
    else
        echo "  ❌ wget test failed"
    fi
else
    echo "  ⏭️  wget not available, skipping"
fi

echo ""
echo "🎉 SOCKS5 Proxy Test Complete!"
echo "================================"
echo "✅ SOCKS5 proxy is working correctly"
echo "📡 Proxy endpoint: socks5://$PROXY_HOST:$PROXY_PORT"
echo ""
echo "💡 Usage examples:"
echo "   curl --socks5 $PROXY_HOST:$PROXY_PORT http://example.com"
echo "   export SOCKS5_PROXY=socks5://$PROXY_HOST:$PROXY_PORT"
echo "   export ALL_PROXY=socks5://$PROXY_HOST:$PROXY_PORT"
