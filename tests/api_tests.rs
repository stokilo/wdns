use anyhow::Result;
use serde_json;
use std::sync::Arc;
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
async fn test_api_endpoints_exist() {
    let routes = create_test_server().await.expect("Failed to create test server");
    
    // Test health endpoint
    let health_response = warp::test::request()
        .method("GET")
        .path("/health")
        .reply(&routes)
        .await;
    assert_eq!(health_response.status(), 200);
    
    // Test root endpoint
    let root_response = warp::test::request()
        .method("GET")
        .path("/")
        .reply(&routes)
        .await;
    assert_eq!(root_response.status(), 200);
    
    // Test DNS resolve endpoint
    let dns_response = warp::test::request()
        .method("POST")
        .path("/api/dns/resolve")
        .header("content-type", "application/json")
        .json(&serde_json::json!({"hosts": ["google.com"]}))
        .reply(&routes)
        .await;
    assert_eq!(dns_response.status(), 200);
}

#[tokio::test]
async fn test_api_response_formats() {
    let routes = create_test_server().await.expect("Failed to create test server");
    
    // Test health response format
    let health_response = warp::test::request()
        .method("GET")
        .path("/health")
        .reply(&routes)
        .await;
    
    let body = String::from_utf8(health_response.body().to_vec()).expect("Invalid UTF-8");
    let json: serde_json::Value = serde_json::from_str(&body).expect("Invalid JSON");
    
    assert!(json["status"].is_string());
    assert!(json["service"].is_string());
    assert_eq!(json["status"], "healthy");
    assert_eq!(json["service"], "wdns");
}

#[tokio::test]
async fn test_api_error_handling() {
    let routes = create_test_server().await.expect("Failed to create test server");
    
    // Test empty hosts
    let response = warp::test::request()
        .method("POST")
        .path("/api/dns/resolve")
        .header("content-type", "application/json")
        .json(&serde_json::json!({"hosts": []}))
        .reply(&routes)
        .await;
    
    assert_eq!(response.status(), 400);
    
    let body = String::from_utf8(response.body().to_vec()).expect("Invalid UTF-8");
    let json: serde_json::Value = serde_json::from_str(&body).expect("Invalid JSON");
    assert_eq!(json["error"], "No hosts provided");
}

#[tokio::test]
async fn test_api_concurrent_requests() {
    let routes = create_test_server().await.expect("Failed to create test server");
    
    // Create multiple concurrent requests
    let mut handles = vec![];
    
    for i in 0..5 {
        let routes_clone = routes.clone();
        let handle = tokio::spawn(async move {
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
    for handle in handles {
        let (request_id, status) = handle.await.expect("Request failed");
        assert_eq!(status, 200, "Request {} failed", request_id);
    }
}

#[tokio::test]
async fn test_api_large_request() {
    let routes = create_test_server().await.expect("Failed to create test server");
    
    // Create a large request with many hosts
    let hosts: Vec<String> = (0..50).map(|i| format!("host{}.example.com", i)).collect();
    
    let request_body = serde_json::json!({
        "hosts": hosts
    });
    
    let response = warp::test::request()
        .method("POST")
        .path("/api/dns/resolve")
        .header("content-type", "application/json")
        .json(&request_body)
        .reply(&routes)
        .await;
    
    assert_eq!(response.status(), 200);
    
    let body = String::from_utf8(response.body().to_vec()).expect("Invalid UTF-8");
    let json: serde_json::Value = serde_json::from_str(&body).expect("Invalid JSON");
    
    assert_eq!(json["results"].as_array().unwrap().len(), 50);
    assert_eq!(json["total_errors"], 50); // All should fail since they don't exist
    assert_eq!(json["total_resolved"], 0);
}

#[tokio::test]
async fn test_api_mixed_success_failure() {
    let routes = create_test_server().await.expect("Failed to create test server");
    
    let request_body = serde_json::json!({
        "hosts": [
            "google.com",                    // Should succeed
            "github.com",                    // Should succeed
            "invalid-host.example",          // Should fail
            "another-invalid-host.example",  // Should fail
            "stackoverflow.com"              // Should succeed
        ]
    });
    
    let response = warp::test::request()
        .method("POST")
        .path("/api/dns/resolve")
        .header("content-type", "application/json")
        .json(&request_body)
        .reply(&routes)
        .await;
    
    assert_eq!(response.status(), 200);
    
    let body = String::from_utf8(response.body().to_vec()).expect("Invalid UTF-8");
    let json: serde_json::Value = serde_json::from_str(&body).expect("Invalid JSON");
    
    assert_eq!(json["results"].as_array().unwrap().len(), 5);
    assert_eq!(json["total_resolved"], 3);
    assert_eq!(json["total_errors"], 2);
    
    // Check individual results
    let results = json["results"].as_array().unwrap();
    
    let google_result = results.iter().find(|r| r["host"] == "google.com").unwrap();
    assert_eq!(google_result["status"], "success");
    
    let github_result = results.iter().find(|r| r["host"] == "github.com").unwrap();
    assert_eq!(github_result["status"], "success");
    
    let stackoverflow_result = results.iter().find(|r| r["host"] == "stackoverflow.com").unwrap();
    assert_eq!(stackoverflow_result["status"], "success");
    
    let invalid_result = results.iter().find(|r| r["host"] == "invalid-host.example").unwrap();
    assert_eq!(invalid_result["status"], "error");
    
    let another_invalid_result = results.iter().find(|r| r["host"] == "another-invalid-host.example").unwrap();
    assert_eq!(another_invalid_result["status"], "error");
}
