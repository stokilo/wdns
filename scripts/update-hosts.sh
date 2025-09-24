#!/bin/bash

# WDNS Service Hosts Update Script
# This script calls the WDNS service and updates /etc/hosts with resolved IP addresses

set -e  # Exit on any error

# Configuration
WDNS_SERVER="${WDNS_SERVER:-192.168.0.115:9700}"

# Allow command line override
if [[ $# -gt 0 ]]; then
    if [[ "$1" == "--cleanup" ]]; then
        # Cleanup mode - just remove existing WDNS entries
        print_status "WDNS Service Cleanup Mode"
        print_status "========================="
        check_root
        backup_hosts_file
        cleanup_wdns_entries
        print_success "Cleanup completed!"
        exit 0
    else
        WDNS_SERVER="$1"
    fi
fi
HOSTS_FILE="/etc/hosts"
BACKUP_FILE="/etc/hosts.backup.$(date +%Y%m%d_%H%M%S)"

# Hosts to resolve
HOSTS=(
    "linde-uat.cprt.kion.cloud"
    "login-uat.kprt.kion.cloud"
    "networkpartner-kion-uat.cprt.kion.cloud"
    "networkpartner-kion-dev.cprt.kion.cloud"
    "kimdev.kiongroup.net"
    "linde-dev.cprt.kion.cloud"
)

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

# Function to check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        print_error "This script must be run as root (use sudo)"
        exit 1
    fi
}

# Function to check if jq is installed
check_jq() {
    if ! command -v jq &> /dev/null; then
        print_error "jq is required but not installed. Please install it:"
        print_status "  Ubuntu/Debian: sudo apt-get install jq"
        print_status "  CentOS/RHEL: sudo yum install jq"
        print_status "  macOS: brew install jq"
        exit 1
    fi
}

# Function to check if curl is available
check_curl() {
    if ! command -v curl &> /dev/null; then
        print_error "curl is required but not installed"
        exit 1
    fi
}

# Function to call WDNS service
call_wdns_service() {
    local hosts_json=$(printf '%s\n' "${HOSTS[@]}" | jq -R . | jq -s .)
    local payload="{\"hosts\": $hosts_json}"
    
    print_status "Calling WDNS service at $WDNS_SERVER..." >&2
    print_status "Resolving hosts: ${HOSTS[*]}" >&2
    
    # First test if the service is reachable
    print_status "Testing connectivity to $WDNS_SERVER..." >&2
    local health_response=$(curl -s --connect-timeout 5 "http://$WDNS_SERVER/health" 2>/dev/null)
    if [[ $? -ne 0 || -z "$health_response" ]]; then
        print_error "Cannot reach WDNS service at $WDNS_SERVER"
        print_error "Please ensure the service is running and accessible"
        print_error "You can test manually with: curl http://$WDNS_SERVER/health"
        exit 1
    fi
    
    # Validate health response
    if ! echo "$health_response" | jq . > /dev/null 2>&1; then
        print_error "WDNS service health check returned invalid response: $health_response"
        exit 1
    fi
    
    print_status "Service is reachable, calling DNS resolution endpoint..." >&2
    
    local response=$(curl -s -X POST "http://$WDNS_SERVER/api/dns/resolve" \
        -H "Content-Type: application/json" \
        -d "$payload" \
        -w "\nHTTP_CODE:%{http_code}")
    
    local http_code=$(echo "$response" | grep "HTTP_CODE:" | cut -d: -f2)
    local json_response=$(echo "$response" | sed '/HTTP_CODE:/d')
    
    
    
    if [[ "$http_code" != "200" ]]; then
        print_error "WDNS service returned HTTP $http_code"
        print_error "Response: $json_response"
        exit 1
    fi
    
    # Validate JSON response
    print_status "Testing JSON validity..." >&2
    local json_test=$(echo "$json_response" | jq . 2>&1)
    if [[ $? -ne 0 ]]; then
        print_error "Invalid JSON response from WDNS service"
        print_error "jq error: $json_test"
        print_error "Raw JSON response: $json_response"
        print_error "This might indicate:"
        print_error "  1. The service is not running properly"
        print_error "  2. The service returned an error page"
        print_error "  3. Network connectivity issues"
        print_error "Try testing manually: curl -X POST http://$WDNS_SERVER/api/dns/resolve -H 'Content-Type: application/json' -d '{\"hosts\": [\"test.example.com\"]}'"
        exit 1
    fi
    
    # Return only the JSON response
    echo "$json_response"
}

# Function to parse DNS response and extract IPs
parse_dns_response() {
    local response="$1"
    local temp_file=$(mktemp)
    
    echo "$response" | jq -r '.results[] | select(.status == "success") | "\(.host) \(.ip_addresses[0])"' > "$temp_file"
    
    if [[ ! -s "$temp_file" ]]; then
        print_error "No successful DNS resolutions found"
        rm -f "$temp_file"
        exit 1
    fi
    
    cat "$temp_file"
    rm -f "$temp_file"
}

# Function to backup hosts file
backup_hosts_file() {
    print_status "Creating backup of $HOSTS_FILE..."
    cp "$HOSTS_FILE" "$BACKUP_FILE"
    print_success "Backup created: $BACKUP_FILE"
}

# Function to clean up existing WDNS entries
cleanup_wdns_entries() {
    local temp_file=$(mktemp)
    
    print_status "Cleaning up existing WDNS entries..."
    
    # Remove all WDNS-related entries
    grep -v -E "(# WDNS Service entries|$(printf '%s|' "${HOSTS[@]}" | sed 's/|$//'))" "$HOSTS_FILE" > "$temp_file"
    
    # Remove empty lines
    sed -i '' '/^[[:space:]]*$/d' "$temp_file" 2>/dev/null || sed -i '/^[[:space:]]*$/d' "$temp_file"
    
    # Replace the hosts file
    mv "$temp_file" "$HOSTS_FILE"
    print_success "Cleaned up existing WDNS entries"
}

# Function to update hosts file
update_hosts_file() {
    local dns_entries="$1"
    local temp_file=$(mktemp)
    
    print_status "Updating $HOSTS_FILE..."
    
    # Remove existing WDNS entries (both comment lines and host entries)
    # This removes lines that contain "# WDNS Service entries" or any of our hostnames
    grep -v -E "(# WDNS Service entries|$(printf '%s|' "${HOSTS[@]}" | sed 's/|$//'))" "$HOSTS_FILE" > "$temp_file"
    
    # Also remove any empty lines that might be left behind
    sed -i '' '/^[[:space:]]*$/d' "$temp_file" 2>/dev/null || sed -i '/^[[:space:]]*$/d' "$temp_file"
    
    # Add new entries
    echo "" >> "$temp_file"
    echo "# WDNS Service entries - Updated $(date)" >> "$temp_file"
    echo "$dns_entries" | while read -r host ip; do
        if [[ -n "$host" && -n "$ip" ]]; then
            echo "$ip $host" >> "$temp_file"
            print_success "Added: $ip $host"
        fi
    done
    
    # Replace hosts file
    mv "$temp_file" "$HOSTS_FILE"
    print_success "Hosts file updated successfully"
}

# Function to display results
display_results() {
    local response="$1"
    
    
    print_status "DNS Resolution Results:"
    local jq_output=$(echo "$response" | jq -r '.results[] | "\(.host): \(.status) - \(.ip_addresses | join(", "))"' 2>&1)
    if [[ $? -ne 0 ]]; then
        print_error "jq parsing failed in display_results: $jq_output"
        print_error "Response was: $response"
        return 1
    fi
    echo "$jq_output"
    
    local total_resolved=$(echo "$response" | jq -r '.total_resolved' 2>/dev/null)
    local total_errors=$(echo "$response" | jq -r '.total_errors' 2>/dev/null)
    
    if [[ -n "$total_resolved" && -n "$total_errors" ]]; then
        print_status "Summary: $total_resolved resolved, $total_errors errors"
    else
        print_warning "Could not parse summary from response"
    fi
}

# Main function
main() {
    print_status "WDNS Service Hosts Update Script"
    print_status "================================="
    
    # Pre-flight checks
    check_root
    check_curl
    check_jq
    
    # Call WDNS service
    local response=$(call_wdns_service)
    
    # Display results
    display_results "$response"
    
    # Parse successful resolutions
    local dns_entries=$(parse_dns_response "$response")
    
    # Backup, cleanup, and update hosts file
    backup_hosts_file
    cleanup_wdns_entries
    update_hosts_file "$dns_entries"
    
    print_success "Hosts file update completed!"
    print_status "You can restore the backup with: cp $BACKUP_FILE $HOSTS_FILE"
}

# Run main function
main "$@"
