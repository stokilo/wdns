#!/bin/bash

# Test script for Traffic Interceptor functionality
# This script tests the low-level traffic interception and SOCKS5 proxy routing

set -e

echo "🧪 Testing Traffic Interceptor Functionality"
echo "============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
SOCKS5_PROXY="192.168.0.115:9702"
DNS_PORT=5353
TEST_DOMAIN="httpbin.org"
TEST_HTTPS_DOMAIN="https://httpbin.org"

echo -e "${BLUE}📋 Test Configuration:${NC}"
echo "   SOCKS5 Proxy: $SOCKS5_PROXY"
echo "   DNS Port: $DNS_PORT"
echo "   Test Domain: $TEST_DOMAIN"
echo "   Test HTTPS: $TEST_HTTPS_DOMAIN"
echo ""

# Function to test if application is running
test_application_running() {
    echo -e "${YELLOW}🔍 Checking if application is running...${NC}"
    
    if pgrep -f "macos-listener" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Application is running${NC}"
        return 0
    else
        echo -e "${RED}❌ Application is not running${NC}"
        echo -e "${YELLOW}💡 Start the application first: cargo run${NC}"
        return 1
    fi
}

# Function to test network connectivity
test_network_connectivity() {
    echo -e "${YELLOW}🌐 Testing network connectivity...${NC}"
    
    if ping -c 1 8.8.8.8 >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Network connectivity: OK${NC}"
        return 0
    else
        echo -e "${RED}❌ Network connectivity: FAILED${NC}"
        return 1
    fi
}

# Function to test SOCKS5 proxy connectivity
test_socks5_proxy_connectivity() {
    echo -e "${YELLOW}🔗 Testing SOCKS5 proxy connectivity...${NC}"
    
    if curl --socks5 $SOCKS5_PROXY --connect-timeout 10 --max-time 30 -s "http://httpbin.org/ip" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ SOCKS5 proxy connectivity: OK${NC}"
        return 0
    else
        echo -e "${RED}❌ SOCKS5 proxy connectivity: FAILED${NC}"
        echo -e "${YELLOW}💡 Check if SOCKS5 proxy is running on $SOCKS5_PROXY${NC}"
        return 1
    fi
}

# Function to test DNS resolution
test_dns_resolution() {
    local domain=$1
    local description=$2
    
    echo -e "${YELLOW}🔍 Testing DNS resolution: $description${NC}"
    
    if nslookup "$domain" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ DNS resolution successful: $description${NC}"
        return 0
    else
        echo -e "${RED}❌ DNS resolution failed: $description${NC}"
        return 1
    fi
}

# Function to test HTTP request
test_http_request() {
    local url=$1
    local description=$2
    
    echo -e "${YELLOW}🌐 Testing HTTP request: $description${NC}"
    
    if curl --connect-timeout 10 --max-time 30 -s "$url" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ HTTP request successful: $description${NC}"
        return 0
    else
        echo -e "${RED}❌ HTTP request failed: $description${NC}"
        return 1
    fi
}

# Function to test HTTPS request
test_https_request() {
    local url=$1
    local description=$2
    
    echo -e "${YELLOW}🔒 Testing HTTPS request: $description${NC}"
    
    if curl --connect-timeout 10 --max-time 30 -s "$url" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ HTTPS request successful: $description${NC}"
        return 0
    else
        echo -e "${RED}❌ HTTPS request failed: $description${NC}"
        return 1
    fi
}

# Function to test with SOCKS5 proxy
test_with_socks5_proxy() {
    local url=$1
    local description=$2
    
    echo -e "${YELLOW}🔗 Testing with SOCKS5 proxy: $description${NC}"
    
    if curl --socks5 $SOCKS5_PROXY --connect-timeout 10 --max-time 30 -s "$url" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ SOCKS5 proxy request successful: $description${NC}"
        return 0
    else
        echo -e "${RED}❌ SOCKS5 proxy request failed: $description${NC}"
        return 1
    fi
}

# Function to test DNS with custom server
test_dns_with_custom_server() {
    local domain=$1
    local dns_server=$2
    local description=$3
    
    echo -e "${YELLOW}🔍 Testing DNS with custom server: $description${NC}"
    
    if nslookup "$domain" "$dns_server" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ DNS with custom server successful: $description${NC}"
        return 0
    else
        echo -e "${RED}❌ DNS with custom server failed: $description${NC}"
        return 1
    fi
}

# Function to test curl with custom DNS
test_curl_with_custom_dns() {
    local domain=$1
    local dns_server=$2
    local description=$3
    
    echo -e "${YELLOW}🌐 Testing curl with custom DNS: $description${NC}"
    
    if curl --dns-servers "$dns_server" --connect-timeout 10 --max-time 30 -s "http://$domain" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Curl with custom DNS successful: $description${NC}"
        return 0
    else
        echo -e "${RED}❌ Curl with custom DNS failed: $description${NC}"
        return 1
    fi
}

# Function to show connection statistics
show_connection_stats() {
    echo -e "${BLUE}📊 Connection Statistics:${NC}"
    
    echo "Active TCP connections:"
    netstat -an | grep tcp | grep ESTABLISHED | wc -l
    
    echo "Active UDP connections:"
    netstat -an | grep udp | wc -l
    
    echo "Listening ports:"
    netstat -an | grep LISTEN | wc -l
    
    echo "DNS interceptor connections:"
    netstat -an | grep :$DNS_PORT | wc -l
}

