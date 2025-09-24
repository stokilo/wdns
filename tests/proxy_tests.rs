use anyhow::Result;
use std::net::SocketAddr;
use wdns_service::{Config, ProxyServer};

// Helper function to create test proxy server
async fn create_test_proxy_server() -> Result<(ProxyServer, SocketAddr)> {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let proxy = ProxyServer::new(addr);
    Ok((proxy, addr))
}

#[tokio::test]
async fn test_proxy_server_creation() {
    let addr: SocketAddr = "127.0.0.1:9701".parse().unwrap();
    let proxy = ProxyServer::new(addr);
    assert_eq!(proxy.bind_addr, addr);
}

#[tokio::test]
async fn test_proxy_server_bind() {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let proxy = ProxyServer::new(addr);
    assert_eq!(proxy.bind_addr, addr);
}

#[tokio::test]
async fn test_config_proxy_settings() {
    let config = Config::default();
    assert_eq!(config.proxy_enabled, true);
    assert_eq!(config.proxy_bind_address, "0.0.0.0:9701");
    
    let proxy_addr = config.proxy_bind_addr().expect("Failed to parse proxy bind address");
    assert_eq!(proxy_addr.to_string(), "0.0.0.0:9701");
}

#[tokio::test]
async fn test_config_proxy_disabled() {
    let config = Config {
        bind_address: "0.0.0.0:9700".to_string(),
        dns_timeout_seconds: 10,
        max_concurrent_resolutions: 100,
        proxy_enabled: false,
        proxy_bind_address: "0.0.0.0:9701".to_string(),
    };
    
    assert_eq!(config.proxy_enabled, false);
    assert_eq!(config.proxy_bind_address, "0.0.0.0:9701");
}

#[tokio::test]
async fn test_config_custom_proxy_address() {
    let config = Config {
        bind_address: "0.0.0.0:9700".to_string(),
        dns_timeout_seconds: 10,
        max_concurrent_resolutions: 100,
        proxy_enabled: true,
        proxy_bind_address: "192.168.1.100:8080".to_string(),
    };
    
    assert_eq!(config.proxy_enabled, true);
    assert_eq!(config.proxy_bind_address, "192.168.1.100:8080");
    
    let proxy_addr = config.proxy_bind_addr().expect("Failed to parse proxy bind address");
    assert_eq!(proxy_addr.to_string(), "192.168.1.100:8080");
}

#[tokio::test]
async fn test_config_serialization_with_proxy() {
    let config = Config::default();
    let json = serde_json::to_string(&config).expect("Failed to serialize config");
    let deserialized: Config = serde_json::from_str(&json).expect("Failed to deserialize config");
    
    assert_eq!(config.proxy_enabled, deserialized.proxy_enabled);
    assert_eq!(config.proxy_bind_address, deserialized.proxy_bind_address);
}

#[tokio::test]
async fn test_proxy_server_with_custom_config() {
    let config = Config {
        bind_address: "0.0.0.0:9700".to_string(),
        dns_timeout_seconds: 5,
        max_concurrent_resolutions: 50,
        proxy_enabled: true,
        proxy_bind_address: "127.0.0.1:8080".to_string(),
    };
    
    let proxy_addr = config.proxy_bind_addr().expect("Failed to parse proxy bind address");
    let proxy = ProxyServer::new(proxy_addr);
    
    assert_eq!(proxy.bind_addr, proxy_addr);
    assert_eq!(proxy.bind_addr.to_string(), "127.0.0.1:8080");
}

#[tokio::test]
async fn test_proxy_server_multiple_instances() {
    let addr1: SocketAddr = "127.0.0.1:9701".parse().unwrap();
    let addr2: SocketAddr = "127.0.0.1:9702".parse().unwrap();
    
    let proxy1 = ProxyServer::new(addr1);
    let proxy2 = ProxyServer::new(addr2);
    
    assert_eq!(proxy1.bind_addr, addr1);
    assert_eq!(proxy2.bind_addr, addr2);
    assert_ne!(proxy1.bind_addr, proxy2.bind_addr);
}

