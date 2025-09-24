# WDNS macOS Proxy Setup

This guide explains how to configure your macOS system to use the WDNS proxy server (port 9701) for all applications including browsers, terminal, and other network applications.

## Quick Start

### 1. Start the WDNS Service

First, make sure the WDNS service is running:

```bash
# Build the service (if not already built)
cargo build --release

# Start the service
./target/release/wdns-service
```

The service will start both:
- DNS service on port 9700
- Proxy server on port 9701

### 2. Configure macOS Proxy

Use the simple proxy manager:

```bash
# Enable proxy for all applications
./scripts/proxy on

# Test the proxy configuration
./scripts/proxy test

# Start applications with proxy
./scripts/proxy apps

# Check proxy status
./scripts/proxy status

# Disable proxy when done
./scripts/proxy off
```

## Detailed Setup Options

### Option 1: Quick Setup (Recommended)

For most users, the quick setup is sufficient:

```bash
# Enable proxy configuration
./scripts/macos-quick-proxy.sh -e

# Test proxy
./scripts/macos-quick-proxy.sh -t

# Start applications
./scripts/macos-quick-proxy.sh -a
```

### Option 2: Full Setup

For advanced users who want complete control:

```bash
# Run the comprehensive setup
./scripts/macos-proxy-setup.sh

# This will:
# - Configure system proxy settings
# - Set up environment variables
# - Create browser profiles
# - Set up automatic monitoring
# - Create management scripts
```

## What Each Script Does

### `scripts/proxy` - Simple Manager
- **Purpose**: Easy-to-use wrapper for common proxy operations
- **Usage**: `./scripts/proxy [on|off|test|apps|status]`
- **Best for**: Quick setup and daily use

### `scripts/macos-quick-proxy.sh` - Quick Setup
- **Purpose**: Fast proxy configuration without complex setup
- **Features**:
  - Configures system proxy settings
  - Sets environment variables
  - Creates shell configuration
  - Starts applications with proxy
- **Best for**: Users who want proxy functionality without complex management

### `scripts/macos-proxy-setup.sh` - Full Setup
- **Purpose**: Comprehensive proxy management system
- **Features**:
  - System proxy configuration
  - Environment variable setup
  - Browser profile creation
  - Automatic monitoring
  - Launch agent for background management
  - Uninstall scripts
- **Best for**: Users who want complete control and automatic management

## Configuration Details

### System Proxy Settings

The scripts configure macOS system proxy settings for:

- **HTTP Proxy**: Routes HTTP traffic through port 9701
- **HTTPS Proxy**: Routes HTTPS traffic through port 9701
- **SOCKS Proxy**: Routes SOCKS traffic through port 9701 (if supported)

### Environment Variables

The following environment variables are set:

```bash
HTTP_PROXY=http://127.0.0.1:9701
HTTPS_PROXY=http://127.0.0.1:9701
http_proxy=http://127.0.0.1:9701
https_proxy=http://127.0.0.1:9701
ALL_PROXY=http://127.0.0.1:9701
all_proxy=http://127.0.0.1:9701
NO_PROXY=localhost,127.0.0.1,::1
no_proxy=localhost,127.0.0.1,::1
```

### Browser Configuration

#### Chrome
- Started with `--proxy-server=http://127.0.0.1:9701`
- All traffic routed through the proxy

#### Firefox
- Custom profile created with proxy settings
- HTTP and HTTPS traffic routed through proxy
- Local traffic bypassed

#### Safari
- Uses system proxy settings
- Automatically configured when system proxy is enabled

## Usage Examples

### Daily Workflow

```bash
# 1. Start WDNS service
./target/release/wdns-service &

# 2. Enable proxy
./scripts/proxy on

# 3. Start your applications
./scripts/proxy apps

# 4. Work normally - all traffic goes through proxy

# 5. When done, disable proxy
./scripts/proxy off
```

### Testing Proxy

```bash
# Test if proxy is working
./scripts/proxy test

# Check your IP through proxy
curl --proxy http://127.0.0.1:9701 http://httpbin.org/ip

# Test HTTPS through proxy
curl --proxy http://127.0.0.1:9701 https://httpbin.org/ip
```

### Terminal Applications

```bash
# Terminal applications will automatically use proxy
# when environment variables are set

# Test with curl
curl http://httpbin.org/ip

# Test with wget
wget -qO- http://httpbin.org/ip

# Test with git (if configured to use proxy)
git clone https://github.com/user/repo.git
```

## Troubleshooting

### Proxy Not Working

1. **Check if WDNS service is running**:
   ```bash
   curl http://127.0.0.1:9701
   ```

2. **Check system proxy settings**:
   ```bash
   networksetup -getwebproxy "Wi-Fi"
   networksetup -getsecurewebproxy "Wi-Fi"
   ```

3. **Check environment variables**:
   ```bash
   env | grep -i proxy
   ```

4. **Test proxy manually**:
   ```bash
   curl --proxy http://127.0.0.1:9701 http://httpbin.org/ip
   ```

### Applications Not Using Proxy

1. **Restart applications** after enabling proxy
2. **Check application-specific proxy settings**
3. **Verify environment variables are set**
4. **Try the full setup script** for comprehensive configuration

### Disable Proxy

```bash
# Quick disable
./scripts/proxy off

# Or manually
sudo networksetup -setwebproxystate "Wi-Fi" off
sudo networksetup -setsecurewebproxystate "Wi-Fi" off
```

## Advanced Configuration

### Custom Proxy Server

```bash
# Use different proxy server
WDNS_PROXY_SERVER="192.168.1.100:9701" ./scripts/proxy on
```

### Browser-Specific Configuration

#### Chrome
```bash
# Start Chrome with custom proxy
open -a "Google Chrome" --args --proxy-server="http://127.0.0.1:9701"
```

#### Firefox
```bash
# Create custom Firefox profile
firefox -profile "$HOME/.wdns-proxy-firefox"
```

### Environment Variables Only

If you only want to set environment variables without system proxy:

```bash
# Source the proxy environment
source ~/.wdns-proxy-env

# Or add to your shell profile
echo 'source ~/.wdns-proxy-env' >> ~/.zshrc
```

## Security Considerations

- **Local Traffic**: Localhost and private network traffic bypasses the proxy
- **HTTPS**: HTTPS traffic is tunneled through the proxy (CONNECT method)
- **Authentication**: The current setup has no authentication
- **Logging**: Consider the privacy implications of proxy logging

## Uninstalling

### Quick Uninstall

```bash
# Disable proxy
./scripts/proxy off

# Remove environment variables
unset HTTP_PROXY HTTPS_PROXY http_proxy https_proxy ALL_PROXY all_proxy
```

### Full Uninstall

```bash
# Run the uninstall script (if using full setup)
~/.wdns-proxy/uninstall.sh
```

## Support

For issues or questions:

1. Check if the WDNS service is running
2. Verify proxy configuration with `./scripts/proxy test`
3. Check system proxy settings
4. Review the logs in `~/.wdns-proxy/` directory

## Script Reference

| Script | Purpose | Usage |
|--------|---------|-------|
| `proxy` | Simple manager | `./scripts/proxy [on\|off\|test\|apps]` |
| `macos-quick-proxy.sh` | Quick setup | `./scripts/macos-quick-proxy.sh -e` |
| `macos-proxy-setup.sh` | Full setup | `./scripts/macos-proxy-setup.sh` |
| `test-proxy.sh` | Test proxy | `./scripts/test-proxy.sh` |

All scripts support `--help` for detailed usage information.
