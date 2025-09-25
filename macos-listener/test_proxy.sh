#!/bin/bash

echo "🧪 Testing Real Traffic Proxy"
echo "================================"

echo "📋 Current configuration:"
echo "   DNS Proxy Port: 5353"
echo "   TCP Proxy Port: 8080"
echo ""

echo "🔍 Testing TCP proxy connection..."
echo "   This will try to connect to the TCP proxy on port 8080"
echo "   If proxy is working, you should see connection logs in the app"

# Test TCP connection to proxy
timeout 5 nc -v 127.0.0.1 8080 2>&1 || echo "❌ Connection failed (expected - no data sent)"

echo ""
echo "🌐 Testing DNS proxy..."
echo "   This will try to send a DNS query to the DNS proxy on port 5353"
echo "   If proxy is working, you should see DNS query logs in the app"

# Test DNS query to proxy
echo "test query" | timeout 3 nc -u 127.0.0.1 5353 2>&1 || echo "❌ DNS query failed (expected - no proper DNS packet)"

echo ""
echo "📝 To test with real traffic:"
echo "   1. Configure your browser to use 127.0.0.1:8080 as HTTP proxy"
echo "   2. Configure your system to use 127.0.0.1:5353 as DNS server"
echo "   3. Visit websites that match your proxy rules"
echo "   4. Check the application logs for proxy activity"
echo ""
echo "⚠️  NOTE: This is a simplified implementation"
echo "   Real traffic interception requires system-level privileges"