#[tokio::test]
async fn test_proxy_server_different_addresses() {
    let addresses = vec![
        "127.0.0.1:9701",
        "0.0.0.0:9701", 
        "127.0.0.1:8080",
        "0.0.0.0:8080",
    ];
    
    for addr_str in addresses {
        let addr: SocketAddr = addr_str.parse().unwrap();
        let proxy = ProxyServer::new(addr);
        assert_eq!(proxy.bind_addr, addr);
        assert_eq!(proxy.bind_addr.to_string(), addr_str);
    }
}

#[tokio::test]
async fn test_config_load_with_proxy_settings() {
    use std::fs;
    use tempfile::TempDir;
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("config.json");
    
    // Create a test config file with proxy settings
    let test_config = Config {
        bind_address: "0.0.0.0:9700".to_string(),
        dns_timeout_seconds: 15,
        max_concurrent_resolutions: 200,
        proxy_enabled: true,
        proxy_bind_address: "0.0.0.0:8080".to_string(),
    };
    
    let config_json = serde_json::to_string_pretty(&test_config).expect("Failed to serialize");
    fs::write(&config_path, config_json).expect("Failed to write config file");
    
    // Change to temp directory and test loading
    let original_dir = std::env::current_dir().expect("Failed to get current dir");
    std::env::set_current_dir(&temp_dir).expect("Failed to change to temp dir");
    
    let loaded_config = Config::load().expect("Failed to load config");
    
    // Restore original directory
    std::env::set_current_dir(&original_dir).expect("Failed to restore original dir");
    
    assert_eq!(loaded_config.proxy_enabled, true);
    assert_eq!(loaded_config.proxy_bind_address, "0.0.0.0:8080");
    assert_eq!(loaded_config.dns_timeout_seconds, 15);
    assert_eq!(loaded_config.max_concurrent_resolutions, 200);
}

#[tokio::test]
async fn test_proxy_server_integration() {
    // This test verifies that the proxy server can be created and configured
    // without actually starting it (which would require more complex setup)
    let config = Config::default();
    
    assert!(config.proxy_enabled);
    let proxy_addr = config.proxy_bind_addr().expect("Failed to parse proxy bind address");
    let proxy = ProxyServer::new(proxy_addr);
    
    assert_eq!(proxy.bind_addr, proxy_addr);
    assert_eq!(proxy.bind_addr.to_string(), "0.0.0.0:9701");
}

#[tokio::test]
async fn test_proxy_server_error_handling() {
    // Test invalid proxy bind address
    let invalid_config = Config {
        bind_address: "0.0.0.0:9700".to_string(),
        dns_timeout_seconds: 10,
        max_concurrent_resolutions: 100,
        proxy_enabled: true,
        proxy_bind_address: "invalid-address".to_string(),
    };
    
    let result = invalid_config.proxy_bind_addr();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("invalid-address"));
}

#[tokio::test]
async fn test_proxy_server_with_different_ports() {
    let ports = vec![8080, 8081, 9000, 9001, 9701, 9702];
    
    for port in ports {
        let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
        let proxy = ProxyServer::new(addr);
        assert_eq!(proxy.bind_addr, addr);
        assert_eq!(proxy.bind_addr.port(), port);
    }
}

#[tokio::test]
async fn test_proxy_server_configuration_combinations() {
    let test_cases = vec![
        (true, "127.0.0.1:8080"),
        (true, "0.0.0.0:8080"),
        (false, "127.0.0.1:8080"),
        (false, "0.0.0.0:8080"),
        (true, "192.168.1.100:9000"),
        (false, "10.0.0.1:8080"),
    ];
    
    for (proxy_enabled, proxy_bind_address) in test_cases {
        let config = Config {
            bind_address: "0.0.0.0:9700".to_string(),
            dns_timeout_seconds: 10,
            max_concurrent_resolutions: 100,
            proxy_enabled,
            proxy_bind_address: proxy_bind_address.to_string(),
        };
        
        assert_eq!(config.proxy_enabled, proxy_enabled);
        assert_eq!(config.proxy_bind_address, proxy_bind_address);
        
        let proxy_addr = config.proxy_bind_addr().expect("Failed to parse proxy bind address");
        assert_eq!(proxy_addr.to_string(), proxy_bind_address);
        
        let proxy = ProxyServer::new(proxy_addr);
        assert_eq!(proxy.bind_addr, proxy_addr);
    }
}
