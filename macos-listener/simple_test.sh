#!/bin/bash

echo "🧪 Simple Proxy Test"
echo "===================="

echo "🔍 Testing TCP proxy connection to 127.0.0.1:8080..."
echo "   If proxy is running, you should see connection logs in the app"
echo "   Press Ctrl+C to cancel after a few seconds"

# Test TCP connection to proxy
nc -v 127.0.0.1 8080 &
TCP_PID=$!

# Wait a bit then kill
sleep 3
kill $TCP_PID 2>/dev/null

echo ""
echo "🌐 Testing DNS proxy connection to 127.0.0.1:5353..."
echo "   If proxy is running, you should see DNS query logs in the app"

# Test DNS query to proxy
echo "test" | nc -u 127.0.0.1 5353 &
DNS_PID=$!

# Wait a bit then kill
sleep 3
kill $DNS_PID 2>/dev/null

echo ""
echo "✅ Test completed - check the application logs for proxy activity"
