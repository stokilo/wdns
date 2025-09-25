# Traffic Interceptor Implementation

This document describes the low-level traffic interceptor implementation for the macOS Network Connection Monitor application.

## Overview

The Traffic Interceptor provides **system-level network traffic interception** that routes matching connections through an **external SOCKS5 proxy** (192.168.0.115:9702) based on configurable rules. It does NOT implement a SOCKS5 server but instead routes existing traffic through your external SOCKS5 proxy.

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Client App    │───▶│ Traffic Interceptor│───▶│ External SOCKS5 │
│  (Browser/App)  │    │   (Low-level)     │    │   Proxy Server  │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │
                              ▼
                       ┌──────────────────┐
                       │  DNS Interceptor  │
                       │   (Port 5353)     │
                       └──────────────────┘
```

## Key Features

✅ **Low-level Traffic Interception** - Captures system-level network traffic
✅ **DNS Query Interception** - Intercepts and routes DNS queries through SOCKS5
✅ **TCP/UDP Connection Routing** - Routes TCP and UDP connections through SOCKS5
✅ **Pattern-based Rules** - Flexible rule system for traffic routing decisions
✅ **Real-time Monitoring** - Live connection tracking and statistics
✅ **External SOCKS5 Integration** - Routes through existing SOCKS5 proxy server

## Components

### 1. Traffic Interceptor (`traffic_interceptor.rs`)

**Core Functionality:**
- System-level traffic interception
- DNS query capture and routing
- TCP/UDP connection monitoring
- Pattern matching for routing decisions
- Real-time connection tracking

**Key Methods:**
- `start()`: Start traffic interception
- `stop()`: Stop traffic interception
- `get_intercepted_connections()`: Get intercepted connections
- `intercept_dns_traffic()`: Intercept DNS queries
- `intercept_tcp_traffic()`: Intercept TCP connections
- `intercept_udp_traffic()`: Intercept UDP connections

### 2. Traffic Interceptor Helpers (`traffic_interceptor_helpers.rs`)

**Helper Functions:**
- SOCKS5 proxy connection handling
- DNS packet parsing and routing
- Pattern matching for rules
- Connection recording and statistics
- System connection monitoring

## Configuration

### Proxy Configuration

The system uses your existing proxy configuration:

```json
{
  "global_enabled": true,
  "proxies": [
    {
      "id": 1,
      "name": "Corporate Proxy",
      "host": "192.168.0.115",
      "port": 9702,
      "proxy_type": "Socks5",
      "username": "user",
      "password": "pass",
      "enabled": true
    }
  ],
  "rules": [
    {
      "id": 1,
      "name": "Corporate Domains",
      "pattern": "*.kion.cloud",
      "enabled": true,
      "proxy_id": 1
    }
  ]
}
```

### Pattern Matching

The system supports flexible pattern matching:

- **Exact match**: `example.com`
- **Domain wildcard**: `*.kion.cloud`
- **Prefix wildcard**: `kion.*`
- **IP wildcard**: `192.168.1.*`
- **Multiple patterns**: `*.kion.cloud;*.corp.com`

## Usage

### 1. Start the Application

```bash
cd macos-listener
cargo run
```

### 2. Configure External SOCKS5 Proxy

1. Click **"Configure Proxies"** in the UI
2. Add your SOCKS5 proxy server:
   - **Host**: `192.168.0.115`
   - **Port**: `9702`
   - **Type**: `SOCKS5`
   - **Authentication**: If required
3. Save configuration

### 3. Configure Routing Rules

1. Click **"Manage Rules"** in the UI
2. Add routing rules with patterns:
   - `*.kion.cloud` - Route all KION cloud domains
   - `100.64.1.*` - Route specific IP ranges
   - `kiongroup.net` - Route specific domains
3. Assign rules to your SOCKS5 proxy
4. Enable rules

### 4. Start Traffic Interceptor

1. Click **"Start Traffic Interceptor"** in the UI
2. The system will start:
   - DNS interceptor on port 5353
   - TCP/UDP connection monitoring
   - Traffic routing through SOCKS5 proxy

### 5. Monitor Traffic

- **View Intercepted Traffic**: See real-time intercepted connections
- **Connection Log**: View connection events and routing decisions
- **Statistics**: Monitor proxied vs direct connections

## How It Works

### 1. DNS Interception

1. **DNS Query Capture**: Intercepts DNS queries on port 5353
2. **Domain Parsing**: Extracts domain names from DNS packets
3. **Rule Matching**: Checks domain against configured rules
4. **SOCKS5 Routing**: Routes matching queries through SOCKS5 proxy
5. **Response Forwarding**: Returns DNS responses to clients

### 2. TCP/UDP Interception

1. **Connection Monitoring**: Monitors system TCP/UDP connections
2. **Hostname Resolution**: Resolves IP addresses to hostnames
3. **Rule Matching**: Checks hostnames against configured rules
4. **SOCKS5 Routing**: Routes matching connections through SOCKS5 proxy
5. **Data Forwarding**: Forwards data through SOCKS5 tunnel

### 3. Pattern Matching

The system uses flexible pattern matching:

```rust
// Exact match
"example.com" matches "example.com"

// Domain wildcard
"*.kion.cloud" matches "test.kion.cloud", "api.kion.cloud"

// Prefix wildcard
"kion.*" matches "kion.test", "kion.api"

