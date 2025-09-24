# Test script for WDNS Proxy Server
# This script tests the proxy server functionality

param(
    [string]$Server = "127.0.0.1:9701",
    [string]$DnsServer = "127.0.0.1:9700",
    [switch]$Help
)

if ($Help) {
    Write-Host "WDNS Proxy Server Test Script" -ForegroundColor Blue
    Write-Host "==============================" -ForegroundColor Blue
    Write-Host ""
    Write-Host "Usage: .\test-proxy.ps1 [OPTIONS]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -Server     Set proxy server address (default: 127.0.0.1:9701)"
    Write-Host "  -DnsServer  Set DNS server address (default: 127.0.0.1:9700)"
    Write-Host "  -Help       Show this help message"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\test-proxy.ps1                                    # Test with default server"
    Write-Host "  .\test-proxy.ps1 -Server 192.168.1.100:9701         # Test with specific server"
    Write-Host "  .\test-proxy.ps1 -DnsServer 192.168.1.100:9700      # Test with specific DNS server"
    exit 0
}

# Function to test if service is running
function Test-ServiceRunning {
    param(
        [string]$ServiceName,
        [string]$Port
    )
    
    Write-Host "Testing $ServiceName on port $Port..." -ForegroundColor Yellow
    
    try {
        $response = Invoke-WebRequest -Uri "http://$Port" -TimeoutSec 5 -UseBasicParsing -ErrorAction Stop
        Write-Host "✓ $ServiceName is running on port $Port" -ForegroundColor Green
        return $true
    }
    catch {
        Write-Host "✗ $ServiceName is not running on port $Port" -ForegroundColor Red
        return $false
    }
}

