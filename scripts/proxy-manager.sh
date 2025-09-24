#!/bin/bash

# WDNS Proxy Manager
# Prosty menedżer proxy dla WDNS

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default proxy server address
PROXY_SERVER="${WDNS_PROXY_SERVER:-127.0.0.1:9701}"

echo -e "${BLUE}WDNS Proxy Manager${NC}"
echo "=================="
echo ""

# Function to check WDNS service status
check_status() {
    echo -e "${YELLOW}Sprawdzanie statusu serwisu WDNS...${NC}"
    
    # Check DNS service
    if curl -s --connect-timeout 5 "http://127.0.0.1:9700/health" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Serwis DNS działa na porcie 9700${NC}"
    else
        echo -e "${RED}✗ Serwis DNS nie działa${NC}"
    fi
    
    # Check proxy service
    if curl -s --connect-timeout 5 "http://$PROXY_SERVER" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Serwis proxy działa na porcie $PROXY_SERVER${NC}"
    else
        echo -e "${RED}✗ Serwis proxy nie działa${NC}"
    fi
    
    # Check environment variables
    echo -e "${YELLOW}Zmienne środowiskowe proxy:${NC}"
    env | grep -i proxy || echo "Brak zmiennych proxy"
}

# Function to start WDNS service
start_service() {
    echo -e "${YELLOW}Uruchamianie serwisu WDNS...${NC}"
    
    if pgrep -f "wdns-service" > /dev/null; then
        echo -e "${GREEN}✓ Serwis WDNS już działa${NC}"
    else
        echo "Uruchamianie serwisu w tle..."
        nohup ./target/release/wdns-service > wdns-service.log 2>&1 &
        sleep 3
        
        if pgrep -f "wdns-service" > /dev/null; then
            echo -e "${GREEN}✓ Serwis WDNS uruchomiony${NC}"
        else
            echo -e "${RED}✗ Nie udało się uruchomić serwisu WDNS${NC}"
            echo "Sprawdź logi: cat wdns-service.log"
        fi
    fi
}

# Function to stop WDNS service
stop_service() {
    echo -e "${YELLOW}Zatrzymywanie serwisu WDNS...${NC}"
    
    if pgrep -f "wdns-service" > /dev/null; then
        pkill -f "wdns-service"
        echo -e "${GREEN}✓ Serwis WDNS zatrzymany${NC}"
    else
        echo -e "${YELLOW}Serwis WDNS nie działa${NC}"
    fi
}

# Function to enable proxy
enable_proxy() {
    echo -e "${YELLOW}Włączanie proxy...${NC}"
    
    # Check if service is running
    if ! curl -s --connect-timeout 5 "http://$PROXY_SERVER" > /dev/null 2>&1; then
        echo -e "${RED}✗ Serwis WDNS nie działa${NC}"
        echo "Uruchom serwis: $0 start"
        return 1
    fi
    
    # Set environment variables
    export HTTP_PROXY="http://$PROXY_SERVER"
    export HTTPS_PROXY="http://$PROXY_SERVER"
    export http_proxy="http://$PROXY_SERVER"
    export https_proxy="http://$PROXY_SERVER"
    export ALL_PROXY="http://$PROXY_SERVER"
    export all_proxy="http://$PROXY_SERVER"
    export NO_PROXY="localhost,127.0.0.1,::1"
    export no_proxy="localhost,127.0.0.1,::1"
    
    echo -e "${GREEN}✓ Proxy włączony${NC}"
    echo "Zmienne środowiskowe ustawione dla tej sesji"
}

# Function to disable proxy
disable_proxy() {
    echo -e "${YELLOW}Wyłączanie proxy...${NC}"
    
    unset HTTP_PROXY HTTPS_PROXY http_proxy https_proxy ALL_PROXY all_proxy NO_PROXY no_proxy
    
    echo -e "${GREEN}✓ Proxy wyłączony${NC}"
}

# Function to test proxy
test_proxy() {
    echo -e "${YELLOW}Testowanie proxy...${NC}"
    
    # Test HTTP request
    if curl -s --connect-timeout 10 "http://httpbin.org/ip" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Test HTTP proxy przeszedł${NC}"
    else
        echo -e "${RED}✗ Test HTTP proxy nie przeszedł${NC}"
    fi
    
    # Show current IP
    echo -e "${YELLOW}Twój aktualny IP:${NC}"
    curl -s "http://httpbin.org/ip" | python3 -m json.tool 2>/dev/null || curl -s "http://httpbin.org/ip"
}

# Function to show DNS resolution
show_dns() {
    echo -e "${YELLOW}Rozwiązywanie DNS przez WDNS...${NC}"
    
    curl -X POST http://127.0.0.1:9700/api/dns/resolve \
        -H "Content-Type: application/json" \
        -d '{"hosts": ["google.com", "github.com", "stackoverflow.com"]}' | \
        python3 -m json.tool 2>/dev/null || echo "DNS resolution completed"
}

# Function to start applications
start_apps() {
    echo -e "${YELLOW}Uruchamianie aplikacji z proxy...${NC}"
    
    ./scripts/start-with-proxy.sh all
}

# Function to show usage
show_usage() {
    echo "Użycie: $0 [KOMENDA]"
    echo ""
    echo "Komendy:"
    echo "  start       Uruchom serwis WDNS"
    echo "  stop        Zatrzymaj serwis WDNS"
    echo "  status      Pokaż status serwisu"
    echo "  enable      Włącz proxy (zmienne środowiskowe)"
    echo "  disable     Wyłącz proxy"
    echo "  test        Testuj proxy"
    echo "  dns         Pokaż rozwiązywanie DNS"
    echo "  apps        Uruchom aplikacje z proxy"
    echo "  help        Pokaż tę pomoc"
    echo ""
    echo "Przykłady:"
    echo "  $0 start        # Uruchom serwis"
    echo "  $0 enable       # Włącz proxy"
    echo "  $0 test         # Testuj proxy"
    echo "  $0 apps         # Uruchom aplikacje"
    echo "  $0 stop         # Zatrzymaj serwis"
}

# Main function
main() {
    case "${1:-}" in
        start)
            start_service
            ;;
        stop)
            stop_service
            ;;
        status)
            check_status
            ;;
        enable)
            enable_proxy
            ;;
        disable)
            disable_proxy
            ;;
        test)
            test_proxy
            ;;
        dns)
            show_dns
            ;;
        apps)
            start_apps
            ;;
        help|--help|-h)
            show_usage
            ;;
        "")
            echo "Wybierz komendę:"
            echo ""
            show_usage
            ;;
        *)
            echo "Nieznana komenda: $1"
            show_usage
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