// IP wildcard
"192.168.1.*" matches "192.168.1.100", "192.168.1.200"
```

## Testing

### Automated Testing

Run the comprehensive test suite:

```bash
./test_traffic_interceptor.sh
```

This script tests:
- Application availability
- Network connectivity
- SOCKS5 proxy connectivity
- DNS resolution
- HTTP/HTTPS requests
- Traffic interception
- Pattern matching

### Manual Testing

#### Test DNS Interception

```bash
# Test DNS resolution through interceptor
nslookup google.com 127.0.0.1:5353
dig @127.0.0.1 -p 5353 google.com

# Test with custom DNS
curl --dns-servers 127.0.0.1:5353 http://google.com
```

#### Test Traffic Routing

1. **Configure Rules**: Add rules for specific domains
2. **Start Interceptor**: Enable traffic interception
3. **Make Requests**: Use browser or curl to test domains
4. **Monitor Traffic**: Check intercepted traffic in UI

#### Test SOCKS5 Proxy

```bash
# Test direct SOCKS5 connection
curl --socks5 192.168.0.115:9702 http://httpbin.org/ip

# Test HTTPS via SOCKS5
curl --socks5 192.168.0.115:9702 https://httpbin.org/ip
```

## macOS Permissions

### Required Permissions

The traffic interceptor requires several permissions:

1. **Network Access**: Allow network connections
2. **Packet Capture**: Capture network packets
3. **DNS Proxy**: Intercept DNS queries
4. **System Monitoring**: Monitor system connections

### Setup Steps

1. **Enable Developer Mode**:
   - System Preferences > Security & Privacy > Privacy
   - Select "Developer Tools"
   - Add your application

2. **Grant Network Permissions**:
   - System Preferences > Security & Privacy > Privacy
   - Select "Network"
   - Add your application

3. **Allow Packet Capture**:
   - System Preferences > Security & Privacy > Privacy
   - Select "Network"
   - Allow packet capture

## Troubleshooting

### Common Issues

#### 1. DNS Interception Not Working
- **Check**: DNS interceptor is running on port 5353
- **Verify**: DNS settings in System Preferences
- **Test**: `nslookup google.com 127.0.0.1:5353`

#### 2. Traffic Not Routed
- **Check**: Proxy rules are configured correctly
- **Verify**: Patterns match target domains
- **Test**: SOCKS5 proxy is accessible

#### 3. SOCKS5 Connection Failed
- **Check**: SOCKS5 proxy server is running
- **Verify**: Host and port are correct
- **Test**: Direct SOCKS5 connection

#### 4. Permission Denied
- **Solution**: Ensure proper permissions
- **Check**: Application is in Developer Tools
- **Verify**: Network access is granted

### Debug Commands

```bash
# Check if services are running
lsof -i :5353  # DNS interceptor
netstat -an | grep :5353

# Check network connections
netstat -an | grep ESTABLISHED

# Test DNS resolution
nslookup google.com 127.0.0.1:5353
dig @127.0.0.1 -p 5353 google.com

# Test SOCKS5 proxy
curl --socks5 192.168.0.115:9702 http://httpbin.org/ip
```

## Performance Considerations

### Optimization Tips

1. **Connection Pooling**: Reuse SOCKS5 connections when possible
2. **Async Processing**: Use async I/O for better performance
3. **Memory Management**: Monitor memory usage for long-running connections
4. **CPU Usage**: Optimize packet processing for minimal CPU impact

### Monitoring

The application provides real-time monitoring:

- **Connection Statistics**: Active connections, bytes transferred
- **Performance Metrics**: Response times, throughput
- **Error Rates**: Failed connections, timeouts
- **Resource Usage**: CPU, memory, network utilization

## Security Considerations

### Authentication

- Use strong usernames and passwords for SOCKS5 proxies
- Implement proper authentication mechanisms
- Consider certificate-based authentication

### Encryption

- Use encrypted connections when possible
- Implement TLS/SSL for sensitive traffic
- Consider VPN integration for additional security

### Privacy

- Don't log sensitive user data
- Implement proper data encryption
- Follow privacy best practices

## Advanced Configuration

### Custom Network Interfaces

```rust
// Configure custom network interface
let interface = NWInterface.interface(withName: "en0");
let parameters = NWParameters.tcp;
parameters.requiredInterface = interface;
```

### Traffic Filtering

```rust
// Implement custom traffic filtering
fn should_route_traffic(connection: &NetworkConnection) -> bool {
    // Custom filtering logic
    // Return true if should be proxied
    true
}
```

### Performance Tuning

```rust
// Optimize packet processing
fn process_packets(packets: &[Data]) {
    for packet in packets {
        // Efficient packet processing
        // Avoid blocking operations
    }
}
```

## Conclusion

The Traffic Interceptor provides a comprehensive solution for routing system traffic through external SOCKS5 proxies on macOS. It includes DNS interception, TCP/UDP routing, and real-time monitoring capabilities.

For additional support:
- Check the application logs for detailed error information
- Use the built-in monitoring tools for troubleshooting
- Refer to the macOS permissions guide for setup issues
- Test with the provided test suite for functionality verification

## References

- [SOCKS5 Protocol (RFC 1928)](https://tools.ietf.org/html/rfc1928)
- [DNS Protocol (RFC 1035)](https://tools.ietf.org/html/rfc1035)
- [Apple Network Extension Framework](https://developer.apple.com/documentation/networkextension)
- [macOS System Extensions](https://developer.apple.com/system-extensions/)
