#!/bin/bash

# Quick macOS Proxy Setup for WDNS Service
# This script quickly configures macOS to use the WDNS proxy server

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default proxy server address
PROXY_SERVER="${WDNS_PROXY_SERVER:-127.0.0.1:9701}"

echo -e "${BLUE}WDNS Quick Proxy Setup for macOS${NC}"
echo "====================================="
echo "Proxy Server: $PROXY_SERVER"
echo ""

# Function to check if WDNS service is running
check_wdns_service() {
    echo -e "${YELLOW}Checking WDNS service...${NC}"
    
    if curl -s --connect-timeout 5 "http://$PROXY_SERVER" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ WDNS proxy server is running${NC}"
        return 0
    else
        echo -e "${RED}✗ WDNS proxy server is not running${NC}"
        echo "Please start the WDNS service first:"
        echo "  ./target/release/wdns-service"
        return 1
    fi
}

# Function to configure system proxy
configure_system_proxy() {
    echo -e "${YELLOW}Configuring system proxy settings...${NC}"
    
    PROXY_HOST=$(echo $PROXY_SERVER | cut -d: -f1)
    PROXY_PORT=$(echo $PROXY_SERVER | cut -d: -f2)
    
    # Configure HTTP proxy
    sudo networksetup -setwebproxy "Wi-Fi" "$PROXY_HOST" "$PROXY_PORT"
    sudo networksetup -setwebproxystate "Wi-Fi" on
    
    # Configure HTTPS proxy
    sudo networksetup -setsecurewebproxy "Wi-Fi" "$PROXY_HOST" "$PROXY_PORT"
    sudo networksetup -setsecurewebproxystate "Wi-Fi" on
    
    echo -e "${GREEN}✓ System proxy configured${NC}"
}

# Function to set environment variables
set_environment_variables() {
    echo -e "${YELLOW}Setting environment variables...${NC}"
    
    # Set environment variables for current session
    export HTTP_PROXY="http://$PROXY_SERVER"
    export HTTPS_PROXY="http://$PROXY_SERVER"
    export http_proxy="http://$PROXY_SERVER"
    export https_proxy="http://$PROXY_SERVER"
    export ALL_PROXY="http://$PROXY_SERVER"
    export all_proxy="http://$PROXY_SERVER"
    export NO_PROXY="localhost,127.0.0.1,::1"
    export no_proxy="localhost,127.0.0.1,::1"
    
    echo -e "${GREEN}✓ Environment variables set${NC}"
}

# Function to create shell profile configuration
create_shell_config() {
    echo -e "${YELLOW}Creating shell configuration...${NC}"
    
    SHELL_CONFIG="$HOME/.wdns-proxy-env"
    
    cat > "$SHELL_CONFIG" << EOF
# WDNS Proxy Environment Variables
# Generated on $(date)

export HTTP_PROXY="http://$PROXY_SERVER"
export HTTPS_PROXY="http://$PROXY_SERVER"
export http_proxy="http://$PROXY_SERVER"
export https_proxy="http://$PROXY_SERVER"
export ALL_PROXY="http://$PROXY_SERVER"
export all_proxy="http://$PROXY_SERVER"
export NO_PROXY="localhost,127.0.0.1,::1"
export no_proxy="localhost,127.0.0.1,::1"

echo "WDNS proxy environment loaded"
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
                echo -e "${GREEN}✓ Added to $config_file${NC}"
            fi
        fi
    done
}

# Function to start applications with proxy
start_applications() {
    echo -e "${YELLOW}Starting applications with proxy configuration...${NC}"
    
    # Start Terminal with proxy environment
    osascript << EOF
tell application "Terminal"
    activate
    do script "source $HOME/.wdns-proxy-env && echo 'Terminal started with WDNS proxy' && env | grep -i proxy"
end tell
EOF

    # Start Chrome with proxy
    if [[ -d "/Applications/Google Chrome.app" ]]; then
        open -a "Google Chrome" --args --proxy-server="http://$PROXY_SERVER"
        echo -e "${GREEN}✓ Chrome started with proxy${NC}"
    fi
    
    # Start Firefox with proxy
    if [[ -d "/Applications/Firefox.app" ]]; then
        # Create Firefox profile with proxy settings
        FIREFOX_PROFILE_DIR="$HOME/.wdns-proxy-firefox"
        mkdir -p "$FIREFOX_PROFILE_DIR"
        
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
        
        /Applications/Firefox.app/Contents/MacOS/firefox -profile "$FIREFOX_PROFILE_DIR" &
        echo -e "${GREEN}✓ Firefox started with proxy${NC}"
    fi
}

