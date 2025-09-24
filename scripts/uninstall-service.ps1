# WDNS Service Uninstallation Script
# Run as Administrator

param(
    [string]$ServiceName = "WDNSService"
)

Write-Host "Uninstalling WDNS Service..." -ForegroundColor Green

# Check if running as Administrator
if (-NOT ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Error "This script must be run as Administrator"
    exit 1
}

# Check if service exists
$service = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
if (-not $service) {
    Write-Host "Service '$ServiceName' not found" -ForegroundColor Yellow
    exit 0
}

Write-Host "Found service: $ServiceName" -ForegroundColor Cyan

# Stop the service if running
if ($service.Status -eq "Running") {
    Write-Host "Stopping service..." -ForegroundColor Yellow
    Stop-Service -Name $ServiceName -Force
    Start-Sleep -Seconds 3
}

# Delete the service
Write-Host "Removing service..." -ForegroundColor Cyan
$result = sc.exe delete $ServiceName

if ($LASTEXITCODE -eq 0) {
    Write-Host "Service uninstalled successfully!" -ForegroundColor Green
} else {
    Write-Error "Failed to uninstall service: $result"
    exit 1
}
