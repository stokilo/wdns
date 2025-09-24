# WDNS Service

# Update MacOS /etc/hosts

```shell
sudo sh scripts/update-hosts.sh
```

A high-performance Windows DNS resolution service built with Rust and Tokio. Provides HTTP API for concurrent DNS resolution and HTTP proxy server with Windows service support.

## Features

- 🚀 **High Performance**: Built with Rust and Tokio for maximum speed
- 🔄 **Concurrent Resolution**: Resolves multiple hosts in parallel
- 🪟 **Windows Service**: Runs as a native Windows service
- 🌐 **HTTP API**: RESTful API for DNS resolution
- 🔗 **HTTP Proxy**: Built-in HTTP proxy server for traffic tunneling
- ⚡ **Async**: Non-blocking I/O with Tokio
- 📊 **Health Checks**: Built-in health monitoring endpoints

## API Endpoints

### Health Check
```
GET /health
```
Returns service health status.

### DNS Resolution
```
POST /api/dns/resolve
Content-Type: application/json

{
  "hosts": ["google.com", "github.com", "stackoverflow.com"]
}
```

**Response:**
```json
{
  "results": [
    {
      "host": "google.com",
      "ip_addresses": ["142.250.191.14", "2607:f8b0:4005:80b::200e"],
      "status": "success",
      "error": null
    },
    {
      "host": "github.com",
      "ip_addresses": ["140.82.112.3"],
      "status": "success",
      "error": null
    }
  ],
  "total_resolved": 2,
  "total_errors": 0
}
```

## Prerequisites

- **Rust 1.70+** with `stable` toolchain
- **Windows 10/11** or **Windows Server 2016+** (for Windows service)
- **Administrator privileges** (for service installation)
- **Cross-platform**: Also works on macOS and Linux for development

## Building

### 1. Install Rust

If you don't have Rust installed:

```powershell
# Install Rust via rustup
Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "rustup-init.exe"
.\rustup-init.exe -y
```

### 2. Build the Project

```bash
# Clone and build
git clone <your-repo-url>
cd wdns
cargo build --release
```

The executable will be created at:
- **Windows**: `target/release/wdns-service.exe`
- **macOS/Linux**: `target/release/wdns-service`

## Running

### Standalone Mode

Run the service directly for testing:

```bash
# Windows
.\target\release\wdns-service.exe

# macOS/Linux
./target/release/wdns-service

# Or with custom config (if implemented)
./target/release/wdns-service --config custom-config.json
```

The service will start on:
- **DNS Service**: `http://0.0.0.0:9700` (HTTP API)
- **Proxy Server**: `http://0.0.0.0:9701` (HTTP Proxy)

Both services listen on all interfaces by default.

### Windows Service Mode

#### Install the Service

```powershell
# Run as Administrator
sc.exe create "WDNSService" binPath="C:\path\to\wdns-service.exe --service" start=auto
sc.exe description "WDNSService" "Windows DNS Resolution Service"
```

#### Start the Service

```powershell
sc.exe start "WDNSService"
```

#### Stop the Service

```powershell
sc.exe stop "WDNSService"
```

#### Uninstall the Service

```powershell
sc.exe delete "WDNSService"
```

## Configuration

The service creates a `config.json` file on first run:

```json
{
  "bind_address": "0.0.0.0:9700",
  "dns_timeout_seconds": 10,
  "max_concurrent_resolutions": 100,
  "proxy_enabled": true,
  "proxy_bind_address": "0.0.0.0:9701"
}
```

### Configuration Options

- `bind_address`: IP address and port to bind the DNS HTTP server
- `dns_timeout_seconds`: Timeout for DNS resolution in seconds
- `max_concurrent_resolutions`: Maximum number of concurrent DNS resolutions
- `proxy_enabled`: Enable/disable the HTTP proxy server
- `proxy_bind_address`: IP address and port to bind the proxy server

## HTTP Proxy Server

The service includes a built-in HTTP proxy server that can tunnel traffic through port 9701.

### Proxy Features

- **HTTP Proxy**: Standard HTTP proxy functionality
- **HTTPS Tunneling**: CONNECT method support for HTTPS traffic
- **Traffic Forwarding**: Transparent forwarding of HTTP requests
- **Concurrent Handling**: Multiple simultaneous connections
- **Configurable**: Can be enabled/disabled via configuration

### Using the Proxy

#### Configure Client to Use Proxy

**Windows (PowerShell/Internet Explorer):**
```powershell
# Set proxy for current session
$env:HTTP_PROXY = "http://127.0.0.1:9701"
$env:HTTPS_PROXY = "http://127.0.0.1:9701"

# Or configure system-wide
netsh winhttp set proxy proxy-server=127.0.0.1:9701
```

