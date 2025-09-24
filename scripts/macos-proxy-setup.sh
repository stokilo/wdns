#!/bin/bash

# macOS Proxy Setup Script for WDNS Service
# This script configures macOS to use the WDNS proxy server for all applications

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default proxy server address
PROXY_SERVER="${WDNS_PROXY_SERVER:-127.0.0.1:9701}"
DNS_SERVER="${WDNS_DNS_SERVER:-127.0.0.1:9700}"

# Configuration files
PROXY_CONFIG_DIR="$HOME/.wdns-proxy"
PROXY_CONFIG_FILE="$PROXY_CONFIG_DIR/config"
PAC_FILE="$PROXY_CONFIG_DIR/proxy.pac"
LAUNCH_AGENT="$HOME/Library/LaunchAgents/com.wdns.proxy.plist"

echo -e "${BLUE}WDNS macOS Proxy Setup${NC}"
echo "=========================="
echo "Proxy Server: $PROXY_SERVER"
echo "DNS Server: $DNS_SERVER"
echo ""

# Function to check if running as root
check_root() {
    if [[ $EUID -eq 0 ]]; then
        echo -e "${RED}Error: This script should not be run as root${NC}"
        echo "Please run as a regular user. The script will prompt for sudo when needed."
        exit 1
    fi
}

# Function to create proxy configuration directory
setup_config_dir() {
    echo -e "${YELLOW}Setting up configuration directory...${NC}"
    
    if [[ ! -d "$PROXY_CONFIG_DIR" ]]; then
        mkdir -p "$PROXY_CONFIG_DIR"
        echo -e "${GREEN}✓ Created configuration directory: $PROXY_CONFIG_DIR${NC}"
    else
        echo -e "${GREEN}✓ Configuration directory already exists${NC}"
    fi
}

# Function to create proxy configuration file
create_proxy_config() {
    echo -e "${YELLOW}Creating proxy configuration...${NC}"
    
    cat > "$PROXY_CONFIG_FILE" << EOF
# WDNS Proxy Configuration
# Generated on $(date)

PROXY_SERVER="$PROXY_SERVER"
DNS_SERVER="$DNS_SERVER"
PROXY_HOST=$(echo $PROXY_SERVER | cut -d: -f1)
PROXY_PORT=$(echo $PROXY_SERVER | cut -d: -f2)

# Environment variables for shell applications
export HTTP_PROXY="http://$PROXY_SERVER"
export HTTPS_PROXY="http://$PROXY_SERVER"
export http_proxy="http://$PROXY_SERVER"
export https_proxy="http://$PROXY_SERVER"
export NO_PROXY="localhost,127.0.0.1,::1"
export no_proxy="localhost,127.0.0.1,::1"

# Additional proxy settings
export ALL_PROXY="http://$PROXY_SERVER"
export all_proxy="http://$PROXY_SERVER"
EOF

    echo -e "${GREEN}✓ Created proxy configuration file${NC}"
}

# Function to create PAC (Proxy Auto-Configuration) file
create_pac_file() {
    echo -e "${YELLOW}Creating PAC file for browser proxy...${NC}"
    
    cat > "$PAC_FILE" << 'EOF'
function FindProxyForURL(url, host) {
    // Direct connections for localhost and private networks
    if (isPlainHostName(host) ||
        host === "127.0.0.1" ||
        host === "localhost" ||
        host === "::1" ||
        isInNet(host, "10.0.0.0", "255.0.0.0") ||
        isInNet(host, "172.16.0.0", "255.240.0.0") ||
        isInNet(host, "192.168.0.0", "255.255.0.0")) {
        return "DIRECT";
    }
    
    // Use proxy for all other connections
    return "PROXY 127.0.0.1:9701";
}
EOF

    echo -e "${GREEN}✓ Created PAC file: $PAC_FILE${NC}"
}

