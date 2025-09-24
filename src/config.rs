use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub bind_address: String,
    pub dns_timeout_seconds: u64,
    pub max_concurrent_resolutions: usize,
    pub proxy_enabled: bool,
    pub proxy_bind_address: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:9700".to_string(),
            dns_timeout_seconds: 10,
            max_concurrent_resolutions: 100,
            proxy_enabled: true,
            proxy_bind_address: "0.0.0.0:9701".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        // Try to load from config file, fallback to defaults
        if let Ok(config_str) = std::fs::read_to_string("config.json") {
            let config: Config = serde_json::from_str(&config_str)?;
            Ok(config)
        } else {
            // Create default config file
            let config = Config::default();
            let config_str = serde_json::to_string_pretty(&config)?;
            std::fs::write("config.json", config_str)?;
            Ok(config)
        }
    }

    pub fn bind_addr(&self) -> anyhow::Result<SocketAddr> {
        self.bind_address.parse()
            .map_err(|e| anyhow::anyhow!("Invalid bind address '{}': {}", self.bind_address, e))
    }

    pub fn proxy_bind_addr(&self) -> anyhow::Result<SocketAddr> {
        self.proxy_bind_address.parse()
            .map_err(|e| anyhow::anyhow!("Invalid proxy bind address '{}': {}", self.proxy_bind_address, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.bind_address, "0.0.0.0:9700");
        assert_eq!(config.dns_timeout_seconds, 10);
        assert_eq!(config.max_concurrent_resolutions, 100);
        assert_eq!(config.proxy_enabled, true);
        assert_eq!(config.proxy_bind_address, "0.0.0.0:9701");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = serde_json::to_string(&config).expect("Failed to serialize config");
        let deserialized: Config = serde_json::from_str(&json).expect("Failed to deserialize config");
        
        assert_eq!(config.bind_address, deserialized.bind_address);
        assert_eq!(config.dns_timeout_seconds, deserialized.dns_timeout_seconds);
        assert_eq!(config.max_concurrent_resolutions, deserialized.max_concurrent_resolutions);
    }

    #[test]
    fn test_config_bind_addr() {
        let config = Config::default();
        let addr = config.bind_addr().expect("Failed to parse bind address");
        assert_eq!(addr.to_string(), "0.0.0.0:9700");
    }

    #[test]
    fn test_config_bind_addr_invalid() {
        let mut config = Config::default();
        config.bind_address = "invalid-address".to_string();
        
        let result = config.bind_addr();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid-address"));
    }

    #[test]
    fn test_config_load_from_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.json");
        
        // Create a test config file
        let test_config = Config {
            bind_address: "0.0.0.0:9090".to_string(),
            dns_timeout_seconds: 30,
            max_concurrent_resolutions: 200,
            proxy_enabled: true,
            proxy_bind_address: "0.0.0.0:9091".to_string(),
        };
        
        let config_json = serde_json::to_string_pretty(&test_config).expect("Failed to serialize");
        fs::write(&config_path, config_json).expect("Failed to write config file");
        
        // Change to temp directory and test loading
        let original_dir = std::env::current_dir().expect("Failed to get current dir");
        std::env::set_current_dir(&temp_dir).expect("Failed to change to temp dir");
        
        let loaded_config = Config::load().expect("Failed to load config");
        
        // Restore original directory
        std::env::set_current_dir(&original_dir).expect("Failed to restore original dir");
        
        assert_eq!(loaded_config.bind_address, "0.0.0.0:9090");
        assert_eq!(loaded_config.dns_timeout_seconds, 30);
        assert_eq!(loaded_config.max_concurrent_resolutions, 200);
    }

    #[test]
    fn test_config_load_default_when_file_missing() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let original_dir = std::env::current_dir().expect("Failed to get current dir");
        std::env::set_current_dir(&temp_dir).expect("Failed to change to temp dir");
        
        let config = Config::load().expect("Failed to load config");
        
        // Restore original directory
        std::env::set_current_dir(&original_dir).expect("Failed to restore original dir");
        
        // Should create default config
        assert_eq!(config.bind_address, "0.0.0.0:9700");
        assert_eq!(config.dns_timeout_seconds, 10);
        assert_eq!(config.max_concurrent_resolutions, 100);
        
        // Should create config.json file
        assert!(temp_dir.path().join("config.json").exists());
    }

    #[test]
    fn test_config_custom_values() {
        let config = Config {
            bind_address: "192.168.1.100:3000".to_string(),
            dns_timeout_seconds: 5,
            max_concurrent_resolutions: 50,
            proxy_enabled: false,
            proxy_bind_address: "192.168.1.100:3001".to_string(),
        };
        
        assert_eq!(config.bind_address, "192.168.1.100:3000");
        assert_eq!(config.dns_timeout_seconds, 5);
        assert_eq!(config.max_concurrent_resolutions, 50);
        assert_eq!(config.proxy_enabled, false);
        assert_eq!(config.proxy_bind_address, "192.168.1.100:3001");
        
        let addr = config.bind_addr().expect("Failed to parse bind address");
        assert_eq!(addr.to_string(), "192.168.1.100:3000");
        
        let proxy_addr = config.proxy_bind_addr().expect("Failed to parse proxy bind address");
        assert_eq!(proxy_addr.to_string(), "192.168.1.100:3001");
    }
}