**macOS/Linux:**
```bash
# Set environment variables
export HTTP_PROXY=http://127.0.0.1:9701
export HTTPS_PROXY=http://127.0.0.1:9701

# Or configure for specific applications
curl --proxy http://127.0.0.1:9701 https://example.com
```

**Browser Configuration:**
- **Chrome**: `--proxy-server=http://127.0.0.1:9701`
- **Firefox**: Manual proxy configuration in Network Settings
- **Edge**: Proxy settings in System Settings

#### Test Proxy Functionality

```bash
# Test HTTP request through proxy
curl --proxy http://127.0.0.1:9701 http://httpbin.org/ip

# Test HTTPS request through proxy
curl --proxy http://127.0.0.1:9701 https://httpbin.org/ip

# Test with environment variables
HTTP_PROXY=http://127.0.0.1:9701 curl http://httpbin.org/ip
```

### Proxy Configuration

The proxy server can be configured in `config.json`:

```json
{
  "proxy_enabled": true,
  "proxy_bind_address": "0.0.0.0:9701"
}
```

To disable the proxy server:
```json
{
  "proxy_enabled": false,
  "proxy_bind_address": "0.0.0.0:9701"
}
```

## Testing the Service

### Using PowerShell

```powershell
# Health check (from local machine)
Invoke-RestMethod -Uri "http://127.0.0.1:9700/health"

# Health check (from remote machine)
Invoke-RestMethod -Uri "http://192.168.0.115:9700/health"

# DNS resolution (from local machine)
$body = @{
    hosts = @("google.com", "github.com", "stackoverflow.com")
} | ConvertTo-Json

Invoke-RestMethod -Uri "http://127.0.0.1:9700/api/dns/resolve" -Method POST -Body $body -ContentType "application/json"

# DNS resolution (from remote machine)
Invoke-RestMethod -Uri "http://192.168.0.115:9700/api/dns/resolve" -Method POST -Body $body -ContentType "application/json"
```

### Using curl

```bash
# Health check (from local machine)
curl http://127.0.0.1:9700/health

# Health check (from remote machine)
curl http://192.168.0.115:9700/health

# DNS resolution (from local machine)
curl -X POST http://127.0.0.1:9700/api/dns/resolve \
  -H "Content-Type: application/json" \
  -d '{"hosts": ["google.com", "github.com"]}'

# DNS resolution (from remote machine)
curl -X POST http://192.168.0.115:9700/api/dns/resolve \
  -H "Content-Type: application/json" \
  -d '{"hosts": ["google.com", "github.com"]}'

# Test proxy server (from local machine)
curl --proxy http://127.0.0.1:9701 http://httpbin.org/ip

# Test proxy server (from remote machine)
curl --proxy http://192.168.0.115:9701 http://httpbin.org/ip
```

## Network Access

### Remote Access Configuration

The service is configured to listen on all interfaces (`0.0.0.0:9700`) by default, allowing remote access from other machines on your network.

#### Windows Firewall Configuration

```powershell
# Allow inbound connections on DNS service port 9700
New-NetFirewallRule -DisplayName "WDNS DNS Service" -Direction Inbound -Protocol TCP -LocalPort 9700 -Action Allow

# Allow inbound connections on proxy port 9701
New-NetFirewallRule -DisplayName "WDNS Proxy Service" -Direction Inbound -Protocol TCP -LocalPort 9701 -Action Allow

# Or using netsh (run as Administrator)
netsh advfirewall firewall add rule name="WDNS DNS Service" dir=in action=allow protocol=TCP localport=9700
netsh advfirewall firewall add rule name="WDNS Proxy Service" dir=in action=allow protocol=TCP localport=9701
```

#### Testing Remote Access

```powershell
# From another machine on the network
# Replace 192.168.0.115 with your server's IP address

# Health check
curl http://192.168.0.115:9700/health

# DNS resolution
curl -X POST http://192.168.0.115:9700/api/dns/resolve \
  -H "Content-Type: application/json" \
  -d '{"hosts": ["linde-uat.cprt.kion.cloud", "login-uat.kion.cloud", "networkpartner-kion-uat.cprt.kion.cloud", "networkpartner-kion-dev.cprt.kion.cloud", "kimdev.kiongroup.net", "linde-dev.cprt.kion.cloud"]}' | jq
```



#### Security Considerations

- **Firewall**: Only open ports 9700 and 9701 if you need remote access
- **Network Security**: Consider using VPN or private networks
- **Authentication**: The current implementation has no authentication
- **HTTPS**: Consider adding TLS encryption for production use
- **Proxy Security**: The proxy server forwards all traffic without filtering
- **Access Control**: Consider implementing IP whitelisting for production use

