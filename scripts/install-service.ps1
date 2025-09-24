# WDNS Service Installation Script
# Run as Administrator

param(
    [string]$ServicePath = ".\target\release\wdns-service.exe",
    [string]$ServiceName = "WDNSService",
    [string]$DisplayName = "Windows DNS Resolution Service",
    [string]$Description = "High-performance DNS resolution service with HTTP API"
)

Write-Host "Installing WDNS Service..." -ForegroundColor Green

# Check if running as Administrator
if (-NOT ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Error "This script must be run as Administrator"
    exit 1
}

# Check if service executable exists
if (-not (Test-Path $ServicePath)) {
    Write-Error "Service executable not found at: $ServicePath"
    Write-Host "Please build the project first: cargo build --release" -ForegroundColor Yellow
    exit 1
}

# Get absolute path
$FullServicePath = (Resolve-Path $ServicePath).Path

Write-Host "Service executable: $FullServicePath" -ForegroundColor Cyan

# Check if service already exists
$existingService = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
if ($existingService) {
    Write-Host "Service already exists. Stopping and removing..." -ForegroundColor Yellow
    
    if ($existingService.Status -eq "Running") {
        Stop-Service -Name $ServiceName -Force
        Write-Host "Service stopped" -ForegroundColor Yellow
    }
    
    sc.exe delete $ServiceName
    Start-Sleep -Seconds 2
}

# Create the service
Write-Host "Creating service..." -ForegroundColor Cyan
$result = sc.exe create $ServiceName binPath="$FullServicePath --service" start=auto

if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to create service: $result"
    exit 1
}

# Set service description
sc.exe description $ServiceName $Description

# Set service to auto-start
sc.exe config $ServiceName start=auto

Write-Host "Service created successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "To start the service:" -ForegroundColor Cyan
Write-Host "  sc.exe start $ServiceName" -ForegroundColor White
Write-Host "  # or" -ForegroundColor Gray
Write-Host "  Start-Service -Name $ServiceName" -ForegroundColor White
Write-Host ""
Write-Host "To check service status:" -ForegroundColor Cyan
Write-Host "  sc.exe query $ServiceName" -ForegroundColor White
Write-Host "  # or" -ForegroundColor Gray
Write-Host "  Get-Service -Name $ServiceName" -ForegroundColor White
Write-Host ""
Write-Host "To uninstall the service:" -ForegroundColor Cyan
Write-Host "  sc.exe delete $ServiceName" -ForegroundColor White
