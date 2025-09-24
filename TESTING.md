# WDNS Service Testing Guide

This document describes the comprehensive testing suite for the WDNS service, including unit tests, integration tests, and load tests.

## Test Structure

The test suite is organized into several categories:

- **Unit Tests**: Test individual components in isolation
- **Integration Tests**: Test API endpoints and HTTP interactions
- **Load Tests**: Test performance under various load conditions
- **API Tests**: Test specific API functionality and error handling

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Specific Test Categories
```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration_tests

# API tests only
cargo test --test api_tests

# Load tests only
cargo test --test load_tests
```

### Run Tests with Output
```bash
# Show test output
cargo test -- --nocapture

# Run tests in parallel
cargo test -- --test-threads=4
```

### Run Tests with Filtering
```bash
# Run tests matching a pattern
cargo test dns

# Run specific test
cargo test test_resolve_single_host
```

## Test Categories

### 1. Unit Tests (`src/*.rs`)

#### DNS Resolver Tests (`src/dns.rs`)
- `test_resolve_single_host`: Tests resolution of a single valid host
- `test_resolve_invalid_host`: Tests handling of invalid hostnames
- `test_resolve_multiple_hosts`: Tests concurrent resolution of multiple hosts
- `test_resolve_empty_hosts`: Tests handling of empty host lists
- `test_resolve_localhost`: Tests localhost resolution
- `test_concurrent_resolution`: Tests performance of concurrent resolution

#### Configuration Tests (`src/config.rs`)
- `test_default_config`: Tests default configuration values
- `test_config_serialization`: Tests JSON serialization/deserialization
- `test_config_bind_addr`: Tests bind address parsing
- `test_config_bind_addr_invalid`: Tests invalid address handling
- `test_config_load_from_file`: Tests loading configuration from file
- `test_config_load_default_when_file_missing`: Tests default config creation
- `test_config_custom_values`: Tests custom configuration values

#### Service Tests (`src/service.rs`)
- `test_is_service_mode_false`: Tests service mode detection
- `test_is_service_mode_true`: Tests service mode detection
- `test_run_as_service`: Tests service execution

### 2. Integration Tests (`tests/integration_tests.rs`)

Tests the complete HTTP API functionality:

- `test_health_endpoint`: Tests health check endpoint
- `test_root_endpoint`: Tests root endpoint
- `test_dns_resolve_single_host`: Tests single host resolution
- `test_dns_resolve_multiple_hosts`: Tests multiple host resolution
- `test_dns_resolve_empty_hosts`: Tests empty host list handling
- `test_dns_resolve_invalid_json`: Tests invalid JSON handling
- `test_dns_resolve_missing_content_type`: Tests content type handling
- `test_dns_resolve_performance`: Tests performance with many hosts
- `test_not_found_endpoint`: Tests 404 handling

### 3. API Tests (`tests/api_tests.rs`)

Tests specific API functionality:

- `test_api_endpoints_exist`: Tests all endpoints are accessible
- `test_api_response_formats`: Tests response format validation
- `test_api_error_handling`: Tests error response handling
- `test_api_concurrent_requests`: Tests concurrent request handling
- `test_api_large_request`: Tests large request handling
- `test_api_mixed_success_failure`: Tests mixed success/failure scenarios

### 4. Load Tests (`tests/load_tests.rs`)

Tests performance under various load conditions:

- `test_load_concurrent_requests`: Tests 20 concurrent requests
- `test_load_rapid_requests`: Tests 50 rapid requests
- `test_load_large_requests`: Tests requests with many hosts
- `test_load_mixed_workload`: Tests mixed request types
- `test_load_sustained_requests`: Tests sustained load over time

## Test Dependencies

The test suite uses the following dependencies:

```toml
[dev-dependencies]
tempfile = "3.0"      # For temporary file operations
tokio-test = "0.4"    # For async testing utilities
```

## Test Data

### Valid Test Hosts
- `google.com` - Reliable, fast resolution
- `github.com` - Popular service
- `stackoverflow.com` - Well-known domain
- `localhost` - Local resolution test

