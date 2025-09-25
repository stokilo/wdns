# Hostname Testing Utility

## Overview

I've added a **hostname testing utility** to the macOS Network Connection Monitor application that allows you to test different hostname resolution methods and see which one works for your specific hostname.

## New Features Added

### 1. Test Hostname Resolution Dialog

**Location**: Main UI → "Test Hostname Resolution" button

**Features**:
- ✅ **Hostname Input Field** - Enter any hostname to test
- ✅ **Direct DNS Test** - Test standard DNS resolution
- ✅ **SOCKS5 Proxy Test** - Test via SOCKS5 proxy (192.168.0.115:9702)
- ✅ **Interceptor Test** - Test via traffic interceptor (127.0.0.1:5353)
- ✅ **Results Display** - Scrollable results with detailed output
- ✅ **Clear Results** - Clear previous test results

### 2. Test Methods

#### **Test Direct DNS**
- Uses standard `nslookup` command
- Tests normal DNS resolution
- Shows if hostname can be resolved directly

#### **Test via SOCKS5**
- Uses `curl --socks5-hostname 192.168.0.115:9702`
- Tests hostname resolution through SOCKS5 proxy
- Shows if SOCKS5 proxy can resolve the hostname

#### **Test via Interceptor**
- Uses `nslookup` with DNS interceptor (127.0.0.1:5353)
- Uses `curl --dns-servers 127.0.0.1:5353`
- Tests if traffic interceptor is working
- Shows if traffic is being routed through interceptor

## Usage Instructions

### 1. Start the Application

```bash
cd macos-listener
cargo run
```

### 2. Open Test Dialog

1. Click **"Test Hostname Resolution"** button in the main UI
2. Enter hostname: `networkpartner-kion-dev.cprt.kion.cloud`
3. Test different resolution methods

### 3. Test Different Methods

#### **Step 1: Test Direct DNS**
- Click **"Test Direct DNS"**
- This will show if the hostname can be resolved normally
- Expected result: ❌ **FAILED** (hostname not resolvable directly)

#### **Step 2: Test via SOCKS5**
- Click **"Test via SOCKS5"**
- This will test if SOCKS5 proxy can resolve the hostname
- Expected result: ✅ **SUCCESS** (SOCKS5 proxy resolves hostname)

#### **Step 3: Test via Interceptor**
- Click **"Test via Interceptor"**
- This will test if traffic interceptor is working
- Expected result: ✅ **SUCCESS** (if interceptor is properly configured)

### 4. Compare Results

The results will show:
- **DNS Resolution**: Whether hostname can be resolved
- **HTTP Connection**: Whether HTTP request succeeds
- **Verbose Output**: Detailed connection information
- **Error Messages**: Any errors encountered

## Expected Results

### For `networkpartner-kion-dev.cprt.kion.cloud`:

#### **Direct DNS Test**
```
❌ Direct DNS Failed:
nslookup: can't resolve 'networkpartner-kion-dev.cprt.kion.cloud'
```

#### **SOCKS5 Proxy Test**
```
✅ SOCKS5 Proxy Test:
* Trying 192.168.0.115:9702...
* Connected to 192.168.0.115 (192.168.0.115) port 9702
* SOCKS5 connect to networkpartner-kion-dev.cprt.kion.cloud:80 (remotely resolved)
* HTTP/1.1 200 OK
```

#### **Interceptor Test**
```
✅ DNS via Interceptor:
Name: networkpartner-kion-dev.cprt.kion.cloud
Address: [resolved IP address]

✅ HTTP via Interceptor:
* Connected to [resolved IP] port 80
* HTTP/1.1 200 OK
```

## Troubleshooting

### If Direct DNS Fails
- **Expected**: Hostname is not publicly resolvable
- **Action**: This is normal for internal/corporate hostnames

### If SOCKS5 Proxy Fails
- **Check**: SOCKS5 proxy server is running on 192.168.0.115:9702
- **Check**: Network connectivity to proxy server
- **Check**: Authentication credentials if required

### If Interceptor Test Fails
- **Check**: Traffic interceptor is started
- **Check**: DNS interceptor is running on port 5353
- **Check**: Proxy rules are configured correctly
- **Check**: Pattern matching rules include the hostname

## Configuration Requirements

### 1. SOCKS5 Proxy Configuration
- **Host**: 192.168.0.115
- **Port**: 9702
- **Type**: SOCKS5
- **Authentication**: If required

### 2. Traffic Interceptor Rules
- **Pattern**: `*.kion.cloud` or `*.cprt.kion.cloud`
- **Proxy**: Assign to SOCKS5 proxy
- **Status**: Enabled

### 3. Traffic Interceptor Status
- **DNS Interceptor**: Running on port 5353
- **TCP/UDP Monitoring**: Active
- **Rule Matching**: Enabled

## Testing Script

I've also created a command-line test script:

```bash
./test_hostname_resolution.sh
```

This script tests:
- Direct DNS resolution
- SOCKS5 proxy resolution
- DNS interceptor resolution
- HTTP via DNS interceptor
- Comparison of all methods

## Debug Information

The test results include:
- **Verbose curl output** - Shows connection details
- **DNS resolution details** - Shows resolved IP addresses
- **Error messages** - Shows specific failure reasons
- **Timing information** - Shows connection times
- **Protocol details** - Shows HTTP/SOCKS5 protocol information

## Next Steps

1. **Test the hostname**: Use the UI to test `networkpartner-kion-dev.cprt.kion.cloud`
2. **Configure rules**: Add pattern `*.kion.cloud` or `*.cprt.kion.cloud`
3. **Start interceptor**: Enable traffic interception
4. **Verify routing**: Check if traffic is being routed through SOCKS5 proxy
5. **Monitor traffic**: Use "View Intercepted Traffic" to see real-time routing

## Expected Behavior

When properly configured:
- **Direct DNS**: ❌ Fails (hostname not publicly resolvable)
- **SOCKS5 Proxy**: ✅ Succeeds (proxy resolves hostname remotely)
- **Interceptor**: ✅ Succeeds (traffic routed through SOCKS5 proxy)

This testing utility helps you verify that your traffic interceptor is working correctly and routing traffic through the SOCKS5 proxy as expected.