# Function to configure system proxy settings
configure_system_proxy() {
    echo -e "${YELLOW}Configuring system proxy settings...${NC}"
    
    # Configure HTTP proxy
    sudo networksetup -setwebproxy "Wi-Fi" "$(echo $PROXY_SERVER | cut -d: -f1)" "$(echo $PROXY_SERVER | cut -d: -f2)"
    sudo networksetup -setwebproxystate "Wi-Fi" on
    
    # Configure HTTPS proxy
    sudo networksetup -setsecurewebproxy "Wi-Fi" "$(echo $PROXY_SERVER | cut -d: -f1)" "$(echo $PROXY_SERVER | cut -d: -f2)"
    sudo networksetup -setsecurewebproxystate "Wi-Fi" on
    
    # Configure SOCKS proxy (for applications that support it)
    sudo networksetup -setsocksfirewallproxy "Wi-Fi" "$(echo $PROXY_SERVER | cut -d: -f1)" "$(echo $PROXY_SERVER | cut -d: -f2)"
    sudo networksetup -setsocksfirewallproxystate "Wi-Fi" on
    
    # Set PAC file
    sudo networksetup -setautoproxyurl "Wi-Fi" "file://$PAC_FILE"
    sudo networksetup -setautoproxystate "Wi-Fi" on
    
    echo -e "${GREEN}✓ System proxy settings configured${NC}"
}

# Function to configure shell environment
configure_shell_proxy() {
    echo -e "${YELLOW}Configuring shell environment...${NC}"
    
    # Add proxy configuration to shell profiles
    SHELL_CONFIGS=("$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.bash_profile" "$HOME/.profile")
    
    for config_file in "${SHELL_CONFIGS[@]}"; do
        if [[ -f "$config_file" ]]; then
            # Check if proxy configuration already exists
            if ! grep -q "WDNS Proxy Configuration" "$config_file"; then
                echo "" >> "$config_file"
                echo "# WDNS Proxy Configuration" >> "$config_file"
                echo "if [[ -f \"$PROXY_CONFIG_FILE\" ]]; then" >> "$config_file"
                echo "    source \"$PROXY_CONFIG_FILE\"" >> "$config_file"
                echo "fi" >> "$config_file"
                echo -e "${GREEN}✓ Added proxy configuration to $config_file${NC}"
            else
                echo -e "${GREEN}✓ Proxy configuration already exists in $config_file${NC}"
            fi
        fi
    done
}

# Function to create launch agent for automatic proxy management
create_launch_agent() {
    echo -e "${YELLOW}Creating launch agent for automatic proxy management...${NC}"
    
    cat > "$LAUNCH_AGENT" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.wdns.proxy</string>
    <key>ProgramArguments</key>
    <array>
        <string>/bin/bash</string>
        <string>$PROXY_CONFIG_DIR/proxy-monitor.sh</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>$PROXY_CONFIG_DIR/proxy-monitor.log</string>
    <key>StandardErrorPath</key>
    <string>$PROXY_CONFIG_DIR/proxy-monitor.log</string>
</dict>
</plist>
EOF

    echo -e "${GREEN}✓ Created launch agent: $LAUNCH_AGENT${NC}"
}

