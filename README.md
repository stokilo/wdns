# WDNS Service

A high-performance Windows DNS resolution service built with Rust and Tokio. Provides HTTP API for concurrent DNS resolution with Windows service support.

## Features

- 🚀 **High Performance**: Built with Rust and Tokio for maximum speed
- 🔄 **Concurrent Resolution**: Resolves multiple hosts in parallel
- 🪟 **Windows Service**: Runs as a native Windows service
- 🌐 **HTTP API**: RESTful API for DNS resolution
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
- **Windows Build Tools**: Microsoft Visual C++ Build Tools or Visual Studio (for Windows compilation)

### Windows Requirements

For Windows development, you need one of:
- **Visual Studio 2017+** with C++ workload
- **Build Tools for Visual Studio 2022** (recommended)
- **Visual Studio Community** with C++ development tools

Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/

### Common Windows Issues

#### Problem: `cargo` command not found
```powershell
# Add cargo to PATH
$env:PATH += ";$env:USERPROFILE\.cargo\bin"

# Or restart PowerShell after Rust installation
```

#### Problem: `link.exe` not found
```powershell
# Install Build Tools for Visual Studio 2022
# Or use GNU toolchain:
rustup target add x86_64-pc-windows-gnu
rustup default stable-x86_64-pc-windows-gnu
```

## Building

### 1. Install Rust

If you don't have Rust installed:

```powershell
# Install Rust via rustup
Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "rustup-init.exe"
.\rustup-init.exe -y

# Add cargo to PATH (if needed)
$env:PATH += ";$env:USERPROFILE\.cargo\bin"

# Verify installation
cargo --version
rustc --version
```

### 1.1. Install Windows Build Tools (Required for Windows)

```powershell
# Option A: Install Build Tools for Visual Studio 2022 (Recommended)
# Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/
# Select "C++ build tools" during installation

# Option B: Use GNU toolchain (Alternative)
rustup target add x86_64-pc-windows-gnu
rustup default stable-x86_64-pc-windows-gnu

# Verify build tools
where link.exe  # Should find link.exe
where cl.exe    # Should find cl.exe
```

### 2. Build the Project

```powershell
# Clone and build
git clone <your-repo-url>
cd wdns

# Clean any previous builds
cargo clean

# Build the project
cargo build --release

# Run tests
cargo test
```

The executable will be created at:
- **Windows**: `target\release\wdns-service.exe`
- **macOS/Linux**: `target/release/wdns-service`

## Running

### Standalone Mode

Run the service directly for testing:

```powershell
# Windows
.\target\release\wdns-service.exe

# macOS/Linux
./target/release/wdns-service

# Or with custom config (if implemented)
.\target\release\wdns-service.exe --config custom-config.json
```

The service will start on `http://127.0.0.1:8080` by default.

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
  "bind_address": "127.0.0.1:8080",
  "dns_timeout_seconds": 10,
  "max_concurrent_resolutions": 100
}
```

### Configuration Options

- `bind_address`: IP address and port to bind the HTTP server
- `dns_timeout_seconds`: Timeout for DNS resolution in seconds
- `max_concurrent_resolutions`: Maximum number of concurrent DNS resolutions

## Testing the Service

### Using PowerShell

```powershell
# Health check
Invoke-RestMethod -Uri "http://127.0.0.1:8080/health"

# DNS resolution
$body = @{
    hosts = @("google.com", "github.com", "stackoverflow.com")
} | ConvertTo-Json

Invoke-RestMethod -Uri "http://127.0.0.1:8080/api/dns/resolve" -Method POST -Body $body -ContentType "application/json"
```

### Using curl

```bash
# Health check
curl http://127.0.0.1:8080/health

# DNS resolution
curl -X POST http://127.0.0.1:8080/api/dns/resolve \
  -H "Content-Type: application/json" \
  -d '{"hosts": ["google.com", "github.com"]}'
```

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
   netstat -an | findstr :8080
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
