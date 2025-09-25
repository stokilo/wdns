#!/bin/bash

echo "🧪 Testing Real Traffic Proxy with HTTP requests"
echo "================================================"

echo "📋 Configuration:"
echo "   TCP Proxy: 127.0.0.1:8080"
echo "   DNS Proxy: 127.0.0.1:5353"
echo ""

echo "🔍 Testing HTTP connection through TCP proxy..."
echo "   This will send an HTTP request to the proxy"
echo "   If proxy is working, you should see connection logs in the app"

# Create a simple HTTP request
HTTP_REQUEST="GET / HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n"

echo "📤 Sending HTTP request through proxy..."
echo "$HTTP_REQUEST" | nc 127.0.0.1 8080 &
HTTP_PID=$!

# Wait a bit then kill
sleep 5
kill $HTTP_PID 2>/dev/null

echo ""
echo "🌐 Testing DNS query through DNS proxy..."
echo "   This will send a DNS query to the proxy"

# Create a simple DNS query (simplified)
DNS_QUERY="\x12\x34\x01\x00\x00\x01\x00\x00\x00\x00\x00\x00\x07example\x03com\x00\x00\x01\x00\x01"
echo -e "$DNS_QUERY" | nc -u 127.0.0.1 5353 &
DNS_PID=$!

# Wait a bit then kill
sleep 3
kill $DNS_PID 2>/dev/null

echo ""
echo "📝 To test with real browser traffic:"
echo "   1. Open browser settings"
echo "   2. Set HTTP proxy to 127.0.0.1:8080"
echo "   3. Set DNS to 127.0.0.1:5353"
echo "   4. Visit websites that match your proxy rules"
echo "   5. Check the application logs for proxy activity"
echo ""
echo "✅ Test completed - check the application logs for detailed proxy activity"
