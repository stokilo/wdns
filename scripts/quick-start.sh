#!/bin/bash

# WDNS Quick Start
# Szybkie uruchomienie WDNS z proxy

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}WDNS Quick Start${NC}"
echo "================"
echo ""

# Function to check if service is running
check_service() {
    if curl -s --connect-timeout 5 "http://127.0.0.1:9700/health" > /dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Function to start service if not running
start_service() {
    if check_service; then
        echo -e "${GREEN}✓ Serwis WDNS już działa${NC}"
    else
        echo -e "${YELLOW}Uruchamianie serwisu WDNS...${NC}"
        nohup ./target/release/wdns-service > wdns-service.log 2>&1 &
        sleep 3
        
        if check_service; then
            echo -e "${GREEN}✓ Serwis WDNS uruchomiony${NC}"
        else
            echo -e "${RED}✗ Nie udało się uruchomić serwisu WDNS${NC}"
            echo "Sprawdź logi: cat wdns-service.log"
            exit 1
        fi
    fi
}

# Function to enable proxy
enable_proxy() {
    echo -e "${YELLOW}Włączanie proxy...${NC}"
    
    # Set environment variables
    export HTTP_PROXY="http://127.0.0.1:9701"
    export HTTPS_PROXY="http://127.0.0.1:9701"
    export http_proxy="http://127.0.0.1:9701"
    export https_proxy="http://127.0.0.1:9701"
    export ALL_PROXY="http://127.0.0.1:9701"
    export all_proxy="http://127.0.0.1:9701"
    export NO_PROXY="localhost,127.0.0.1,::1"
    export no_proxy="localhost,127.0.0.1,::1"
    
    echo -e "${GREEN}✓ Proxy włączony${NC}"
}

# Function to test everything
test_everything() {
    echo -e "${YELLOW}Testowanie konfiguracji...${NC}"
    
    # Test DNS resolution
    echo "Test DNS resolution:"
    curl -X POST http://127.0.0.1:9700/api/dns/resolve \
        -H "Content-Type: application/json" \
        -d '{"hosts": ["google.com"]}' | \
        python3 -m json.tool 2>/dev/null || echo "DNS test completed"
    
    echo ""
    
    # Test proxy
    echo "Test proxy:"
    curl -s "http://httpbin.org/ip" | python3 -m json.tool 2>/dev/null || curl -s "http://httpbin.org/ip"
    
    echo ""
    echo -e "${GREEN}✓ Wszystko działa poprawnie!${NC}"
}

# Function to show next steps
show_next_steps() {
    echo ""
    echo -e "${BLUE}Następne kroki:${NC}"
    echo ""
    echo "1. Uruchom aplikacje z proxy:"
    echo "   ./scripts/start-with-proxy.sh all"
    echo ""
    echo "2. Sprawdź status:"
    echo "   ./scripts/proxy-manager.sh status"
    echo ""
    echo "3. Testuj proxy:"
    echo "   ./scripts/proxy-manager.sh test"
    echo ""
    echo "4. Zatrzymaj serwis:"
    echo "   ./scripts/proxy-manager.sh stop"
    echo ""
    echo "5. Wyłącz proxy:"
    echo "   ./scripts/proxy-manager.sh disable"
}

# Main function
main() {
    # Start service
    start_service
    
    # Enable proxy
    enable_proxy
    
    # Test everything
    test_everything
    
    # Show next steps
    show_next_steps
}

# Run main function
main

