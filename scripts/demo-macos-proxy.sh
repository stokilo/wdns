#!/bin/bash

# WDNS macOS Proxy Demo Script
# This script demonstrates the proxy functionality

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}WDNS macOS Proxy Demo${NC}"
echo "======================"
echo ""

# Function to check if WDNS service is running
check_wdns_service() {
    echo -e "${YELLOW}Checking WDNS service...${NC}"
    
    if curl -s --connect-timeout 5 "http://127.0.0.1:9700/health" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ DNS service is running on port 9700${NC}"
    else
        echo -e "${RED}✗ DNS service is not running${NC}"
        echo "Please start the WDNS service first:"
        echo "  ./target/release/wdns-service"
        return 1
    fi
    
    if curl -s --connect-timeout 5 "http://127.0.0.1:9701" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Proxy server is running on port 9701${NC}"
    else
        echo -e "${RED}✗ Proxy server is not running${NC}"
        echo "Please start the WDNS service first:"
        echo "  ./target/release/wdns-service"
        return 1
    fi
}

# Function to demonstrate DNS resolution
demo_dns_resolution() {
    echo -e "${YELLOW}Demonstrating DNS resolution...${NC}"
    
    echo "Resolving hosts through WDNS service:"
    curl -s -X POST http://127.0.0.1:9700/api/dns/resolve \
        -H "Content-Type: application/json" \
        -d '{"hosts": ["google.com", "github.com", "stackoverflow.com"]}' | \
        python3 -m json.tool 2>/dev/null || echo "DNS resolution completed"
    echo ""
}

# Function to demonstrate proxy functionality
demo_proxy_functionality() {
    echo -e "${YELLOW}Demonstrating proxy functionality...${NC}"
    
    echo "Testing HTTP request through proxy:"
    echo "Your IP without proxy:"
    curl -s http://httpbin.org/ip | python3 -m json.tool 2>/dev/null || curl -s http://httpbin.org/ip
    echo ""
    
    echo "Your IP through proxy:"
    curl -s --proxy http://127.0.0.1:9701 http://httpbin.org/ip | python3 -m json.tool 2>/dev/null || curl -s --proxy http://127.0.0.1:9701 http://httpbin.org/ip
    echo ""
    
    echo "Testing HTTPS request through proxy:"
    curl -s --proxy http://127.0.0.1:9701 https://httpbin.org/ip | python3 -m json.tool 2>/dev/null || curl -s --proxy http://127.0.0.1:9701 https://httpbin.org/ip
    echo ""
}

# Function to demonstrate system proxy configuration
demo_system_proxy() {
    echo -e "${YELLOW}Demonstrating system proxy configuration...${NC}"
    
    echo "Current system proxy settings:"
    echo "HTTP Proxy:"
    networksetup -getwebproxy "Wi-Fi"
    echo ""
    echo "HTTPS Proxy:"
    networksetup -getsecurewebproxy "Wi-Fi"
    echo ""
}

# Function to demonstrate environment variables
demo_environment_variables() {
    echo -e "${YELLOW}Demonstrating environment variables...${NC}"
    
    echo "Setting proxy environment variables:"
    export HTTP_PROXY="http://127.0.0.1:9701"
    export HTTPS_PROXY="http://127.0.0.1:9701"
    export http_proxy="http://127.0.0.1:9701"
    export https_proxy="http://127.0.0.1:9701"
    
    echo "Environment variables set:"
    env | grep -i proxy
    echo ""
    
    echo "Testing with environment variables:"
    curl -s http://httpbin.org/ip | python3 -m json.tool 2>/dev/null || curl -s http://httpbin.org/ip
    echo ""
}

# Function to demonstrate browser configuration
demo_browser_configuration() {
    echo -e "${YELLOW}Demonstrating browser configuration...${NC}"
    
    echo "Browser proxy configuration options:"
    echo ""
    echo "Chrome:"
    echo "  open -a \"Google Chrome\" --args --proxy-server=\"http://127.0.0.1:9701\""
    echo ""
    echo "Firefox:"
    echo "  Create custom profile with proxy settings"
    echo "  /Applications/Firefox.app/Contents/MacOS/firefox -profile ~/.wdns-proxy-firefox"
    echo ""
    echo "Safari:"
    echo "  Uses system proxy settings (configured automatically)"
    echo ""
}

# Function to show next steps
show_next_steps() {
    echo -e "${GREEN}Demo completed!${NC}"
    echo ""
    echo "Next steps:"
    echo ""
    echo "1. Configure your system to use the proxy:"
    echo "   ./scripts/proxy on"
    echo ""
    echo "2. Test the proxy configuration:"
    echo "   ./scripts/proxy test"
    echo ""
    echo "3. Start applications with proxy:"
    echo "   ./scripts/proxy apps"
    echo ""
    echo "4. Check proxy status:"
    echo "   ./scripts/proxy status"
    echo ""
    echo "5. Disable proxy when done:"
    echo "   ./scripts/proxy off"
    echo ""
    echo "For detailed documentation:"
    echo "   cat scripts/README-macos-proxy.md"
}

# Main demo function
main() {
    echo "This demo will show you how the WDNS proxy works."
    echo "Make sure the WDNS service is running first."
    echo ""
    read -p "Press Enter to continue..."
    echo ""
    
    # Check if WDNS service is running
    if ! check_wdns_service; then
        echo -e "${RED}Please start the WDNS service first and try again.${NC}"
        exit 1
    fi
    
    echo ""
    
    # Demonstrate DNS resolution
    demo_dns_resolution
    
    # Demonstrate proxy functionality
    demo_proxy_functionality
    
    # Demonstrate system proxy configuration
    demo_system_proxy
    
    # Demonstrate environment variables
    demo_environment_variables
    
    # Demonstrate browser configuration
    demo_browser_configuration
    
    # Show next steps
    show_next_steps
}

# Run the demo
main
