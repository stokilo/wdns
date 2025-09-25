#!/bin/bash

# Diagnose SOCKS5 proxy issues
# This script helps identify what's causing SOCKS5 connection problems

set -e

PROXY_HOST="127.0.0.1"
PROXY_PORT="9702"
TEST_URL="http://httpbin.org/ip"

echo "🔍 SOCKS5 Proxy Diagnostics"
echo "=============================="

# Check if service is running
echo "📡 Checking WDNS service status..."
if curl -s http://127.0.0.1:9700/health > /dev/null; then
    echo "✅ WDNS service is running"
else
    echo "❌ WDNS service is not running"
    exit 1
fi

# Check SOCKS5 configuration
echo "🔧 Checking SOCKS5 configuration..."
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

# Test basic connectivity
echo "🔌 Testing basic connectivity to SOCKS5 port..."
if nc -z "$PROXY_HOST" "$PROXY_PORT" 2>/dev/null; then
    echo "✅ Port $PROXY_PORT is open and accepting connections"
else
    echo "❌ Port $PROXY_PORT is not accessible"
    echo "   Check if the service is running and the port is correct"
    exit 1
fi

# Test with different SOCKS5 clients
echo "🧪 Testing SOCKS5 with different clients..."

# Test with curl
echo "  📡 Testing with curl..."
if curl --socks5 "$PROXY_HOST:$PROXY_PORT" --connect-timeout 10 --max-time 30 "$TEST_URL" > /dev/null 2>&1; then
    echo "  ✅ curl SOCKS5 test passed"
else
    echo "  ❌ curl SOCKS5 test failed"
    echo "     This might indicate a protocol issue"
fi

# Test with wget (if available)
if command -v wget >/dev/null 2>&1; then
    echo "  📡 Testing with wget..."
    if wget --quiet --proxy=on --proxy-type=socks5 --proxy-host="$PROXY_HOST" --proxy-port="$PROXY_PORT" --timeout=30 -O- "$TEST_URL" > /dev/null 2>&1; then
        echo "  ✅ wget SOCKS5 test passed"
    else
        echo "  ❌ wget SOCKS5 test failed"
    fi
else
    echo "  ⏭️  wget not available, skipping"
fi

# Test with netcat to see raw data
echo "🔍 Testing raw connection to SOCKS5 port..."
echo "  Sending SOCKS5 greeting manually..."

# Create a simple SOCKS5 greeting (version 5, 1 method, no auth)
echo -ne '\x05\x01\x00' | nc "$PROXY_HOST" "$PROXY_PORT" 2>/dev/null &
NC_PID=$!
sleep 2
kill $NC_PID 2>/dev/null || true

echo "  ✅ Raw connection test completed"

# Check for common issues
echo "🔍 Checking for common issues..."

# Check if port is being used by another service
echo "  📊 Checking port usage..."
if lsof -i :$PROXY_PORT 2>/dev/null | grep -q LISTEN; then
    echo "  ✅ Port $PROXY_PORT is properly bound"
else
    echo "  ❌ Port $PROXY_PORT is not bound"
fi

# Check firewall
echo "  🔥 Checking firewall status..."
if command -v ufw >/dev/null 2>&1; then
    if ufw status | grep -q "Status: active"; then
        echo "  ⚠️  UFW firewall is active - check if port $PROXY_PORT is allowed"
    else
        echo "  ✅ UFW firewall is not blocking"
    fi
else
    echo "  ⏭️  UFW not available, skipping firewall check"
fi

echo ""
echo "📋 Diagnostic Summary"
echo "===================="
echo "✅ Service Status: Running"
echo "✅ SOCKS5 Configuration: Enabled"
echo "✅ Port Accessibility: Open"
echo ""
echo "💡 If tests are failing, check the service logs for detailed error messages:"
echo "   tail -f /var/log/wdns-service.log"
echo ""
echo "🔧 Common fixes:"
echo "   1. Ensure the service is running: ./target/release/wdns-service"
echo "   2. Check config.json has socks5_enabled: true"
echo "   3. Verify the port is not blocked by firewall"
echo "   4. Check service logs for specific error messages"