# Function to test proxy functionality
function Test-ProxyHttp {
    Write-Host "Testing HTTP proxy functionality..." -ForegroundColor Yellow
    
    $testUrl = "http://httpbin.org/ip"
    $proxyUrl = "http://$Server"
    
    Write-Host "Testing: $testUrl through proxy: $proxyUrl"
    
    try {
        $response = Invoke-WebRequest -Uri $testUrl -Proxy $proxyUrl -TimeoutSec 10 -UseBasicParsing -ErrorAction Stop
        Write-Host "✓ HTTP proxy is working" -ForegroundColor Green
        return $true
    }
    catch {
        Write-Host "✗ HTTP proxy test failed: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

# Function to test proxy with HTTPS
function Test-ProxyHttps {
    Write-Host "Testing HTTPS proxy functionality..." -ForegroundColor Yellow
    
    $testUrl = "https://httpbin.org/ip"
    $proxyUrl = "http://$Server"
    
    Write-Host "Testing: $testUrl through proxy: $proxyUrl"
    
    try {
        $response = Invoke-WebRequest -Uri $testUrl -Proxy $proxyUrl -TimeoutSec 10 -UseBasicParsing -ErrorAction Stop
        Write-Host "✓ HTTPS proxy is working" -ForegroundColor Green
        return $true
    }
    catch {
        Write-Host "✗ HTTPS proxy test failed: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

# Function to test proxy with environment variables
function Test-ProxyEnv {
    Write-Host "Testing proxy with environment variables..." -ForegroundColor Yellow
    
    $testUrl = "http://httpbin.org/ip"
    
    Write-Host "Setting HTTP_PROXY and HTTPS_PROXY environment variables"
    $env:HTTP_PROXY = "http://$Server"
    $env:HTTPS_PROXY = "http://$Server"
    
    try {
        $response = Invoke-WebRequest -Uri $testUrl -TimeoutSec 10 -UseBasicParsing -ErrorAction Stop
        Write-Host "✓ Proxy with environment variables is working" -ForegroundColor Green
        Remove-Item Env:HTTP_PROXY -ErrorAction SilentlyContinue
        Remove-Item Env:HTTPS_PROXY -ErrorAction SilentlyContinue
        return $true
    }
    catch {
        Write-Host "✗ Proxy with environment variables test failed: $($_.Exception.Message)" -ForegroundColor Red
        Remove-Item Env:HTTP_PROXY -ErrorAction SilentlyContinue
        Remove-Item Env:HTTPS_PROXY -ErrorAction SilentlyContinue
        return $false
    }
}

# Function to test proxy response
function Test-ProxyResponse {
    Write-Host "Testing proxy response content..." -ForegroundColor Yellow
    
    $testUrl = "http://httpbin.org/ip"
    $proxyUrl = "http://$Server"
    
    Write-Host "Getting response from: $testUrl through proxy: $proxyUrl"
    
    try {
        $response = Invoke-WebRequest -Uri $testUrl -Proxy $proxyUrl -TimeoutSec 10 -UseBasicParsing -ErrorAction Stop
        Write-Host "✓ Proxy response received:" -ForegroundColor Green
        Write-Host $response.Content
        return $true
    }
    catch {
        Write-Host "✗ Proxy response test failed: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

# Function to test proxy with different hosts
function Test-ProxyHosts {
    Write-Host "Testing proxy with different hosts..." -ForegroundColor Yellow
    
    $hosts = @("httpbin.org", "google.com", "github.com")
    $proxyUrl = "http://$Server"
    
    foreach ($host in $hosts) {
        Write-Host "Testing: $host through proxy"
        
        try {
            $response = Invoke-WebRequest -Uri "http://$host" -Proxy $proxyUrl -TimeoutSec 5 -UseBasicParsing -ErrorAction Stop
            Write-Host "✓ $host is accessible through proxy" -ForegroundColor Green
        }
        catch {
            Write-Host "✗ $host is not accessible through proxy" -ForegroundColor Red
        }
    }
}

# Function to test proxy performance
function Test-ProxyPerformance {
    Write-Host "Testing proxy performance..." -ForegroundColor Yellow
    
    $testUrl = "http://httpbin.org/ip"
    $proxyUrl = "http://$Server"
    $numRequests = 5
    
    Write-Host "Making $numRequests concurrent requests through proxy"
    
    $startTime = Get-Date
    
    $jobs = @()
    for ($i = 1; $i -le $numRequests; $i++) {
        $jobs += Start-Job -ScriptBlock {
            param($url, $proxy)
            try {
                Invoke-WebRequest -Uri $url -Proxy $proxy -TimeoutSec 10 -UseBasicParsing -ErrorAction Stop
                return $true
            }
            catch {
                return $false
            }
        } -ArgumentList $testUrl, $proxyUrl
    }
    
    $results = $jobs | Wait-Job | Receive-Job
    $jobs | Remove-Job
    
    $endTime = Get-Date
    $duration = ($endTime - $startTime).TotalSeconds
    
    $successCount = ($results | Where-Object { $_ -eq $true }).Count
    Write-Host "✓ Completed $successCount/$numRequests requests in $([math]::Round($duration, 2))s" -ForegroundColor Green
}

# Function to show proxy configuration
function Show-ProxyConfig {
    Write-Host "Proxy Configuration:" -ForegroundColor Yellow
    Write-Host "HTTP_PROXY=http://$Server"
    Write-Host "HTTPS_PROXY=http://$Server"
    Write-Host ""
    Write-Host "Browser Configuration:"
    Write-Host "Chrome: --proxy-server=http://$Server"
    Write-Host "Firefox: Manual proxy configuration"
    Write-Host "Edge: Proxy settings in System Settings"
    Write-Host ""
    Write-Host "PowerShell:"
    Write-Host "`$env:HTTP_PROXY='http://$Server'"
    Write-Host "`$env:HTTPS_PROXY='http://$Server'"
    Write-Host ""
    Write-Host "Command Line:"
    Write-Host "curl --proxy http://$Server https://example.com"
    Write-Host ""
}

# Function to test DNS service
function Test-DnsService {
    Write-Host "Testing DNS service..." -ForegroundColor Yellow
    
    $dnsUrl = "http://$DnsServer/api/dns/resolve"
    $body = @{
        hosts = @("google.com", "github.com")
    } | ConvertTo-Json
    
    try {
        $response = Invoke-RestMethod -Uri $dnsUrl -Method POST -Body $body -ContentType "application/json" -TimeoutSec 10
        Write-Host "✓ DNS service is working" -ForegroundColor Green
        Write-Host "Response: $($response | ConvertTo-Json -Depth 3)"
        return $true
    }
    catch {
        Write-Host "✗ DNS service test failed: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

# Main test execution
function Main {
    Write-Host "WDNS Proxy Server Test" -ForegroundColor Blue
    Write-Host "=======================" -ForegroundColor Blue
    Write-Host "Proxy Server: $Server"
    Write-Host "DNS Server: $DnsServer"
    Write-Host ""
    
    # Test if services are running
    Test-ServiceRunning "DNS Service" $DnsServer
    Test-ServiceRunning "Proxy Service" $Server
    Write-Host ""
    
    # Test DNS service
    Test-DnsService
    Write-Host ""
    
    # Test proxy functionality
    Test-ProxyHttp
    Write-Host ""
    
    Test-ProxyHttps
    Write-Host ""
    
    Test-ProxyEnv
    Write-Host ""
    
    Test-ProxyResponse
    Write-Host ""
    
    Test-ProxyHosts
    Write-Host ""
    
    Test-ProxyPerformance
    Write-Host ""
    
    Show-ProxyConfig
    
    Write-Host "Proxy server tests completed!" -ForegroundColor Green
}

# Check if required modules are available
function Test-Dependencies {
    $missingModules = @()
    
    if (-not (Get-Module -ListAvailable -Name "Invoke-WebRequest")) {
        $missingModules += "Invoke-WebRequest"
    }
    
    if ($missingModules.Count -ne 0) {
        Write-Host "Error: Missing required modules: $($missingModules -join ', ')" -ForegroundColor Red
        Write-Host "Please install the missing modules and try again."
        exit 1
    }
}

# Run the tests
Test-Dependencies
Main
