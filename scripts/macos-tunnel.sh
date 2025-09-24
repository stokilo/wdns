#!/bin/bash

set -e

# Configuration
SSH_USER="a0104757"
SSH_HOST="192.168.0.115"
SSH_PORT="23"
SSH_TUNNEL_PORT="1080"
WDNS_SERVER="127.0.0.1:9700"
UPDATE_INTERVAL="300"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PID_FILE="/tmp/wdns-tunnel.pid"
LOG_FILE="/tmp/wdns-tunnel.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    printf "${BLUE}[INFO]${NC} %s\n" "$1"
}

print_success() {
    printf "${GREEN}[SUCCESS]${NC} %s\n" "$1"
}

print_warning() {
    printf "${YELLOW}[WARNING]${NC} %s\n" "$1"
}

print_error() {
    printf "${RED}[ERROR]${NC} %s\n" "$1"
}

# Function to check if SSH is available
check_ssh() {
    if ! command -v ssh &> /dev/null; then
        print_error "SSH is required but not installed"
        exit 1
    fi
}

# Function to check if expect is available
check_expect() {
    if ! command -v expect &> /dev/null; then
        print_error "expect is required for automated password handling"
        print_status "Install it with: brew install expect"
        exit 1
    fi
}

# Function to check if tunnel is running
is_tunnel_running() {
    if [[ -f "$PID_FILE" ]]; then
        local pid=$(cat "$PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            return 0
        else
            rm -f "$PID_FILE"
            return 1
        fi
    fi
    return 1
}

# Function to prompt for SSH password
prompt_ssh_password() {
    if [[ -n "$SSH_PASSWORD" ]]; then
        print_status "Using SSH_PASSWORD environment variable"
        return 0
    fi
    
    print_status "SSH password required for $SSH_USER@$SSH_HOST:$SSH_PORT"
    read -s -p "Enter SSH password: " SSH_PASSWORD
    echo  # New line after password input
    print_success "Password stored in memory"
}

# Function to start SSH tunnel
start_tunnel() {
    if is_tunnel_running; then
        print_warning "Tunnel is already running (PID: $(cat "$PID_FILE"))"
        return 0
    fi

    print_status "Starting SSH tunnel to $SSH_USER@$SSH_HOST:$SSH_PORT..."
    print_status "SOCKS4 proxy will be available on port $SSH_TUNNEL_PORT"
    
    # Prompt for password if not already set
    prompt_ssh_password
    
    # Create expect script for password authentication
    local expect_script=$(mktemp)
    cat > "$expect_script" << EOF
#!/usr/bin/expect -f
set timeout 30
spawn ssh -D $SSH_TUNNEL_PORT -p $SSH_PORT -N -f $SSH_USER@$SSH_HOST
expect "password:"
send "$SSH_PASSWORD\r"
expect {
    "Connection established" { }
    "Tunnel established" { }
    timeout { exit 1 }
}
# Don't wait for eof - let SSH run in background
EOF
    chmod +x "$expect_script"
    
    print_status "Starting SSH tunnel with password authentication..."
    
    # Run expect script and capture output
    local expect_output=$(expect -f "$expect_script" 2>&1)
    local expect_exit_code=$?
    
    # Clean up expect script
    rm -f "$expect_script"
    
    if [[ $expect_exit_code -ne 0 ]]; then
        print_error "Failed to start SSH tunnel"
        print_error "Expect output: $expect_output"
        print_error "Check your SSH credentials and connection"
        exit 1
    fi
    
    # Wait a moment for SSH to establish
    sleep 3
    
    # Find the SSH process PID
    local ssh_pid=$(pgrep -f "ssh -D $SSH_TUNNEL_PORT")
    
    if [[ -n "$ssh_pid" ]]; then
        echo "$ssh_pid" > "$PID_FILE"
        print_success "SSH tunnel started successfully (PID: $ssh_pid)"
        print_status "SOCKS4 proxy available at 127.0.0.1:$SSH_TUNNEL_PORT"
        print_status "Tunnel will continue running in background"
    else
        print_error "SSH tunnel process not found after startup"
        print_error "Check if SSH connection was successful"
        exit 1
    fi
}

# Function to stop SSH tunnel
stop_tunnel() {
    if ! is_tunnel_running; then
        print_warning "Tunnel is not running"
        return 0
    fi

    local pid=$(cat "$PID_FILE")
    print_status "Stopping SSH tunnel (PID: $pid)..."
    
    if kill "$pid" 2>/dev/null; then
        # Wait for process to terminate
        local count=0
        while kill -0 "$pid" 2>/dev/null && [[ $count -lt 10 ]]; do
            sleep 1
            ((count++))
        done
        
        if kill -0 "$pid" 2>/dev/null; then
            print_warning "Force killing tunnel process..."
            kill -9 "$pid" 2>/dev/null
        fi
        
        rm -f "$PID_FILE"
        print_success "SSH tunnel stopped"
    else
        print_error "Failed to stop tunnel (PID: $pid)"
        exit 1
    fi
}

# Function to update DNS through tunnel
update_dns() {
    print_status "Updating DNS through tunnel..."
    
    # Set proxy environment for curl
    export http_proxy="socks4://127.0.0.1:$SSH_TUNNEL_PORT"
    export https_proxy="socks4://127.0.0.1:$SSH_TUNNEL_PORT"
    
    # Run the update script with tunnel
    if [[ -f "$SCRIPT_DIR/update-hosts.sh" ]]; then
        WDNS_SERVER="$WDNS_SERVER" "$SCRIPT_DIR/update-hosts.sh"
    else
        print_error "update-hosts.sh not found in $SCRIPT_DIR"
        return 1
    fi
    
    # Unset proxy
    unset http_proxy
    unset https_proxy
}

# Function to run periodic updates
run_periodic_updates() {
    print_status "Starting periodic DNS updates (every ${UPDATE_INTERVAL}s)..."
    print_status "Press Ctrl+C to stop"
    
    while true; do
        if is_tunnel_running; then
            update_dns
        else
            print_warning "Tunnel is not running, attempting to restart..."
            start_tunnel
            if is_tunnel_running; then
                update_dns
            else
                print_error "Failed to restart tunnel, waiting before retry..."
            fi
        fi
        
        print_status "Waiting ${UPDATE_INTERVAL} seconds before next update..."
        sleep "$UPDATE_INTERVAL"
    done
}

# Function to test tunnel connectivity
test_tunnel() {
    if ! is_tunnel_running; then
        print_error "Tunnel is not running"
        return 1
    fi

    print_status "Testing tunnel connectivity..."
    
    # Set proxy environment
    export http_proxy="socks4://127.0.0.1:$SSH_TUNNEL_PORT"
    export https_proxy="socks4://127.0.0.1:$SSH_TUNNEL_PORT"
    
    # Test WDNS service through tunnel
    if curl -s --connect-timeout 10 "http://$WDNS_SERVER/health" > /dev/null; then
        print_success "Tunnel connectivity test passed"
        print_status "WDNS service is reachable through tunnel"
    else
        print_error "Tunnel connectivity test failed"
        print_error "WDNS service is not reachable through tunnel"
    fi
    
    # Unset proxy
    unset http_proxy
    unset https_proxy
}

# Function to show status
show_status() {
    print_status "WDNS Tunnel Status"
    print_status "=================="
    
    if is_tunnel_running; then
        local pid=$(cat "$PID_FILE")
        print_success "Tunnel is running (PID: $pid)"
        print_status "SOCKS4 proxy: 127.0.0.1:$SSH_TUNNEL_PORT"
        print_status "WDNS server: $WDNS_SERVER"
        print_status "Tunnel will continue running until stopped"
        
        # Test connectivity
        test_tunnel
    else
        print_warning "Tunnel is not running"
        print_status "Start it with: $0 start"
    fi
}

# Function to show help
show_help() {
    echo "WDNS macOS Tunnel Management Script"
    echo "==================================="
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  start       Start SSH tunnel"
    echo "  stop        Stop SSH tunnel"
    echo "  restart     Restart SSH tunnel"
    echo "  status      Show tunnel status"
    echo "  test        Test tunnel connectivity"
    echo "  update      Run DNS update once"
    echo "  periodic    Run periodic DNS updates"
    echo "  interactive Start tunnel interactively (for password prompt)"
    echo "  help        Show this help message"
    echo ""
    echo "Configuration:"
    echo "  SSH_USER: $SSH_USER"
    echo "  SSH_HOST: $SSH_HOST"
    echo "  SSH_PORT: $SSH_PORT"
    echo "  TUNNEL_PORT: $SSH_TUNNEL_PORT"
    echo "  WDNS_SERVER: $WDNS_SERVER"
    echo "  UPDATE_INTERVAL: ${UPDATE_INTERVAL}s"
    echo ""
    echo "Password Authentication:"
    echo "  The script will prompt for your SSH password when needed"
    echo "  You can also set SSH_PASSWORD environment variable:"
    echo "    export SSH_PASSWORD='your_password'"
    echo "  Note: expect is required for automated password handling"
    echo "    brew install expect"
    echo ""
    echo "Files:"
    echo "  PID_FILE: $PID_FILE"
    echo "  LOG_FILE: $LOG_FILE"
}

# Main function
main() {
    # Pre-flight checks
    check_ssh
    
    # Skip expect check for interactive mode
    if [[ "${1:-help}" != "interactive" ]]; then
        check_expect
    fi
    
    case "${1:-help}" in
        start)
            start_tunnel
            ;;
        stop)
            stop_tunnel
            ;;
        restart)
            stop_tunnel
            sleep 2
            start_tunnel
            ;;
        status)
            show_status
            ;;
        test)
            test_tunnel
            ;;
        update)
            if is_tunnel_running; then
                update_dns
            else
                print_error "Tunnel is not running. Start it first with: $0 start"
                exit 1
            fi
            ;;
        periodic)
            if ! is_tunnel_running; then
                print_status "Starting tunnel for periodic updates..."
                start_tunnel
            fi
            run_periodic_updates
            ;;
        interactive)
            print_status "Starting SSH tunnel interactively..."
            print_warning "You will be prompted for the SSH password"
            print_status "The tunnel will run in the foreground - press Ctrl+C to stop"
            print_status "SSH command: ssh -D $SSH_TUNNEL_PORT -p $SSH_PORT -N $SSH_USER@$SSH_HOST"
            print_status "Logging to: $LOG_FILE"
            
            # Run SSH interactively with logging
            ssh -D "$SSH_TUNNEL_PORT" -p "$SSH_PORT" -N "$SSH_USER@$SSH_HOST" 2>&1 | tee "$LOG_FILE"
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            print_error "Unknown command: $1"
            show_help
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
