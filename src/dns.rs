use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::TokioAsyncResolver;
use futures_util::future;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsResult {
    pub host: String,
    pub ip_addresses: Vec<String>,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRequest {
    pub hosts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsResponse {
    pub results: Vec<DnsResult>,
    pub total_resolved: usize,
    pub total_errors: usize,
}

pub struct DnsResolver {
    resolver: TokioAsyncResolver,
    timeout_duration: Duration,
}

impl DnsResolver {
    pub fn new() -> Result<Self> {
        // Use system DNS configuration
        let resolver_config = ResolverConfig::default();
        let resolver_opts = ResolverOpts::default();
        
        let resolver = TokioAsyncResolver::tokio(resolver_config, resolver_opts);
        
        Ok(Self {
            resolver,
            timeout_duration: Duration::from_secs(10),
        })
    }

    pub async fn resolve_host(&self, host: &str) -> DnsResult {
        let host = host.to_string();
        
        match timeout(self.timeout_duration, self.resolver.lookup_ip(&host)).await {
            Ok(Ok(lookup)) => {
                let ip_addresses: Vec<String> = lookup
                    .iter()
                    .map(|ip| ip.to_string())
                    .collect();
                
                DnsResult {
                    host,
                    ip_addresses,
                    status: "success".to_string(),
                    error: None,
                }
            }
            Ok(Err(e)) => DnsResult {
                host,
                ip_addresses: vec![],
                status: "error".to_string(),
                error: Some(e.to_string()),
            },
            Err(_) => DnsResult {
                host,
                ip_addresses: vec![],
                status: "timeout".to_string(),
                error: Some("DNS resolution timeout".to_string()),
            },
        }
    }

    pub async fn resolve_hosts(&self, hosts: Vec<String>) -> DnsResponse {
        let mut results = Vec::new();
        let mut total_resolved = 0;
        let mut total_errors = 0;

        // Resolve all hosts concurrently
        let futures: Vec<_> = hosts
            .iter()
            .map(|host| self.resolve_host(host))
            .collect();

        let resolved_results = future::join_all(futures).await;

        for result in resolved_results {
            if result.status == "success" {
                total_resolved += 1;
            } else {
                total_errors += 1;
            }
            results.push(result);
        }

        DnsResponse {
            results,
            total_resolved,
            total_errors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_resolve_single_host() {
        let resolver = DnsResolver::new().expect("Failed to create resolver");
        let result = resolver.resolve_host("google.com").await;
        
        assert_eq!(result.host, "google.com");
        assert_eq!(result.status, "success");
        assert!(!result.ip_addresses.is_empty());
        assert!(result.error.is_none());
    }

    #[tokio::test]
    async fn test_resolve_invalid_host() {
        let resolver = DnsResolver::new().expect("Failed to create resolver");
        let result = resolver.resolve_host("invalid-host-that-does-not-exist.example").await;
        
        assert_eq!(result.host, "invalid-host-that-does-not-exist.example");
        assert_eq!(result.status, "error");
        assert!(result.ip_addresses.is_empty());
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_resolve_multiple_hosts() {
        let resolver = DnsResolver::new().expect("Failed to create resolver");
        let hosts = vec![
            "google.com".to_string(),
            "github.com".to_string(),
            "invalid-host-that-does-not-exist.example".to_string(),
        ];
        
        let response = resolver.resolve_hosts(hosts).await;
        
        assert_eq!(response.results.len(), 3);
        assert_eq!(response.total_resolved, 2);
        assert_eq!(response.total_errors, 1);
        
        // Check individual results
        let google_result = response.results.iter().find(|r| r.host == "google.com").unwrap();
        assert_eq!(google_result.status, "success");
        assert!(!google_result.ip_addresses.is_empty());
        
        let github_result = response.results.iter().find(|r| r.host == "github.com").unwrap();
        assert_eq!(github_result.status, "success");
        assert!(!github_result.ip_addresses.is_empty());
        
        let invalid_result = response.results.iter().find(|r| r.host == "invalid-host-that-does-not-exist.example").unwrap();
        assert_eq!(invalid_result.status, "error");
        assert!(invalid_result.ip_addresses.is_empty());
        assert!(invalid_result.error.is_some());
    }

    #[tokio::test]
    async fn test_resolve_empty_hosts() {
        let resolver = DnsResolver::new().expect("Failed to create resolver");
        let response = resolver.resolve_hosts(vec![]).await;
        
        assert_eq!(response.results.len(), 0);
        assert_eq!(response.total_resolved, 0);
        assert_eq!(response.total_errors, 0);
    }

    #[tokio::test]
    async fn test_resolve_localhost() {
        let resolver = DnsResolver::new().expect("Failed to create resolver");
        let result = resolver.resolve_host("localhost").await;
        
        assert_eq!(result.host, "localhost");
        assert_eq!(result.status, "success");
        assert!(!result.ip_addresses.is_empty());
        // localhost should resolve to 127.0.0.1 or ::1
        assert!(result.ip_addresses.iter().any(|ip| ip == "127.0.0.1" || ip == "::1"));
    }

    #[tokio::test]
    async fn test_concurrent_resolution() {
        let resolver = DnsResolver::new().expect("Failed to create resolver");
        let hosts = vec![
            "google.com".to_string(),
            "github.com".to_string(),
            "stackoverflow.com".to_string(),
            "microsoft.com".to_string(),
            "amazon.com".to_string(),
        ];
        
        let start = std::time::Instant::now();
        let response = resolver.resolve_hosts(hosts).await;
        let duration = start.elapsed();
        
        // Should resolve all hosts successfully
        assert_eq!(response.results.len(), 5);
        assert_eq!(response.total_resolved, 5);
        assert_eq!(response.total_errors, 0);
        
        // Should be fast due to concurrent resolution
        assert!(duration.as_secs() < 5, "Resolution took too long: {:?}", duration);
        
        // All results should be successful
        for result in response.results {
            assert_eq!(result.status, "success");
            assert!(!result.ip_addresses.is_empty());
        }
    }
}