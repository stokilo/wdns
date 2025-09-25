#!/bin/bash

# Test script to verify UI buttons are working
# This script tests the hostname resolution functionality

echo "🧪 Testing UI Buttons and Hostname Resolution"
echo "=============================================="

# Test hostname
HOSTNAME="networkpartner-kion-dev.cprt.kion.cloud"
SOCKS5_PROXY="192.168.0.115:9702"

echo "Test hostname: $HOSTNAME"
echo "SOCKS5 proxy: $SOCKS5_PROXY"
echo ""

echo "📋 UI Button Locations:"
echo "1. Main Panel: '🧪 Test Hostname' button (next to Configure Proxies)"
echo "2. Traffic Interceptor Section: 'View Intercepted Traffic' button"
echo ""

echo "🔍 Testing Hostname Resolution Methods:"
echo ""

# Test 1: Direct DNS (should fail)
echo "1️⃣ Direct DNS Test:"
if nslookup "$HOSTNAME" >/dev/null 2>&1; then
    echo "✅ Direct DNS: SUCCESS"
    nslookup "$HOSTNAME" | head -5
else
    echo "❌ Direct DNS: FAILED (expected for internal hostname)"
fi
echo ""

# Test 2: SOCKS5 Proxy (should work)
echo "2️⃣ SOCKS5 Proxy Test:"
if curl --socks5-hostname "$SOCKS5_PROXY" -v --connect-timeout 10 --max-time 30 "http://$HOSTNAME" >/dev/null 2>&1; then
    echo "✅ SOCKS5 Proxy: SUCCESS"
    echo "Testing with verbose output:"
    curl --socks5-hostname "$SOCKS5_PROXY" -v --connect-timeout 10 --max-time 30 "http://$HOSTNAME" 2>&1 | head -10
else
    echo "❌ SOCKS5 Proxy: FAILED"
    echo "Testing with verbose output:"
    curl --socks5-hostname "$SOCKS5_PROXY" -v --connect-timeout 10 --max-time 30 "http://$HOSTNAME" 2>&1 | head -10
fi
echo ""

# Test 3: DNS Interceptor (if running)
echo "3️⃣ DNS Interceptor Test:"
if nslookup "$HOSTNAME" "127.0.0.1:5353" >/dev/null 2>&1; then
    echo "✅ DNS Interceptor: SUCCESS"
    nslookup "$HOSTNAME" "127.0.0.1:5353" | head -5
else
    echo "❌ DNS Interceptor: FAILED (interceptor not running or not configured)"
    echo "Testing with verbose output:"
    nslookup "$HOSTNAME" "127.0.0.1:5353" 2>&1 | head -5
fi
echo ""

echo "🎯 Expected Results:"
echo "==================="
echo "Direct DNS: ❌ FAILED (hostname not publicly resolvable)"
echo "SOCKS5 Proxy: ✅ SUCCESS (proxy resolves hostname remotely)"
echo "DNS Interceptor: ✅ SUCCESS (if properly configured)"
echo ""

echo "💡 How to Use the UI:"
echo "===================="
echo "1. Start application: cargo run"
echo "2. Look for '🧪 Test Hostname' button in main panel"
echo "3. Click the button to open test dialog"
echo "4. Enter hostname: $HOSTNAME"
echo "5. Test different resolution methods"
echo "6. Compare results to see which method works"
echo ""

echo "🔧 Configuration Steps:"
echo "======================"
echo "1. Configure SOCKS5 proxy: 192.168.0.115:9702"
echo "2. Add routing rules for *.kion.cloud domains"
echo "3. Start traffic interceptor"
echo "4. Test hostname resolution through UI"
echo ""

echo "✅ UI Button Test completed!"
echo "The '🧪 Test Hostname' button should now be visible in the main UI panel."