### Invalid Test Hosts
- `invalid-host-that-does-not-exist.example` - Non-existent domain
- `another-invalid-host.example` - Another non-existent domain

## Performance Expectations

### DNS Resolution
- Single host resolution: < 1 second
- Multiple host resolution: < 5 seconds (concurrent)
- 10 hosts resolution: < 3 seconds
- 50 hosts resolution: < 10 seconds

### HTTP API
- Health check: < 100ms
- Single host API call: < 2 seconds
- Multiple host API call: < 5 seconds
- Concurrent requests: < 10 seconds for 20 requests

### Load Testing
- 20 concurrent requests: < 10 seconds
- 50 rapid requests: < 15 seconds
- 100 sustained requests: < 30 seconds

## Test Environment Setup

### Prerequisites
- Rust 1.70+ with stable toolchain
- Internet connection for DNS resolution tests
- Sufficient system resources for load tests

### Environment Variables
```bash
# Set log level for tests
export RUST_LOG=wdns=debug

# Run tests with output
export RUST_TEST_ARGS="--nocapture"
```

## Continuous Integration

### GitHub Actions Example
```yaml
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --verbose
      - run: cargo test --test integration_tests --verbose
      - run: cargo test --test api_tests --verbose
      - run: cargo test --test load_tests --verbose
```

## Troubleshooting Tests

### Common Issues

1. **DNS Resolution Failures**
   - Check internet connection
   - Verify DNS servers are accessible
   - Some tests may fail in restricted network environments

2. **Timeout Issues**
   - Increase timeout values for slow networks
   - Check system resource usage
   - Verify DNS server response times

3. **Load Test Failures**
   - Ensure sufficient system resources
   - Check for network bottlenecks
   - Verify DNS server capacity

### Debugging Tests

```bash
# Run with detailed output
RUST_LOG=debug cargo test -- --nocapture

# Run single test with output
cargo test test_resolve_single_host -- --nocapture

# Run tests with timeout
timeout 300 cargo test
```

## Test Coverage

The test suite covers:

- ✅ DNS resolution functionality
- ✅ HTTP API endpoints
- ✅ Error handling
- ✅ Configuration management
- ✅ Service lifecycle
- ✅ Performance characteristics
- ✅ Load handling
- ✅ Concurrent operations
- ✅ Edge cases and error conditions

## Adding New Tests

### Unit Test Example
```rust
#[tokio::test]
async fn test_new_functionality() {
    let resolver = DnsResolver::new().expect("Failed to create resolver");
    let result = resolver.resolve_host("example.com").await;
    
    assert_eq!(result.status, "success");
    assert!(!result.ip_addresses.is_empty());
}
```

### Integration Test Example
```rust
#[tokio::test]
async fn test_new_api_endpoint() {
    let routes = create_test_server().await.expect("Failed to create test server");
    
    let response = warp::test::request()
        .method("GET")
        .path("/new-endpoint")
        .reply(&routes)
        .await;
    
    assert_eq!(response.status(), 200);
}
```

## Test Best Practices

1. **Isolation**: Each test should be independent
2. **Cleanup**: Clean up resources after tests
3. **Timeouts**: Use appropriate timeouts for network operations
4. **Assertions**: Use specific assertions with clear error messages
5. **Coverage**: Test both success and failure cases
6. **Performance**: Include performance expectations in tests
7. **Documentation**: Document test purpose and expectations

## Test Results Interpretation

### Success Criteria
- All unit tests pass
- All integration tests pass
- All API tests pass
- Load tests complete within time limits
- No memory leaks or resource issues

### Failure Analysis
- Check network connectivity for DNS tests
- Verify system resources for load tests
- Review error messages for specific issues
- Check test environment setup

## Maintenance

### Regular Updates
- Update test dependencies regularly
- Review and update performance expectations
- Add tests for new features
- Remove obsolete tests

### Test Data Updates
- Update test hosts as needed
- Verify test hosts are still accessible
- Add new test scenarios as requirements change