## Logging

The service uses structured logging with the `tracing` crate. Set the log level using the `RUST_LOG` environment variable:

```powershell
# Set log level
$env:RUST_LOG="wdns=debug,hyper=info"
.\target\release\wdns-service.exe
```

## Performance

- **Concurrent Resolution**: Resolves multiple hosts in parallel
- **Memory Efficient**: Uses async I/O to minimize memory usage
- **Fast**: Rust + Tokio provides excellent performance
- **Scalable**: Handles hundreds of concurrent requests

## Troubleshooting

### Service Won't Start

1. Check if the port is already in use:
   ```powershell
   netstat -an | findstr :9700
   ```

2. Check Windows Event Log for service errors:
   ```powershell
   Get-WinEvent -LogName Application | Where-Object {$_.ProviderName -eq "WDNSService"}
   ```

### DNS Resolution Issues

1. Check your DNS configuration:
   ```powershell
   nslookup google.com
   ```

2. Verify the service is running:
   ```powershell
   sc.exe query "WDNSService"
   ```

## macOS Proxy Setup

The WDNS service includes comprehensive macOS proxy setup scripts that automatically configure your system to route all traffic through the proxy server.

### Quick Setup

```bash
# 1. Start the WDNS service
./target/release/wdns-service

# 2. Enable proxy for all applications
./scripts/proxy on

# 3. Test the proxy configuration
./scripts/proxy test

# 4. Start applications with proxy
./scripts/proxy apps
```

### Available Scripts

- **`./scripts/proxy`** - Simple manager for common operations
- **`./scripts/macos-quick-proxy.sh`** - Quick proxy setup
- **`./scripts/macos-proxy-setup.sh`** - Full proxy management system
- **`./scripts/README-macos-proxy.md`** - Detailed macOS proxy documentation

### What It Does

- **System Proxy**: Configures macOS system proxy settings
- **Environment Variables**: Sets proxy environment variables for terminal applications
- **Browser Configuration**: Creates browser profiles with proxy settings
- **Automatic Management**: Monitors proxy server and reconfigures as needed

### Usage Examples

```bash
# Enable proxy
./scripts/proxy on

# Test proxy
./scripts/proxy test

# Start applications
./scripts/proxy apps

# Check status
./scripts/proxy status

# Disable proxy
./scripts/proxy off
```

For detailed information, see [macOS Proxy Setup Documentation](scripts/README-macos-proxy.md).

## Host Update Scripts

### Update /etc/hosts (Linux/macOS)

The `update-hosts.sh` script calls the WDNS service and updates `/etc/hosts` with resolved IP addresses:

```bash
# Make executable
chmod +x scripts/update-hosts.sh

# Run as root (required to modify /etc/hosts)
sudo ./scripts/update-hosts.sh

# Use different WDNS server
sudo ./scripts/update-hosts.sh 192.168.1.100:9700

# Or set environment variable
WDNS_SERVER="192.168.1.100:9700" sudo ./scripts/update-hosts.sh

# Test the service first
./scripts/test-wdns.sh

# Diagnose connection issues
./scripts/diagnose.sh

# Diagnose specific server
WDNS_SERVER="192.168.1.100:9700" ./scripts/diagnose.sh
```

**Features:**
- Calls WDNS service to resolve hostnames
- Creates backup of existing hosts file
- Removes old entries for the same hosts
- Adds new entries with resolved IP addresses
- Colored output for better visibility

### Update C:\Windows\System32\drivers\etc\hosts (Windows)

The `update-hosts.ps1` script does the same for Windows:

```powershell
# Run as Administrator
.\scripts\update-hosts.ps1

# Test what would be changed (dry run)
.\scripts\update-hosts.ps1 -WhatIf

# Use different WDNS server
.\scripts\update-hosts.ps1 -WdnsServer "192.168.1.100:9700"
```

**Features:**
- PowerShell script for Windows
- Administrator privileges required
- Backup creation before changes
- WhatIf mode for testing
- Configurable WDNS server

### Test Script

Test the WDNS service with the configured hosts:

```bash
# Linux/macOS
./scripts/test-wdns.sh

# Windows PowerShell
.\scripts\test-wdns.ps1
```

## Development

### Running in Development Mode

```powershell
# Set debug logging
$env:RUST_LOG="wdns=debug"
cargo run
```

### Building for Production

```powershell
# Optimized release build
cargo build --release

# Strip debug symbols (optional)
strip target/release/wdns-service.exe
```

## License

MIT License - see LICENSE file for details.
