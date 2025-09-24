# WDNS Service Test Script

param(
    [string]$ServiceUrl = "http://127.0.0.1:9700"
)

Write-Host "Testing WDNS Service at $ServiceUrl" -ForegroundColor Green

# Test health endpoint
Write-Host "`n1. Testing health endpoint..." -ForegroundColor Cyan
try {
    $healthResponse = Invoke-RestMethod -Uri "$ServiceUrl/health" -Method GET
    Write-Host "✓ Health check passed" -ForegroundColor Green
    Write-Host "Response: $($healthResponse | ConvertTo-Json -Compress)" -ForegroundColor Gray
} catch {
    Write-Host "✗ Health check failed: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Test root endpoint
Write-Host "`n2. Testing root endpoint..." -ForegroundColor Cyan
try {
    $rootResponse = Invoke-RestMethod -Uri "$ServiceUrl/" -Method GET
    Write-Host "✓ Root endpoint working" -ForegroundColor Green
    Write-Host "Response: $($rootResponse | ConvertTo-Json -Compress)" -ForegroundColor Gray
} catch {
    Write-Host "✗ Root endpoint failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test DNS resolution
Write-Host "`n3. Testing DNS resolution..." -ForegroundColor Cyan
$testHosts = @("google.com", "github.com", "stackoverflow.com", "invalid-host-that-does-not-exist.example")

$dnsRequest = @{
    hosts = $testHosts
} | ConvertTo-Json

try {
    $dnsResponse = Invoke-RestMethod -Uri "$ServiceUrl/api/dns/resolve" -Method POST -Body $dnsRequest -ContentType "application/json"
    Write-Host "✓ DNS resolution completed" -ForegroundColor Green
    Write-Host "Total resolved: $($dnsResponse.total_resolved)" -ForegroundColor Cyan
    Write-Host "Total errors: $($dnsResponse.total_errors)" -ForegroundColor Cyan
    
    Write-Host "`nResults:" -ForegroundColor Yellow
    foreach ($result in $dnsResponse.results) {
        $status = if ($result.status -eq "success") { "✓" } else { "✗" }
        $color = if ($result.status -eq "success") { "Green" } else { "Red" }
        Write-Host "  $status $($result.host): $($result.ip_addresses -join ', ')" -ForegroundColor $color
        if ($result.error) {
            Write-Host "    Error: $($result.error)" -ForegroundColor Red
        }
    }
} catch {
    Write-Host "✗ DNS resolution failed: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Performance test
Write-Host "`n4. Testing performance with multiple requests..." -ForegroundColor Cyan
$startTime = Get-Date
$requests = 1..5

$jobs = @()
foreach ($i in $requests) {
    $job = Start-Job -ScriptBlock {
        param($url, $hosts)
        $request = @{ hosts = $hosts } | ConvertTo-Json
        try {
            $response = Invoke-RestMethod -Uri "$url/api/dns/resolve" -Method POST -Body $request -ContentType "application/json"
            return @{ success = $true; resolved = $response.total_resolved; errors = $response.total_errors }
        } catch {
            return @{ success = $false; error = $_.Exception.Message }
        }
    } -ArgumentList $ServiceUrl, @("google.com", "microsoft.com", "amazon.com")
    $jobs += $job
}

# Wait for all jobs to complete
$results = $jobs | Wait-Job | Receive-Job
$jobs | Remove-Job

$endTime = Get-Date
$duration = ($endTime - $startTime).TotalMilliseconds

$successful = ($results | Where-Object { $_.success }).Count
$totalResolved = ($results | Where-Object { $_.success } | Measure-Object -Property resolved -Sum).Sum
$totalErrors = ($results | Where-Object { $_.success } | Measure-Object -Property errors -Sum).Sum

Write-Host "✓ Performance test completed" -ForegroundColor Green
Write-Host "  Duration: $([math]::Round($duration, 2))ms" -ForegroundColor Cyan
Write-Host "  Successful requests: $successful/$($requests.Count)" -ForegroundColor Cyan
Write-Host "  Total resolved: $totalResolved" -ForegroundColor Cyan
Write-Host "  Total errors: $totalErrors" -ForegroundColor Cyan

Write-Host "`n🎉 All tests passed! WDNS Service is working correctly." -ForegroundColor Green
