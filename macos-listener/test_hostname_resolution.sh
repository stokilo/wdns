#!/bin/bash

# Test script for hostname resolution testing
# This script demonstrates the different ways to test hostname resolution

set -e

echo "🧪 Testing Hostname Resolution"
echo "=============================="

# Test hostname
HOSTNAME="networkpartner-kion-dev.cprt.kion.cloud"
SOCKS5_PROXY="192.168.0.115:9702"
DNS_INTERCEPTOR="127.0.0.1:5353"

echo "Test hostname: $HOSTNAME"
echo "SOCKS5 proxy: $SOCKS5_PROXY"
echo "DNS interceptor: $DNS_INTERCEPTOR"
echo ""

# Test 1: Direct DNS resolution
echo "🔍 Test 1: Direct DNS Resolution"
echo "-------------------------------"
if nslookup "$HOSTNAME" >/dev/null 2>&1; then
    echo "✅ Direct DNS resolution: SUCCESS"
    nslookup "$HOSTNAME"
else
    echo "❌ Direct DNS resolution: FAILED"
fi
echo ""

# Test 2: SOCKS5 proxy resolution
echo "🔗 Test 2: SOCKS5 Proxy Resolution"
echo "----------------------------------"
if curl --socks5-hostname "$SOCKS5_PROXY" -v --connect-timeout 10 --max-time 30 "http://$HOSTNAME" >/dev/null 2>&1; then
    echo "✅ SOCKS5 proxy resolution: SUCCESS"
    echo "Testing with verbose output:"
    curl --socks5-hostname "$SOCKS5_PROXY" -v --connect-timeout 10 --max-time 30 "http://$HOSTNAME" 2>&1 | head -20
else
    echo "❌ SOCKS5 proxy resolution: FAILED"
    echo "Testing with verbose output:"
    curl --socks5-hostname "$SOCKS5_PROXY" -v --connect-timeout 10 --max-time 30 "http://$HOSTNAME" 2>&1 | head -20
fi
echo ""

# Test 3: DNS interceptor resolution
echo "🌐 Test 3: DNS Interceptor Resolution"
echo "------------------------------------"
if nslookup "$HOSTNAME" "$DNS_INTERCEPTOR" >/dev/null 2>&1; then
    echo "✅ DNS interceptor resolution: SUCCESS"
    nslookup "$HOSTNAME" "$DNS_INTERCEPTOR"
else
    echo "❌ DNS interceptor resolution: FAILED"
    echo "Testing with verbose output:"
    nslookup "$HOSTNAME" "$DNS_INTERCEPTOR" 2>&1 | head -10
fi
echo ""

# Test 4: HTTP via DNS interceptor
echo "🌐 Test 4: HTTP via DNS Interceptor"
echo "-----------------------------------"
if curl --dns-servers "$DNS_INTERCEPTOR" -v --connect-timeout 10 --max-time 30 "http://$HOSTNAME" >/dev/null 2>&1; then
    echo "✅ HTTP via DNS interceptor: SUCCESS"
    echo "Testing with verbose output:"
    curl --dns-servers "$DNS_INTERCEPTOR" -v --connect-timeout 10 --max-time 30 "http://$HOSTNAME" 2>&1 | head -20
else
    echo "❌ HTTP via DNS interceptor: FAILED"
    echo "Testing with verbose output:"
    curl --dns-servers "$DNS_INTERCEPTOR" -v --connect-timeout 10 --max-time 30 "http://$HOSTNAME" 2>&1 | head -20
fi
echo ""

# Test 5: Compare results
echo "📊 Test 5: Comparison Summary"
echo "----------------------------"
echo "Direct DNS:"
nslookup "$HOSTNAME" 2>&1 | grep -E "(Name:|Address:|Non-authoritative answer)" || echo "No results"

echo ""
echo "SOCKS5 Proxy:"
curl --socks5-hostname "$SOCKS5_PROXY" -v --connect-timeout 10 --max-time 30 "http://$HOSTNAME" 2>&1 | grep -E "(Trying|Connected|SOCKS5)" || echo "No results"

echo ""
echo "DNS Interceptor:"
nslookup "$HOSTNAME" "$DNS_INTERCEPTOR" 2>&1 | grep -E "(Name:|Address:|Non-authoritative answer)" || echo "No results"

echo ""
echo "🎉 Testing completed!"
echo ""
echo "💡 Usage Instructions:"
echo "1. Start the application: cargo run"
echo "2. Click 'Test Hostname Resolution' in the UI"
echo "3. Enter hostname: $HOSTNAME"
echo "4. Test different resolution methods"
echo "5. Compare results to see which method works"
echo ""
echo "🔧 Configuration:"
echo "- Configure SOCKS5 proxy: 192.168.0.115:9702"
echo "- Add routing rules for *.kion.cloud domains"
echo "- Start traffic interceptor to enable routing"
