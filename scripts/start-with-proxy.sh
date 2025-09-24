#!/bin/bash

# Start Applications with WDNS Proxy
# Uruchamia aplikacje z konfiguracją proxy

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default proxy server address
PROXY_SERVER="${WDNS_PROXY_SERVER:-127.0.0.1:9701}"

echo -e "${BLUE}Uruchamianie aplikacji z WDNS Proxy${NC}"
echo "===================================="
echo "Proxy Server: $PROXY_SERVER"
echo ""

# Function to check if WDNS service is running
check_wdns_service() {
    if curl -s --connect-timeout 5 "http://$PROXY_SERVER" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Serwis WDNS proxy działa${NC}"
        return 0
    else
        echo -e "${RED}✗ Serwis WDNS proxy nie działa${NC}"
        echo "Uruchom serwis WDNS:"
        echo "  ./target/release/wdns-service"
        return 1
    fi
}

# Function to start Terminal with proxy
start_terminal() {
    echo -e "${YELLOW}Uruchamianie Terminal z proxy...${NC}"
    
    # Source proxy environment
    source "$HOME/.wdns-proxy-env" 2>/dev/null || true
    
    # Start new Terminal window
    osascript << EOF
tell application "Terminal"
    activate
    do script "echo 'Terminal uruchomiony z WDNS proxy' && env | grep -i proxy"
end tell
EOF

    echo -e "${GREEN}✓ Terminal uruchomiony z proxy${NC}"
}

# Function to start Chrome with proxy
start_chrome() {
    echo -e "${YELLOW}Uruchamianie Chrome z proxy...${NC}"
    
    if [[ -d "/Applications/Google Chrome.app" ]]; then
        open -a "Google Chrome" --args --proxy-server="http://$PROXY_SERVER"
        echo -e "${GREEN}✓ Chrome uruchomiony z proxy${NC}"
    else
        echo -e "${RED}✗ Chrome nie znaleziony${NC}"
    fi
}

# Function to start Firefox with proxy
start_firefox() {
    echo -e "${YELLOW}Uruchamianie Firefox z proxy...${NC}"
    
    if [[ -d "/Applications/Firefox.app" ]]; then
        # Create Firefox profile directory
        FIREFOX_PROFILE_DIR="$HOME/.wdns-proxy-firefox"
        mkdir -p "$FIREFOX_PROFILE_DIR"
        
        # Create user.js with proxy settings
        cat > "$FIREFOX_PROFILE_DIR/user.js" << EOF
// WDNS Proxy Configuration for Firefox
user_pref("network.proxy.type", 1);
user_pref("network.proxy.http", "$(echo $PROXY_SERVER | cut -d: -f1)");
user_pref("network.proxy.http_port", $(echo $PROXY_SERVER | cut -d: -f2));
user_pref("network.proxy.ssl", "$(echo $PROXY_SERVER | cut -d: -f1)");
user_pref("network.proxy.ssl_port", $(echo $PROXY_SERVER | cut -d: -f2));
user_pref("network.proxy.share_proxy_settings", true);
user_pref("network.proxy.no_proxies_on", "localhost, 127.0.0.1");
EOF
        
        # Start Firefox with custom profile
        /Applications/Firefox.app/Contents/MacOS/firefox -profile "$FIREFOX_PROFILE_DIR" &
        echo -e "${GREEN}✓ Firefox uruchomiony z proxy${NC}"
    else
        echo -e "${RED}✗ Firefox nie znaleziony${NC}"
    fi
}

# Function to start Safari (uses system proxy)
start_safari() {
    echo -e "${YELLOW}Uruchamianie Safari...${NC}"
    
    if [[ -d "/Applications/Safari.app" ]]; then
        open -a "Safari"
        echo -e "${GREEN}✓ Safari uruchomiony (używa system proxy)${NC}"
    else
        echo -e "${RED}✗ Safari nie znaleziony${NC}"
    fi
}

# Function to start VS Code with proxy
start_vscode() {
    echo -e "${YELLOW}Uruchamianie VS Code z proxy...${NC}"
    
    if command -v code > /dev/null 2>&1; then
        # Set proxy environment variables
        export HTTP_PROXY="http://$PROXY_SERVER"
        export HTTPS_PROXY="http://$PROXY_SERVER"
        export http_proxy="http://$PROXY_SERVER"
        export https_proxy="http://$PROXY_SERVER"
        
        # Start VS Code
        code &
        echo -e "${GREEN}✓ VS Code uruchomiony z proxy${NC}"
    else
        echo -e "${RED}✗ VS Code nie znaleziony${NC}"
    fi
}

# Function to show usage
show_usage() {
    echo "Użycie: $0 [APLIKACJA]"
    echo ""
    echo "Aplikacje:"
    echo "  terminal    Uruchom Terminal z proxy"
    echo "  chrome      Uruchom Chrome z proxy"
    echo "  firefox     Uruchom Firefox z proxy"
    echo "  safari      Uruchom Safari (system proxy)"
    echo "  vscode      Uruchom VS Code z proxy"
    echo "  all         Uruchom wszystkie aplikacje"
    echo ""
    echo "Przykłady:"
    echo "  $0 terminal     # Uruchom Terminal"
    echo "  $0 chrome       # Uruchom Chrome"
    echo "  $0 all          # Uruchom wszystkie"
}

# Main function
main() {
    # Check if WDNS service is running
    if ! check_wdns_service; then
        exit 1
    fi
    
    case "${1:-}" in
        terminal)
            start_terminal
            ;;
        chrome)
            start_chrome
            ;;
        firefox)
            start_firefox
            ;;
        safari)
            start_safari
            ;;
        vscode)
            start_vscode
            ;;
        all)
            echo -e "${YELLOW}Uruchamianie wszystkich aplikacji...${NC}"
            start_terminal
            sleep 2
            start_chrome
            sleep 2
            start_firefox
            sleep 2
            start_safari
            sleep 2
            start_vscode
            echo -e "${GREEN}✓ Wszystkie aplikacje uruchomione${NC}"
            ;;
        help|--help|-h)
            show_usage
            ;;
        "")
            echo "Wybierz aplikację do uruchomienia:"
            echo ""
            show_usage
            ;;
        *)
            echo "Nieznana aplikacja: $1"
            show_usage
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