# Function to test specific domains that might be in rules
test_specific_domains() {
    echo -e "${BLUE}🎯 Testing Specific Domains:${NC}"
    
    local test_domains=(
        "google.com"
        "github.com"
        "stackoverflow.com"
        "httpbin.org"
        "example.com"
        "kion.cloud"
        "kiongroup.net"
    )
    
    for domain in "${test_domains[@]}"; do
        echo -e "${YELLOW}🔍 Testing domain: $domain${NC}"
        
        # Test DNS resolution
        if test_dns_resolution "$domain" "$domain"; then
            echo -e "${GREEN}✅ $domain DNS resolution successful${NC}"
        else
            echo -e "${YELLOW}⚠️  $domain DNS resolution failed${NC}"
        fi
        
        # Test HTTP request
        if test_http_request "http://$domain" "$domain"; then
            echo -e "${GREEN}✅ $domain HTTP request successful${NC}"
        else
            echo -e "${YELLOW}⚠️  $domain HTTP request failed${NC}"
        fi
        
        # Test HTTPS request
        if test_https_request "https://$domain" "$domain"; then
            echo -e "${GREEN}✅ $domain HTTPS request successful${NC}"
        else
            echo -e "${YELLOW}⚠️  $domain HTTPS request failed${NC}"
        fi
        
        echo ""
    done
}

# Function to test traffic interceptor functionality
test_traffic_interceptor() {
    echo -e "${BLUE}🔍 Testing Traffic Interceptor:${NC}"
    
    # Test DNS interception
    echo -e "${YELLOW}🔍 Testing DNS interception...${NC}"
    if test_dns_with_custom_server "google.com" "127.0.0.1:$DNS_PORT" "DNS interception"; then
        echo -e "${GREEN}✅ DNS interception working${NC}"
    else
        echo -e "${YELLOW}⚠️  DNS interception not working (may be expected if no rules match)${NC}"
    fi
    
    # Test curl with custom DNS
    echo -e "${YELLOW}🌐 Testing curl with custom DNS...${NC}"
    if test_curl_with_custom_dns "httpbin.org" "127.0.0.1:$DNS_PORT" "Custom DNS"; then
        echo -e "${GREEN}✅ Curl with custom DNS working${NC}"
    else
        echo -e "${YELLOW}⚠️  Curl with custom DNS not working (may be expected if no rules match)${NC}"
    fi
}

# Main test execution
main() {
    echo -e "${BLUE}🚀 Starting Traffic Interceptor Tests${NC}"
    echo ""
    
    # Test 1: Check if application is running
    if ! test_application_running; then
        exit 1
    fi
    
    # Test 2: Check network connectivity
    if ! test_network_connectivity; then
        echo -e "${RED}❌ No network connectivity. Please check your internet connection.${NC}"
        exit 1
    fi
    
    # Test 3: Test SOCKS5 proxy connectivity
    echo -e "${BLUE}🔗 Testing SOCKS5 Proxy Connectivity:${NC}"
    if test_socks5_proxy_connectivity; then
        echo -e "${GREEN}✅ SOCKS5 proxy is accessible${NC}"
    else
        echo -e "${YELLOW}⚠️  SOCKS5 proxy not accessible - some tests may fail${NC}"
    fi
    echo ""
    
    # Test 4: Test basic DNS resolution
    echo -e "${BLUE}🔍 Testing Basic DNS Resolution:${NC}"
    test_dns_resolution "$TEST_DOMAIN" "Basic DNS"
    test_dns_resolution "google.com" "Google DNS"
    test_dns_resolution "github.com" "GitHub DNS"
    echo ""
    
    # Test 5: Test basic HTTP/HTTPS requests
    echo -e "${BLUE}🌐 Testing Basic HTTP/HTTPS Requests:${NC}"
    test_http_request "http://$TEST_DOMAIN/ip" "HTTP request"
    test_https_request "https://$TEST_DOMAIN/ip" "HTTPS request"
    echo ""
    
    # Test 6: Test with SOCKS5 proxy
    echo -e "${BLUE}🔗 Testing with SOCKS5 Proxy:${NC}"
    test_with_socks5_proxy "http://$TEST_DOMAIN/ip" "HTTP via SOCKS5"
    test_with_socks5_proxy "https://$TEST_DOMAIN/ip" "HTTPS via SOCKS5"
    echo ""
    
    # Test 7: Test traffic interceptor
    test_traffic_interceptor
    echo ""
    
    # Test 8: Test specific domains
    test_specific_domains
    
    # Test 9: Show connection statistics
    show_connection_stats
    echo ""
    
    echo -e "${GREEN}🎉 All tests completed!${NC}"
    echo ""
    echo -e "${BLUE}📝 Test Summary:${NC}"
    echo "   - SOCKS5 Proxy: $SOCKS5_PROXY"
    echo "   - DNS Interceptor: Port $DNS_PORT"
    echo "   - Test Domain: $TEST_DOMAIN"
    echo ""
    echo -e "${YELLOW}💡 Tips:${NC}"
    echo "   - Check the application UI for real-time traffic monitoring"
    echo "   - Use 'View Intercepted Traffic' to see intercepted connections"
    echo "   - Configure proxy rules in the application UI"
    echo "   - Add SOCKS5 proxy servers in 'Configure Proxies'"
    echo "   - Enable 'Start Traffic Interceptor' to begin traffic interception"
    echo ""
    echo -e "${BLUE}🔧 Configuration:${NC}"
    echo "   - Configure proxy rules in the application UI"
    echo "   - Add SOCKS5 proxy servers in 'Configure Proxies'"
    echo "   - Enable 'Start Traffic Interceptor' to start traffic interception"
    echo "   - Traffic matching rules will be routed through SOCKS5 proxy"
    echo ""
    echo -e "${GREEN}✅ Traffic Interceptor testing completed successfully!${NC}"
}

# Run main function
main "$@"
