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
async fn test_health_endpoint() {
    let routes = create_test_server().await.expect("Failed to create test server");
    
    let response = warp::test::request()
        .method("GET")
        .path("/health")
        .reply(&routes)
        .await;
    
    assert_eq!(response.status(), 200);
    
    let body = String::from_utf8(response.body().to_vec()).expect("Invalid UTF-8");
    let json: serde_json::Value = serde_json::from_str(&body).expect("Invalid JSON");
    
    assert_eq!(json["status"], "healthy");
    assert_eq!(json["service"], "wdns");
}

#[tokio::test]
async fn test_root_endpoint() {
    let routes = create_test_server().await.expect("Failed to create test server");
    
    let response = warp::test::request()
        .method("GET")
        .path("/")
        .reply(&routes)
        .await;
    
    assert_eq!(response.status(), 200);
    
    let body = String::from_utf8(response.body().to_vec()).expect("Invalid UTF-8");
    let json: serde_json::Value = serde_json::from_str(&body).expect("Invalid JSON");
    
    assert_eq!(json["service"], "WDNS");
    assert_eq!(json["version"], "0.1.0");
    assert!(json["endpoints"].is_array());
}

#[tokio::test]
async fn test_dns_resolve_single_host() {
    let routes = create_test_server().await.expect("Failed to create test server");
    
    let request_body = serde_json::json!({
        "hosts": ["google.com"]
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
    
    assert!(json["results"].is_array());
    assert_eq!(json["results"].as_array().unwrap().len(), 1);
    assert_eq!(json["total_resolved"], 1);
    assert_eq!(json["total_errors"], 0);
    
    let result = &json["results"][0];
    assert_eq!(result["host"], "google.com");
    assert_eq!(result["status"], "success");
    assert!(result["ip_addresses"].is_array());
    assert!(!result["ip_addresses"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_dns_resolve_multiple_hosts() {
    let routes = create_test_server().await.expect("Failed to create test server");
    
    let request_body = serde_json::json!({
        "hosts": ["google.com", "github.com", "invalid-host-that-does-not-exist.example"]
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
    
    assert!(json["results"].is_array());
    assert_eq!(json["results"].as_array().unwrap().len(), 3);
    assert_eq!(json["total_resolved"], 2);
    assert_eq!(json["total_errors"], 1);
    
    // Check individual results
    let results = json["results"].as_array().unwrap();
    
    let google_result = results.iter().find(|r| r["host"] == "google.com").unwrap();
    assert_eq!(google_result["status"], "success");
    assert!(!google_result["ip_addresses"].as_array().unwrap().is_empty());
    
    let github_result = results.iter().find(|r| r["host"] == "github.com").unwrap();
    assert_eq!(github_result["status"], "success");
    assert!(!github_result["ip_addresses"].as_array().unwrap().is_empty());
    
    let invalid_result = results.iter().find(|r| r["host"] == "invalid-host-that-does-not-exist.example").unwrap();
    assert_eq!(invalid_result["status"], "error");
    assert!(invalid_result["ip_addresses"].as_array().unwrap().is_empty());
    assert!(invalid_result["error"].is_string());
}

#[tokio::test]
async fn test_dns_resolve_empty_hosts() {
    let routes = create_test_server().await.expect("Failed to create test server");
    
    let request_body = serde_json::json!({
        "hosts": []
    });
    
    let response = warp::test::request()
        .method("POST")
        .path("/api/dns/resolve")
        .header("content-type", "application/json")
        .json(&request_body)
        .reply(&routes)
        .await;
    
    assert_eq!(response.status(), 400);
    
    let body = String::from_utf8(response.body().to_vec()).expect("Invalid UTF-8");
    let json: serde_json::Value = serde_json::from_str(&body).expect("Invalid JSON");
    
    assert_eq!(json["error"], "No hosts provided");
}

#[tokio::test]
async fn test_dns_resolve_invalid_json() {
    let routes = create_test_server().await.expect("Failed to create test server");
    
    let response = warp::test::request()
        .method("POST")
        .path("/api/dns/resolve")
        .header("content-type", "application/json")
        .body("invalid json")
        .reply(&routes)
        .await;
    
    assert_eq!(response.status(), 400);
    
    let body = String::from_utf8(response.body().to_vec()).expect("Invalid UTF-8");
    
    // The response might be JSON or plain text depending on the error
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
        assert!(json["error"].is_string());
        let error_msg = json["error"].as_str().unwrap();
        assert!(error_msg.contains("Invalid JSON") || error_msg.contains("expected value"));
    } else {
        // If it's not JSON, it should be an error message
        assert!(body.contains("error") || body.contains("Invalid"));
    }
}

#[tokio::test]
async fn test_dns_resolve_missing_content_type() {
    let routes = create_test_server().await.expect("Failed to create test server");
    
    let request_body = serde_json::json!({
        "hosts": ["google.com"]
    });
    
    let response = warp::test::request()
        .method("POST")
        .path("/api/dns/resolve")
        .json(&request_body)
        .reply(&routes)
        .await;
    
    // Should still work without explicit content-type header
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_dns_resolve_performance() {
    let routes = create_test_server().await.expect("Failed to create test server");
    
    let hosts = vec![
        "google.com", "github.com", "stackoverflow.com", 
        "microsoft.com", "amazon.com", "netflix.com",
        "spotify.com", "twitter.com", "linkedin.com", "reddit.com"
    ];
    
    let request_body = serde_json::json!({
        "hosts": hosts
    });
    
    let start = std::time::Instant::now();
    
    let response = warp::test::request()
        .method("POST")
        .path("/api/dns/resolve")
        .header("content-type", "application/json")
        .json(&request_body)
        .reply(&routes)
        .await;
    
    let duration = start.elapsed();
    
    assert_eq!(response.status(), 200);
    
    let body = String::from_utf8(response.body().to_vec()).expect("Invalid UTF-8");
    let json: serde_json::Value = serde_json::from_str(&body).expect("Invalid JSON");
    
    assert_eq!(json["results"].as_array().unwrap().len(), 10);
    assert!(json["total_resolved"].as_u64().unwrap() >= 8); // Most should resolve
    assert!(duration.as_secs() < 5); // Should be fast due to concurrent resolution
}

#[tokio::test]
async fn test_not_found_endpoint() {
    let routes = create_test_server().await.expect("Failed to create test server");
    
    let response = warp::test::request()
        .method("GET")
        .path("/nonexistent")
        .reply(&routes)
        .await;
    
    // Warp returns 404 for unmatched routes
    assert_eq!(response.status(), 404);
}