# Function to test proxy configuration
test_proxy() {
    echo -e "${YELLOW}Testing proxy configuration...${NC}"
    
    # Test HTTP request
    if curl -s --proxy "http://$PROXY_SERVER" --connect-timeout 10 "http://httpbin.org/ip" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ HTTP proxy test passed${NC}"
    else
        echo -e "${RED}✗ HTTP proxy test failed${NC}"
    fi
    
    # Test HTTPS request
    if curl -s --proxy "http://$PROXY_SERVER" --connect-timeout 10 "https://httpbin.org/ip" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ HTTPS proxy test passed${NC}"
    else
        echo -e "${RED}✗ HTTPS proxy test failed${NC}"
    fi
    
    # Show current IP
    echo -e "${YELLOW}Your current IP through proxy:${NC}"
    curl -s --proxy "http://$PROXY_SERVER" "http://httpbin.org/ip" | python3 -m json.tool 2>/dev/null || curl -s --proxy "http://$PROXY_SERVER" "http://httpbin.org/ip"
}

# Function to disable proxy
disable_proxy() {
    echo -e "${YELLOW}Disabling proxy configuration...${NC}"
    
    # Disable system proxy
    sudo networksetup -setwebproxystate "Wi-Fi" off
    sudo networksetup -setsecurewebproxystate "Wi-Fi" off
    
    # Unset environment variables
    unset HTTP_PROXY HTTPS_PROXY http_proxy https_proxy ALL_PROXY all_proxy NO_PROXY no_proxy
    
    echo -e "${GREEN}✓ Proxy disabled${NC}"
}

# Function to show current proxy status
show_status() {
    echo -e "${YELLOW}Current proxy status:${NC}"
    
    # Check system proxy settings
    echo "System HTTP Proxy:"
    networksetup -getwebproxy "Wi-Fi"
    echo ""
    echo "System HTTPS Proxy:"
    networksetup -getsecurewebproxy "Wi-Fi"
    echo ""
    echo "Environment Variables:"
    env | grep -i proxy || echo "No proxy environment variables set"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  -s, --server   Set proxy server address (default: 127.0.0.1:9701)"
    echo "  -e, --enable   Enable proxy configuration"
    echo "  -d, --disable  Disable proxy configuration"
    echo "  -t, --test     Test proxy configuration"
    echo "  -a, --apps     Start applications with proxy"
    echo "  -u, --status   Show current proxy status"
    echo ""
    echo "Examples:"
    echo "  $0 -e                    # Enable proxy"
    echo "  $0 -t                    # Test proxy"
    echo "  $0 -a                    # Start applications"
    echo "  $0 -d                    # Disable proxy"
    echo "  $0 -s 192.168.1.100:9701 # Use specific server"
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
            configure_system_proxy
            set_environment_variables
            create_shell_config
            test_proxy
            echo -e "${GREEN}✓ Proxy configuration enabled${NC}"
            ;;
        -d|--disable)
            disable_proxy
            ;;
        -t|--test)
            test_proxy
            ;;
        -a|--apps)
            start_applications
            ;;
        -u|--status)
            show_status
            ;;
        "")
            # Default: enable proxy
            check_wdns_service || exit 1
            configure_system_proxy
            set_environment_variables
            create_shell_config
            test_proxy
            echo -e "${GREEN}✓ Proxy configuration enabled${NC}"
            echo ""
            echo "To start applications with proxy:"
            echo "  $0 -a"
            echo ""
            echo "To test proxy:"
            echo "  $0 -t"
            echo ""
            echo "To disable proxy:"
            echo "  $0 -d"
            ;;
        *)
            echo "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"
