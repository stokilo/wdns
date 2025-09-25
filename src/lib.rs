// Library module for WDNS Service
// This allows the code to be used as both a library and binary

pub mod dns;
pub mod config;
pub mod service;
pub mod proxy;
pub mod socks5;
pub mod ssh_tunnel;

// Re-export main types for external use
pub use dns::{DnsResolver, DnsRequest, DnsResponse, DnsResult};
pub use config::{Config, SshTunnelConfig};
pub use service::{is_service_mode, run_as_service};
pub use proxy::ProxyServer;
pub use socks5::Socks5Server;
pub use ssh_tunnel::SshTunnelManager;
