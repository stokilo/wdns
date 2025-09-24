// Library module for WDNS Service
// This allows the code to be used as both a library and binary

pub mod dns;
pub mod config;
pub mod service;

// Re-export main types for external use
pub use dns::{DnsResolver, DnsRequest, DnsResponse, DnsResult};
pub use config::Config;
pub use service::{is_service_mode, run_as_service};
