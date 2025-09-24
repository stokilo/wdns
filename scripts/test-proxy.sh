#!/bin/bash

# Test script for WDNS Proxy Server
# This script tests the proxy server functionality

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default proxy server address
PROXY_SERVER="${WDNS_SERVER:-127.0.0.1:9701}"
DNS_SERVER="${WDNS_SERVER:-127.0.0.1:9700}"

echo -e "${BLUE}WDNS Proxy Server Test${NC}"
echo "================================"
echo "Proxy Server: $PROXY_SERVER"
echo "DNS Server: $DNS_SERVER"
echo ""

# Function to test if service is running
test_service_running() {
    local service_name=$1
    local port=$2
    
    echo -e "${YELLOW}Testing $service_name on port $port...${NC}"
    
    if curl -s --connect-timeout 5 "http://$port" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ $service_name is running on port $port${NC}"
        return 0
    else
        echo -e "${RED}✗ $service_name is not running on port $port${NC}"
        return 1
    fi
}

# Function to test proxy functionality
test_proxy_http() {
    echo -e "${YELLOW}Testing HTTP proxy functionality...${NC}"
    
    local test_url="http://httpbin.org/ip"
    local proxy_url="http://$PROXY_SERVER"
    
    echo "Testing: $test_url through proxy: $proxy_url"
    
    if curl -s --proxy "$proxy_url" --connect-timeout 10 "$test_url" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ HTTP proxy is working${NC}"
        return 0
    else
        echo -e "${RED}✗ HTTP proxy test failed${NC}"
        return 1
    fi
}

# Function to test proxy with HTTPS
test_proxy_https() {
    echo -e "${YELLOW}Testing HTTPS proxy functionality...${NC}"
    
    local test_url="https://httpbin.org/ip"
    local proxy_url="http://$PROXY_SERVER"
    
    echo "Testing: $test_url through proxy: $proxy_url"
    
    if curl -s --proxy "$proxy_url" --connect-timeout 10 "$test_url" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ HTTPS proxy is working${NC}"
        return 0
    else
        echo -e "${RED}✗ HTTPS proxy test failed${NC}"
        return 1
    fi
}

# Function to test proxy with environment variables
test_proxy_env() {
    echo -e "${YELLOW}Testing proxy with environment variables...${NC}"
    
    local test_url="http://httpbin.org/ip"
    
    echo "Setting HTTP_PROXY and HTTPS_PROXY environment variables"
    export HTTP_PROXY="http://$PROXY_SERVER"
    export HTTPS_PROXY="http://$PROXY_SERVER"
    
    if curl -s --connect-timeout 10 "$test_url" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Proxy with environment variables is working${NC}"
        unset HTTP_PROXY HTTPS_PROXY
        return 0
    else
        echo -e "${RED}✗ Proxy with environment variables test failed${NC}"
        unset HTTP_PROXY HTTPS_PROXY
        return 1
    fi
}

# Function to test proxy response
test_proxy_response() {
    echo -e "${YELLOW}Testing proxy response content...${NC}"
    
    local test_url="http://httpbin.org/ip"
    local proxy_url="http://$PROXY_SERVER"
    
    echo "Getting response from: $test_url through proxy: $proxy_url"
    
    local response=$(curl -s --proxy "$proxy_url" --connect-timeout 10 "$test_url" 2>/dev/null)
    
    if [ $? -eq 0 ] && [ -n "$response" ]; then
        echo -e "${GREEN}✓ Proxy response received:${NC}"
        echo "$response" | jq . 2>/dev/null || echo "$response"
        return 0
    else
        echo -e "${RED}✗ Proxy response test failed${NC}"
        return 1
    fi
}

# Function to test proxy with different hosts
test_proxy_hosts() {
    echo -e "${YELLOW}Testing proxy with different hosts...${NC}"
    
    local hosts=("httpbin.org" "google.com" "github.com")
    local proxy_url="http://$PROXY_SERVER"
    
    for host in "${hosts[@]}"; do
        echo "Testing: $host through proxy"
        
        if curl -s --proxy "$proxy_url" --connect-timeout 5 "http://$host" > /dev/null 2>&1; then
            echo -e "${GREEN}✓ $host is accessible through proxy${NC}"
        else
            echo -e "${RED}✗ $host is not accessible through proxy${NC}"
        fi
    done
}

# Function to test proxy performance
test_proxy_performance() {
    echo -e "${YELLOW}Testing proxy performance...${NC}"
    
    local test_url="http://httpbin.org/ip"
    local proxy_url="http://$PROXY_SERVER"
    local num_requests=5
    
    echo "Making $num_requests concurrent requests through proxy"
    
    local start_time=$(date +%s.%N)
    
    for i in $(seq 1 $num_requests); do
        curl -s --proxy "$proxy_url" --connect-timeout 10 "$test_url" > /dev/null 2>&1 &
    done
    
    wait
    local end_time=$(date +%s.%N)
    local duration=$(echo "$end_time - $start_time" | bc)
    
    echo -e "${GREEN}✓ Completed $num_requests requests in ${duration}s${NC}"
}

# Function to show proxy configuration
show_proxy_config() {
    echo -e "${YELLOW}Proxy Configuration:${NC}"
    echo "HTTP_PROXY=http://$PROXY_SERVER"
    echo "HTTPS_PROXY=http://$PROXY_SERVER"
    echo ""
    echo "Browser Configuration:"
    echo "Chrome: --proxy-server=http://$PROXY_SERVER"
    echo "Firefox: Manual proxy configuration"
    echo ""
    echo "Command Line:"
    echo "curl --proxy http://$PROXY_SERVER https://example.com"
    echo ""
}

# Main test execution
main() {
    echo -e "${BLUE}Starting WDNS Proxy Server Tests${NC}"
    echo ""
    
    # Test if services are running
    test_service_running "DNS Service" "$DNS_SERVER"
    test_service_running "Proxy Service" "$PROXY_SERVER"
    echo ""
    
    # Test proxy functionality
    test_proxy_http
    echo ""
    
    test_proxy_https
    echo ""
    
    test_proxy_env
    echo ""
    
    test_proxy_response
    echo ""
    
    test_proxy_hosts
    echo ""
    
    test_proxy_performance
    echo ""
    
    show_proxy_config
    
    echo -e "${GREEN}Proxy server tests completed!${NC}"
}

# Check if required tools are available
check_dependencies() {
    local missing_tools=()
    
    if ! command -v curl &> /dev/null; then
        missing_tools+=("curl")
    fi
    
    if ! command -v jq &> /dev/null; then
        echo -e "${YELLOW}Warning: jq not found. JSON responses will not be formatted.${NC}"
    fi
    
    if ! command -v bc &> /dev/null; then
        echo -e "${YELLOW}Warning: bc not found. Performance timing may not work.${NC}"
    fi
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        echo -e "${RED}Error: Missing required tools: ${missing_tools[*]}${NC}"
        echo "Please install the missing tools and try again."
        exit 1
    fi
}

# Show usage information
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  -s, --server   Set proxy server address (default: 127.0.0.1:9701)"
    echo ""
    echo "Environment Variables:"
    echo "  WDNS_SERVER    Set proxy server address (default: 127.0.0.1:9701)"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Test with default server"
    echo "  $0 -s 192.168.1.100:9701            # Test with specific server"
    echo "  WDNS_SERVER=192.168.1.100:9701 $0   # Test with environment variable"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_usage
            exit 0
            ;;
        -s|--server)
            PROXY_SERVER="$2"
            DNS_SERVER="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Run the tests
check_dependencies
main