# Function to create proxy monitor script
create_proxy_monitor() {
    echo -e "${YELLOW}Creating proxy monitor script...${NC}"
    
    cat > "$PROXY_CONFIG_DIR/proxy-monitor.sh" << 'EOF'
#!/bin/bash

# WDNS Proxy Monitor Script
# This script monitors the WDNS proxy server and reconfigures system proxy if needed

PROXY_SERVER="127.0.0.1:9701"
LOG_FILE="$HOME/.wdns-proxy/proxy-monitor.log"

log_message() {
    echo "[$(date)] $1" >> "$LOG_FILE"
}

check_proxy_server() {
    if curl -s --connect-timeout 5 "http://$PROXY_SERVER" > /dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

configure_proxy() {
    # Configure system proxy
    networksetup -setwebproxy "Wi-Fi" "127.0.0.1" "9701" > /dev/null 2>&1
    networksetup -setwebproxystate "Wi-Fi" on > /dev/null 2>&1
    networksetup -setsecurewebproxy "Wi-Fi" "127.0.0.1" "9701" > /dev/null 2>&1
    networksetup -setsecurewebproxystate "Wi-Fi" on > /dev/null 2>&1
}

disable_proxy() {
    # Disable system proxy
    networksetup -setwebproxystate "Wi-Fi" off > /dev/null 2>&1
    networksetup -setsecurewebproxystate "Wi-Fi" off > /dev/null 2>&1
}

# Main monitoring loop
while true; do
    if check_proxy_server; then
        configure_proxy
        log_message "Proxy server is running - system proxy enabled"
    else
        disable_proxy
        log_message "Proxy server is not running - system proxy disabled"
    fi
    
    sleep 30
done
EOF

    chmod +x "$PROXY_CONFIG_DIR/proxy-monitor.sh"
    echo -e "${GREEN}✓ Created proxy monitor script${NC}"
}

# Function to create browser configuration scripts
create_browser_configs() {
    echo -e "${YELLOW}Creating browser configuration scripts...${NC}"
    
    # Chrome configuration
    cat > "$PROXY_CONFIG_DIR/chrome-proxy.sh" << 'EOF'
#!/bin/bash
# Chrome proxy configuration

# Kill existing Chrome processes
pkill -f "Google Chrome" 2>/dev/null || true

# Start Chrome with proxy settings
open -a "Google Chrome" --args \
    --proxy-server="http://127.0.0.1:9701" \
    --proxy-pac-url="file://$HOME/.wdns-proxy/proxy.pac" \
    --disable-web-security \
    --disable-features=VizDisplayCompositor

echo "Chrome started with proxy configuration"
EOF

    # Firefox configuration
    cat > "$PROXY_CONFIG_DIR/firefox-proxy.sh" << 'EOF'
#!/bin/bash
# Firefox proxy configuration

# Create Firefox profile directory
FIREFOX_PROFILE_DIR="$HOME/.wdns-proxy/firefox-profile"
mkdir -p "$FIREFOX_PROFILE_DIR"

# Create user.js with proxy settings
cat > "$FIREFOX_PROFILE_DIR/user.js" << 'FIREFOX_EOF'
// WDNS Proxy Configuration for Firefox
user_pref("network.proxy.type", 1);
user_pref("network.proxy.http", "127.0.0.1");
user_pref("network.proxy.http_port", 9701);
user_pref("network.proxy.ssl", "127.0.0.1");
user_pref("network.proxy.ssl_port", 9701);
user_pref("network.proxy.share_proxy_settings", true);
user_pref("network.proxy.no_proxies_on", "localhost, 127.0.0.1");
FIREFOX_EOF

# Start Firefox with custom profile
/Applications/Firefox.app/Contents/MacOS/firefox -profile "$FIREFOX_PROFILE_DIR" &

echo "Firefox started with proxy configuration"
EOF

    chmod +x "$PROXY_CONFIG_DIR/chrome-proxy.sh"
    chmod +x "$PROXY_CONFIG_DIR/firefox-proxy.sh"
    
    echo -e "${GREEN}✓ Created browser configuration scripts${NC}"
}

# Function to create application launcher
create_app_launcher() {
    echo -e "${YELLOW}Creating application launcher...${NC}"
    
    cat > "$PROXY_CONFIG_DIR/start-proxy-apps.sh" << 'EOF'
#!/bin/bash
# Start applications with proxy configuration

PROXY_CONFIG_DIR="$HOME/.wdns-proxy"

# Source proxy environment variables
if [[ -f "$PROXY_CONFIG_DIR/config" ]]; then
    source "$PROXY_CONFIG_DIR/config"
fi

echo "Starting applications with proxy configuration..."

# Start Terminal with proxy environment
osascript << 'APPLESCRIPT_EOF'
tell application "Terminal"
    activate
    do script "echo 'Terminal started with proxy configuration' && env | grep -i proxy"
end tell
APPLESCRIPT_EOF

# Start Chrome with proxy
if [[ -f "$PROXY_CONFIG_DIR/chrome-proxy.sh" ]]; then
    bash "$PROXY_CONFIG_DIR/chrome-proxy.sh" &
fi

# Start Firefox with proxy
if [[ -f "$PROXY_CONFIG_DIR/firefox-proxy.sh" ]]; then
    bash "$PROXY_CONFIG_DIR/firefox-proxy.sh" &
fi

echo "Applications started with proxy configuration"
EOF

    chmod +x "$PROXY_CONFIG_DIR/start-proxy-apps.sh"
    echo -e "${GREEN}✓ Created application launcher${NC}"
}

# Function to create uninstall script
create_uninstall_script() {
    echo -e "${YELLOW}Creating uninstall script...${NC}"
    
    cat > "$PROXY_CONFIG_DIR/uninstall.sh" << 'EOF'
#!/bin/bash
# Uninstall WDNS proxy configuration

echo "Uninstalling WDNS proxy configuration..."

# Disable system proxy
networksetup -setwebproxystate "Wi-Fi" off
networksetup -setsecurewebproxystate "Wi-Fi" off
networksetup -setautoproxystate "Wi-Fi" off

# Unload launch agent
launchctl unload "$HOME/Library/LaunchAgents/com.wdns.proxy.plist" 2>/dev/null || true

# Remove launch agent
rm -f "$HOME/Library/LaunchAgents/com.wdns.proxy.plist"

# Remove configuration directory
rm -rf "$HOME/.wdns-proxy"

# Remove proxy configuration from shell profiles
SHELL_CONFIGS=("$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.bash_profile" "$HOME/.profile")

for config_file in "${SHELL_CONFIGS[@]}"; do
    if [[ -f "$config_file" ]]; then
        # Remove proxy configuration
        sed -i.bak '/# WDNS Proxy Configuration/,/^fi$/d' "$config_file"
        echo "Removed proxy configuration from $config_file"
    fi
done

echo "WDNS proxy configuration uninstalled"
EOF

    chmod +x "$PROXY_CONFIG_DIR/uninstall.sh"
    echo -e "${GREEN}✓ Created uninstall script${NC}"
}

# Function to test proxy configuration
test_proxy_configuration() {
    echo -e "${YELLOW}Testing proxy configuration...${NC}"
    
    # Test if proxy server is running
    if curl -s --connect-timeout 5 "http://$PROXY_SERVER" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Proxy server is running${NC}"
    else
        echo -e "${RED}✗ Proxy server is not running${NC}"
        echo "Please start the WDNS service first:"
        echo "  ./target/release/wdns-service"
        return 1
    fi
    
    # Test HTTP request through proxy
    if curl -s --proxy "http://$PROXY_SERVER" --connect-timeout 10 "http://httpbin.org/ip" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ HTTP proxy is working${NC}"
    else
        echo -e "${RED}✗ HTTP proxy test failed${NC}"
    fi
    
    # Test HTTPS request through proxy
    if curl -s --proxy "http://$PROXY_SERVER" --connect-timeout 10 "https://httpbin.org/ip" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ HTTPS proxy is working${NC}"
    else
        echo -e "${RED}✗ HTTPS proxy test failed${NC}"
    fi
}

# Function to show usage information
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  -s, --server   Set proxy server address (default: 127.0.0.1:9701)"
    echo "  -t, --test     Test proxy configuration only"
    echo "  -u, --uninstall Uninstall proxy configuration"
    echo ""
    echo "Environment Variables:"
    echo "  WDNS_PROXY_SERVER    Set proxy server address (default: 127.0.0.1:9701)"
    echo "  WDNS_DNS_SERVER      Set DNS server address (default: 127.0.0.1:9700)"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Setup with default server"
    echo "  $0 -s 192.168.1.100:9701            # Setup with specific server"
    echo "  $0 -t                                # Test configuration only"
    echo "  $0 -u                                # Uninstall configuration"
    echo ""
    echo "After setup, you can:"
    echo "  - Start applications: $PROXY_CONFIG_DIR/start-proxy-apps.sh"
    echo "  - Uninstall: $PROXY_CONFIG_DIR/uninstall.sh"
}

