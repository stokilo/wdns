#!/bin/bash

# WDNS Proxy Environment Setup (bez sudo)
# Ten skrypt konfiguruje zmienne środowiskowe dla proxy

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default proxy server address
PROXY_SERVER="${WDNS_PROXY_SERVER:-127.0.0.1:9701}"

echo -e "${BLUE}WDNS Proxy Environment Setup${NC}"
echo "================================"
echo "Proxy Server: $PROXY_SERVER"
echo ""

# Function to check if WDNS service is running
check_wdns_service() {
    echo -e "${YELLOW}Sprawdzanie serwisu WDNS...${NC}"
    
    if curl -s --connect-timeout 5 "http://$PROXY_SERVER" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Serwis WDNS proxy działa na porcie $PROXY_SERVER${NC}"
        return 0
    else
        echo -e "${RED}✗ Serwis WDNS proxy nie działa${NC}"
        echo "Uruchom serwis WDNS:"
        echo "  ./target/release/wdns-service"
        return 1
    fi
}

# Function to set environment variables
set_proxy_environment() {
    echo -e "${YELLOW}Konfigurowanie zmiennych środowiskowych...${NC}"
    
    # Set environment variables
    export HTTP_PROXY="http://$PROXY_SERVER"
    export HTTPS_PROXY="http://$PROXY_SERVER"
    export http_proxy="http://$PROXY_SERVER"
    export https_proxy="http://$PROXY_SERVER"
    export ALL_PROXY="http://$PROXY_SERVER"
    export all_proxy="http://$PROXY_SERVER"
    export NO_PROXY="localhost,127.0.0.1,::1"
    export no_proxy="localhost,127.0.0.1,::1"
    
    echo -e "${GREEN}✓ Zmienne środowiskowe ustawione${NC}"
}

# Function to create shell profile configuration
create_shell_config() {
    echo -e "${YELLOW}Tworzenie konfiguracji shell...${NC}"
    
    SHELL_CONFIG="$HOME/.wdns-proxy-env"
    
    cat > "$SHELL_CONFIG" << EOF
# WDNS Proxy Environment Variables
# Wygenerowane $(date)

export HTTP_PROXY="http://$PROXY_SERVER"
export HTTPS_PROXY="http://$PROXY_SERVER"
export http_proxy="http://$PROXY_SERVER"
export https_proxy="http://$PROXY_SERVER"
export ALL_PROXY="http://$PROXY_SERVER"
export all_proxy="http://$PROXY_SERVER"
export NO_PROXY="localhost,127.0.0.1,::1"
export no_proxy="localhost,127.0.0.1,::1"

echo "WDNS proxy environment załadowany"
EOF

    # Add to shell profiles
    SHELL_CONFIGS=("$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.bash_profile" "$HOME/.profile")
    
    for config_file in "${SHELL_CONFIGS[@]}"; do
        if [[ -f "$config_file" ]]; then
            if ! grep -q "WDNS Proxy Environment" "$config_file"; then
                echo "" >> "$config_file"
                echo "# WDNS Proxy Environment" >> "$config_file"
                echo "if [[ -f \"$SHELL_CONFIG\" ]]; then" >> "$config_file"
                echo "    source \"$SHELL_CONFIG\"" >> "$config_file"
                echo "fi" >> "$config_file"
                echo -e "${GREEN}✓ Dodano do $config_file${NC}"
            else
                echo -e "${GREEN}✓ Konfiguracja już istnieje w $config_file${NC}"
            fi
        fi
    done
}

# Function to test proxy configuration
test_proxy() {
    echo -e "${YELLOW}Testowanie konfiguracji proxy...${NC}"
    
    # Test HTTP request
    if curl -s --connect-timeout 10 "http://httpbin.org/ip" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Test HTTP proxy przeszedł${NC}"
    else
        echo -e "${RED}✗ Test HTTP proxy nie przeszedł${NC}"
    fi
    
    # Show current IP
    echo -e "${YELLOW}Twój aktualny IP przez proxy:${NC}"
    curl -s "http://httpbin.org/ip" | python3 -m json.tool 2>/dev/null || curl -s "http://httpbin.org/ip"
}

# Function to show usage
show_usage() {
    echo "Użycie: $0 [OPCJE]"
    echo ""
    echo "Opcje:"
    echo "  -h, --help     Pokaż tę pomoc"
    echo "  -s, --server   Ustaw adres serwera proxy (domyślnie: 127.0.0.1:9701)"
    echo "  -e, --enable   Włącz konfigurację proxy"
    echo "  -t, --test     Testuj konfigurację proxy"
    echo "  -u, --unset    Wyłącz zmienne proxy"
    echo ""
    echo "Przykłady:"
    echo "  $0 -e                    # Włącz proxy"
    echo "  $0 -t                    # Testuj proxy"
    echo "  $0 -u                    # Wyłącz proxy"
    echo "  $0 -s 192.168.1.100:9701 # Użyj konkretnego serwera"
}

# Function to unset proxy environment
unset_proxy_environment() {
    echo -e "${YELLOW}Wyłączanie zmiennych proxy...${NC}"
    
    unset HTTP_PROXY HTTPS_PROXY http_proxy https_proxy ALL_PROXY all_proxy NO_PROXY no_proxy
    
    echo -e "${GREEN}✓ Zmienne proxy wyłączone${NC}"
}

# Function to show current status
show_status() {
    echo -e "${YELLOW}Aktualny status proxy:${NC}"
    echo ""
    echo "Zmienne środowiskowe:"
    env | grep -i proxy || echo "Brak zmiennych proxy"
    echo ""
    echo "Aby włączyć proxy:"
    echo "  source $HOME/.wdns-proxy-env"
    echo ""
    echo "Aby wyłączyć proxy:"
    echo "  unset HTTP_PROXY HTTPS_PROXY http_proxy https_proxy ALL_PROXY all_proxy"
}

# Main function
main() {
    case "${1:-}" in
        -h|--help)
            show_usage
            exit 0
            ;;
        -s|--server)
            PROXY_SERVER="$2"
            shift 2
            main "$@"
            ;;
        -e|--enable)
            check_wdns_service || exit 1
            set_proxy_environment
            create_shell_config
            test_proxy
            echo -e "${GREEN}✓ Konfiguracja proxy włączona${NC}"
            echo ""
            echo "Aby załadować proxy w nowym terminalu:"
            echo "  source $HOME/.wdns-proxy-env"
            ;;
        -t|--test)
            test_proxy
            ;;
        -u|--unset)
            unset_proxy_environment
            ;;
        -u|--status)
            show_status
            ;;
        "")
            # Default: enable proxy
            check_wdns_service || exit 1
            set_proxy_environment
            create_shell_config
            test_proxy
            echo -e "${GREEN}✓ Konfiguracja proxy włączona${NC}"
            echo ""
            echo "Aby załadować proxy w nowym terminalu:"
            echo "  source $HOME/.wdns-proxy-env"
            ;;
        *)
            echo "Nieznana opcja: $1"
            show_usage
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"
