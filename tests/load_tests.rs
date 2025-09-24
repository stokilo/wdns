use anyhow::Result;
use serde_json;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{sleep, Duration};
use warp::Filter;

use wdns_service::{DnsResolver, DnsRequest};

// Helper function to create test server
async fn create_test_server() -> Result<impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone> {
    let dns_resolver = Arc::new(DnsResolver::new()?);
    
    // Health check endpoint
    let health = warp::path("health")
        .and(warp::get())
        .map(|| warp::reply::json(&serde_json::json!({
            "status": "healthy",
            "service": "wdns"
        })));

    // Root endpoint
    let root = warp::path::end()
        .and(warp::get())
        .map(|| warp::reply::json(&serde_json::json!({
            "service": "WDNS",
            "version": "0.1.0",
            "endpoints": ["/health", "/api/dns/resolve"]
        })));

    // DNS resolution endpoint
    let dns_resolver_filter = warp::any().map(move || dns_resolver.clone());
    
    let dns_resolve = warp::path("api")
        .and(warp::path("dns"))
        .and(warp::path("resolve"))
        .and(warp::post())
        .and(warp::body::json())
        .and(dns_resolver_filter)
        .and_then(handle_dns_resolve);

    let routes = health.or(root).or(dns_resolve);
    Ok(routes)
}

async fn handle_dns_resolve(
    request: DnsRequest,
    dns_resolver: Arc<DnsResolver>,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Validate request
    if request.hosts.is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "error": "No hosts provided"
            })),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Resolve DNS
    let dns_response = dns_resolver.resolve_hosts(request.hosts).await;

    Ok(warp::reply::with_status(
        warp::reply::json(&dns_response),
        warp::http::StatusCode::OK,
    ))
}