# Function to uninstall proxy configuration
uninstall_proxy() {
    echo -e "${YELLOW}Uninstalling WDNS proxy configuration...${NC}"
    
    if [[ -f "$PROXY_CONFIG_DIR/uninstall.sh" ]]; then
        bash "$PROXY_CONFIG_DIR/uninstall.sh"
    else
        echo -e "${RED}Uninstall script not found${NC}"
        exit 1
    fi
}

# Main setup function
main() {
    echo -e "${BLUE}Setting up WDNS proxy for macOS...${NC}"
    echo ""
    
    # Check if running as root
    check_root
    
    # Setup configuration directory
    setup_config_dir
    
    # Create configuration files
    create_proxy_config
    create_pac_file
    
    # Configure system settings
    configure_system_proxy
    
    # Configure shell environment
    configure_shell_proxy
    
    # Create monitoring and management scripts
    create_launch_agent
    create_proxy_monitor
    create_browser_configs
    create_app_launcher
    create_uninstall_script
    
    # Load launch agent
    launchctl load "$LAUNCH_AGENT" 2>/dev/null || true
    
    echo ""
    echo -e "${GREEN}✓ WDNS proxy configuration completed!${NC}"
    echo ""
    echo "Configuration files created in: $PROXY_CONFIG_DIR"
    echo ""
    echo "To start applications with proxy:"
    echo "  $PROXY_CONFIG_DIR/start-proxy-apps.sh"
    echo ""
    echo "To uninstall:"
    echo "  $PROXY_CONFIG_DIR/uninstall.sh"
    echo ""
    echo "To test configuration:"
    echo "  $0 -t"
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
        -t|--test)
            test_proxy_configuration
            exit 0
            ;;
        -u|--uninstall)
            uninstall_proxy
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Run the setup
main
