#!/bin/bash

echo "🧪 Simple HTTP Test"
echo "==================="

echo "🔍 Testing TCP proxy connection..."
echo "   Sending HTTP request to 127.0.0.1:8080"
echo "   Press Ctrl+C to cancel after seeing logs"

# Send HTTP request to proxy
echo "GET / HTTP/1.1
Host: example.com
Connection: close

" | nc 127.0.0.1 8080

echo ""
echo "✅ HTTP test completed"