#[tokio::test]
async fn test_load_concurrent_requests() {
    let routes = create_test_server().await.expect("Failed to create test server");
    
    let start = Instant::now();
    let mut handles = vec![];
    
    // Create 20 concurrent requests
    for i in 0..20 {
        let routes_clone = routes.clone();
        let handle = tokio::spawn(async move {
            let response = warp::test::request()
                .method("POST")
                .path("/api/dns/resolve")
                .header("content-type", "application/json")
                .json(&serde_json::json!({
                    "hosts": ["google.com", "github.com", "stackoverflow.com"]
                }))
                .reply(&routes_clone)
                .await;
            
            (i, response.status(), response.body().len())
        });
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    let mut results = vec![];
    for handle in handles {
        let result = handle.await.expect("Request failed");
        results.push(result);
    }
    
    let duration = start.elapsed();
    
    // All requests should succeed
    for (request_id, status, body_len) in results {
        assert_eq!(status, 200, "Request {} failed with status {}", request_id, status);
        assert!(body_len > 0, "Request {} returned empty body", request_id);
    }
    
    // Should complete within reasonable time
    assert!(duration.as_secs() < 15, "Load test took too long: {:?}", duration);
    
    println!("Load test completed in {:?} with 20 concurrent requests", duration);
}

#[tokio::test]
async fn test_load_rapid_requests() {
    let routes = create_test_server().await.expect("Failed to create test server");
    
    let start = Instant::now();
    let mut handles = vec![];
    
    // Create 50 rapid requests
    for i in 0..50 {
        let routes_clone = routes.clone();
        let handle = tokio::spawn(async move {
            let response = warp::test::request()
                .method("POST")
                .path("/api/dns/resolve")
                .header("content-type", "application/json")
                .json(&serde_json::json!({
                    "hosts": ["google.com"]
                }))
                .reply(&routes_clone)
                .await;
            
            (i, response.status())
        });
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    let mut results = vec![];
    for handle in handles {
        let result = handle.await.expect("Request failed");
        results.push(result);
    }
    
    let duration = start.elapsed();
    
    // All requests should succeed
    for (request_id, status) in results {
        assert_eq!(status, 200, "Request {} failed with status {}", request_id, status);
    }
    
    // Should complete within reasonable time
    assert!(duration.as_secs() < 15, "Rapid requests test took too long: {:?}", duration);
    
    println!("Rapid requests test completed in {:?} with 50 requests", duration);
}

#[tokio::test]
async fn test_load_large_requests() {
    let routes = create_test_server().await.expect("Failed to create test server");
    
    let start = Instant::now();
    let mut handles = vec![];
    
    // Create 10 requests with many hosts each
    for i in 0..10 {
        let routes_clone = routes.clone();
        let handle = tokio::spawn(async move {
            let hosts: Vec<String> = (0..20).map(|j| format!("host{}.example.com", j)).collect();
            
            let response = warp::test::request()
                .method("POST")
                .path("/api/dns/resolve")
                .header("content-type", "application/json")
                .json(&serde_json::json!({
                    "hosts": hosts
                }))
                .reply(&routes_clone)
                .await;
            
            (i, response.status())
        });
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    let mut results = vec![];
    for handle in handles {
        let result = handle.await.expect("Request failed");
        results.push(result);
    }
    
    let duration = start.elapsed();
    
    // All requests should succeed
    for (request_id, status) in results {
        assert_eq!(status, 200, "Request {} failed with status {}", request_id, status);
    }
    
    // Should complete within reasonable time
    assert!(duration.as_secs() < 20, "Large requests test took too long: {:?}", duration);
    
    println!("Large requests test completed in {:?} with 10 requests of 20 hosts each", duration);
}

#[tokio::test]
async fn test_load_mixed_workload() {
    let routes = create_test_server().await.expect("Failed to create test server");
    
    let start = Instant::now();
    let mut handles = vec![];
    
    // Mix of different request types
    for i in 0..30 {
        let routes_clone = routes.clone();
        let handle = tokio::spawn(async move {
            let request_type = i % 3;
            let response = match request_type {
                0 => {
                    // Single host
                    warp::test::request()
                        .method("POST")
                        .path("/api/dns/resolve")
                        .header("content-type", "application/json")
                        .json(&serde_json::json!({
                            "hosts": ["google.com"]
                        }))
                        .reply(&routes_clone)
                        .await
                }
                1 => {
                    // Multiple hosts
                    warp::test::request()
                        .method("POST")
                        .path("/api/dns/resolve")
                        .header("content-type", "application/json")
                        .json(&serde_json::json!({
                            "hosts": ["google.com", "github.com", "stackoverflow.com"]
                        }))
                        .reply(&routes_clone)
                        .await
                }
                _ => {
                    // Health check
                    warp::test::request()
                        .method("GET")
                        .path("/health")
                        .reply(&routes_clone)
                        .await
                }
            };
            
            (i, response.status())
        });
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    let mut results = vec![];
    for handle in handles {
        let result = handle.await.expect("Request failed");
        results.push(result);
    }
    
    let duration = start.elapsed();
    
    // All requests should succeed
    for (request_id, status) in results {
        assert_eq!(status, 200, "Request {} failed with status {}", request_id, status);
    }
    
    // Should complete within reasonable time
    assert!(duration.as_secs() < 15, "Mixed workload test took too long: {:?}", duration);
    
    println!("Mixed workload test completed in {:?} with 30 mixed requests", duration);
}

#[tokio::test]
async fn test_load_sustained_requests() {
    let routes = create_test_server().await.expect("Failed to create test server");
    
    let start = Instant::now();
    let mut handles = vec![];
    
    // Create sustained requests over time
    for i in 0..100 {
        let routes_clone = routes.clone();
        let handle = tokio::spawn(async move {
            // Add small delay to simulate real-world usage
            sleep(Duration::from_millis(10)).await;
            
            let response = warp::test::request()
                .method("POST")
                .path("/api/dns/resolve")
                .header("content-type", "application/json")
                .json(&serde_json::json!({
                    "hosts": ["google.com", "github.com"]
                }))
                .reply(&routes_clone)
                .await;
            
            (i, response.status())
        });
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    let mut results = vec![];
    for handle in handles {
        let result = handle.await.expect("Request failed");
        results.push(result);
    }
    
    let duration = start.elapsed();
    
    // All requests should succeed
    for (request_id, status) in results {
        assert_eq!(status, 200, "Request {} failed with status {}", request_id, status);
    }
    
    // Should complete within reasonable time
    assert!(duration.as_secs() < 30, "Sustained requests test took too long: {:?}", duration);
    
    println!("Sustained requests test completed in {:?} with 100 requests", duration);
}
