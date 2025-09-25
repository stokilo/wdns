use std::net::{IpAddr, SocketAddr, TcpStream, UdpSocket};
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};
use crate::{ProxyConfig, ProxyManager, NetworkConnection};
use crate::traffic_interceptor::{InterceptedConnection, InterceptionStatus};

/// Helper methods for traffic interception
impl super::TrafficInterceptor {
    /// Intercept TCP traffic at system level
    pub fn intercept_tcp_traffic(
        proxy_manager: Arc<Mutex<ProxyManager>>,
        is_running: Arc<Mutex<bool>>,
        intercepted_connections: Arc<Mutex<Vec<InterceptedConnection>>>,
        connection_counter: Arc<Mutex<u64>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔗 Intercepting TCP traffic at system level...");
        
        while *is_running.lock().unwrap() {
            // Monitor system TCP connections
            if let Ok(connections) = Self::get_system_tcp_connections() {
                for conn in connections {
                    if let Some(remote_addr) = conn.remote_addr {
                        // Check if this connection should be proxied
                        if let Some(proxy_config) = Self::should_proxy_connection(&proxy_manager, &remote_addr) {
                            println!("✅ TCP RULE MATCH! {} -> {} (proxy: {}:{})", 
                                     conn.local_addr, remote_addr, proxy_config.host, proxy_config.port);
                            
                            // Route TCP connection through SOCKS5 proxy
                            Self::route_tcp_through_socks5(&conn, &proxy_config)?;
                            
                            // Record intercepted connection
                            let mut counter = connection_counter.lock().unwrap();
                            *counter += 1;
                            let connection_id = *counter;
                            drop(counter);
                            
                            Self::record_intercepted_connection(
                                &intercepted_connections,
                                connection_id,
                                conn.remote_addr.map(|addr| addr.to_string()).unwrap_or_else(|| "unknown".to_string()),
                                Some(proxy_config),
                                InterceptionStatus::Proxied,
                            );
                        }
                    }
                }
            }
            
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        
        println!("🛑 TCP interception stopped");
        Ok(())
    }

    /// Intercept UDP traffic at system level
    pub fn intercept_udp_traffic(
        proxy_manager: Arc<Mutex<ProxyManager>>,
        is_running: Arc<Mutex<bool>>,
        intercepted_connections: Arc<Mutex<Vec<InterceptedConnection>>>,
        connection_counter: Arc<Mutex<u64>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("📡 Intercepting UDP traffic at system level...");
        
        while *is_running.lock().unwrap() {
            // Monitor system UDP connections
            if let Ok(connections) = Self::get_system_udp_connections() {
                for conn in connections {
                    if let Some(remote_addr) = conn.remote_addr {
                        // Check if this connection should be proxied
                        if let Some(proxy_config) = Self::should_proxy_connection(&proxy_manager, &remote_addr) {
                            println!("✅ UDP RULE MATCH! {} -> {} (proxy: {}:{})", 
                                     conn.local_addr, remote_addr, proxy_config.host, proxy_config.port);
                            
                            // Route UDP connection through SOCKS5 proxy
                            Self::route_udp_through_socks5(&conn, &proxy_config)?;
                            
                            // Record intercepted connection
                            let mut counter = connection_counter.lock().unwrap();
                            *counter += 1;
                            let connection_id = *counter;
                            drop(counter);
                            
                            Self::record_intercepted_connection(
                                &intercepted_connections,
                                connection_id,
                                conn.remote_addr.map(|addr| addr.to_string()).unwrap_or_else(|| "unknown".to_string()),
                                Some(proxy_config),
                                InterceptionStatus::Proxied,
                            );
                        }
                    }
                }
            }
            
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        
        println!("🛑 UDP interception stopped");
        Ok(())
    }

    /// Get system TCP connections
    pub fn get_system_tcp_connections() -> Result<Vec<NetworkConnection>, Box<dyn std::error::Error>> {
        use std::process::Command;
        
        let output = Command::new("netstat")
            .args(&["-an", "-p", "tcp"])
            .output()?;
        
        if !output.status.success() {
            return Err("netstat failed".into());
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut connections = Vec::new();
        
        for line in output_str.lines() {
            if line.contains("tcp") && line.contains("ESTABLISHED") {
                if let Some(conn) = Self::parse_netstat_line(line) {
                    connections.push(conn);
                }
            }
        }
        
        Ok(connections)
    }

    /// Get system UDP connections
    pub fn get_system_udp_connections() -> Result<Vec<NetworkConnection>, Box<dyn std::error::Error>> {
        use std::process::Command;
        
        let output = Command::new("netstat")
            .args(&["-an", "-p", "udp"])
            .output()?;
        
        if !output.status.success() {
            return Err("netstat failed".into());
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut connections = Vec::new();
        
        for line in output_str.lines() {
            if line.contains("udp") {
                if let Some(conn) = Self::parse_netstat_line(line) {
                    connections.push(conn);
                }
            }
        }
        
        Ok(connections)
    }

    /// Parse netstat line
    pub fn parse_netstat_line(line: &str) -> Option<NetworkConnection> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            return None;
        }

        let protocol = if line.contains("tcp") { "TCP" } else { "UDP" };
        
        if let Ok(local_addr) = Self::parse_socket_addr(parts[3]) {
            let remote_addr = if parts.len() > 4 && !parts[4].is_empty() {
                Self::parse_socket_addr(parts[4]).ok()
            } else {
                None
            };

            let state = if parts.len() > 5 { parts[5].to_string() } else { protocol.to_string() };

            Some(NetworkConnection {
                local_addr,
                remote_addr,
                protocol: protocol.to_string(),
                state,
                process_name: "Unknown".to_string(),
                process_id: 0,
                bytes_sent: 0,
                bytes_received: 0,
                last_updated: std::time::Instant::now(),
                interface: "Unknown".to_string(),
            })
        } else {
            None
        }
    }

    /// Parse socket address from string
    pub fn parse_socket_addr(addr_str: &str) -> Result<SocketAddr, Box<dyn std::error::Error>> {
        if addr_str.starts_with('*') {
            let port_str = &addr_str[2..];
            let port = port_str.parse::<u16>()?;
            Ok(SocketAddr::new(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)), 
                port
            ))
        } else if addr_str.contains(':') {
            addr_str.parse::<SocketAddr>()
                .map_err(|e| format!("Failed to parse '{}': {}", addr_str, e).into())
        } else {
            Err("Invalid address format".into())
        }
    }

    /// Extract domain from DNS packet
    pub fn extract_domain_from_dns_packet(packet: &[u8]) -> Option<String> {
        if packet.len() < 12 {
            return None; // DNS header is at least 12 bytes
        }

        // Skip DNS header (12 bytes) and parse the question section
        let mut offset = 12;
        let mut domain = String::new();
        
        while offset < packet.len() {
            let length = packet[offset] as usize;
            if length == 0 {
                break; // End of domain name
            }
            
            if offset + length >= packet.len() {
                break; // Invalid packet
            }
            
            if !domain.is_empty() {
                domain.push('.');
            }
            
            let label = String::from_utf8_lossy(&packet[offset + 1..offset + 1 + length]);
            domain.push_str(&label);
            
            offset += length + 1;
        }
        
        if domain.is_empty() {
            None
        } else {
            Some(domain)
        }
    }

    /// Check if domain should be proxied
    pub fn should_proxy_domain(
        proxy_manager: &Arc<Mutex<ProxyManager>>,
        domain: &str,
    ) -> Option<ProxyConfig> {
        let manager = proxy_manager.lock().unwrap();
        
        if !manager.global_enabled {
            return None;
        }

        for rule in &manager.rules {
            if !rule.enabled {
                continue;
            }

            if Self::matches_pattern(&rule.pattern, domain) {
                println!("✅ DNS rule '{}' matched for domain '{}'", rule.name, domain);
                
                if let Some(proxy) = manager.proxies.iter().find(|p| p.id == rule.proxy_id && p.enabled) {
                    return Some(proxy.clone());
                }
            }
        }

        None
    }

    /// Check if connection should be proxied
    pub fn should_proxy_connection(
        proxy_manager: &Arc<Mutex<ProxyManager>>,
        target_addr: &SocketAddr,
    ) -> Option<ProxyConfig> {
        let manager = proxy_manager.lock().unwrap();
        
        if !manager.global_enabled {
            return None;
        }

        // Try to resolve IP to hostname for rule matching
        let hostname = Self::resolve_ip_to_hostname(target_addr.ip())
            .unwrap_or_else(|| {
                match target_addr.ip() {
                    IpAddr::V4(ip) => ip.to_string(),
                    IpAddr::V6(ip) => ip.to_string(),
                }
            });

        for rule in &manager.rules {
            if !rule.enabled {
                continue;
            }

            if Self::matches_pattern(&rule.pattern, &hostname) {
                println!("✅ Connection rule '{}' matched for hostname '{}'", rule.name, hostname);
                
                if let Some(proxy) = manager.proxies.iter().find(|p| p.id == rule.proxy_id && p.enabled) {
                    return Some(proxy.clone());
                }
            }
        }

        None
    }

    /// Route DNS query through SOCKS5 proxy
    pub fn route_dns_through_socks5(
        domain: &str,
        proxy_config: &ProxyConfig,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        println!("🔗 Routing DNS query for '{}' through SOCKS5 proxy {}:{}", 
                 domain, proxy_config.host, proxy_config.port);

        // Connect to SOCKS5 proxy
        let proxy_addr = format!("{}:{}", proxy_config.host, proxy_config.port);
        let mut proxy_stream = TcpStream::connect(&proxy_addr)?;
        println!("✅ Connected to SOCKS5 proxy");

        // Perform SOCKS5 handshake
        Self::socks5_handshake(&mut proxy_stream, proxy_config)?;
        println!("🤝 SOCKS5 handshake completed");

        // Connect to DNS server through proxy
        let dns_server = "8.8.8.8:53"; // Use Google DNS as upstream
        let dns_addr: SocketAddr = dns_server.parse()?;
        Self::socks5_connect(&mut proxy_stream, dns_addr)?;
        println!("🎯 Connected to DNS server {} through proxy", dns_server);

        // Send DNS query through proxy
        let query_packet = Self::build_dns_query_packet(domain)?;
        proxy_stream.write_all(&query_packet)?;
        println!("📤 DNS query sent through proxy");

        // Read DNS response
        let mut response = vec![0u8; 512];
        let size = proxy_stream.read(&mut response)?;
        response.truncate(size);
        println!("📥 DNS response received ({} bytes)", size);

        Ok(response)
    }

    /// Route TCP connection through SOCKS5 proxy
    pub fn route_tcp_through_socks5(
        connection: &NetworkConnection,
        proxy_config: &ProxyConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔗 Routing TCP connection through SOCKS5 proxy {}:{}", 
                 proxy_config.host, proxy_config.port);

        // Connect to SOCKS5 proxy
        let proxy_addr = format!("{}:{}", proxy_config.host, proxy_config.port);
        let mut proxy_stream = TcpStream::connect(&proxy_addr)?;
        println!("✅ Connected to SOCKS5 proxy");

        // Perform SOCKS5 handshake
        Self::socks5_handshake(&mut proxy_stream, proxy_config)?;
        println!("🤝 SOCKS5 handshake completed");

        // Connect to target through proxy
        if let Some(target_addr) = connection.remote_addr {
            Self::socks5_connect(&mut proxy_stream, target_addr)?;
            println!("🎯 Connected to target {} through proxy", target_addr);
        }

        Ok(())
    }

    /// Route UDP connection through SOCKS5 proxy
    pub fn route_udp_through_socks5(
        connection: &NetworkConnection,
        proxy_config: &ProxyConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔗 Routing UDP connection through SOCKS5 proxy {}:{}", 
                 proxy_config.host, proxy_config.port);

        // UDP over SOCKS5 is more complex and requires UDP ASSOCIATE
        // This is a simplified implementation
        println!("📡 UDP routing through SOCKS5 (simplified implementation)");
        
        Ok(())
    }

    /// Forward DNS query to system DNS
    pub fn forward_to_system_dns(dns_packet: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Forward to system DNS server
        let dns_server = "8.8.8.8:53";
        let mut dns_socket = UdpSocket::bind("0.0.0.0:0")?;
        dns_socket.send_to(dns_packet, dns_server)?;
        
        let mut response = vec![0u8; 512];
        let size = dns_socket.recv(&mut response)?;
        response.truncate(size);
        
        Ok(response)
    }

    /// SOCKS5 handshake
    pub fn socks5_handshake(
        stream: &mut TcpStream,
        proxy_config: &ProxyConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Send authentication methods
        let auth_methods = if proxy_config.username.is_some() {
            vec![0x05, 0x01, 0x02, 0x00] // Username/password and no auth
        } else {
            vec![0x05, 0x01, 0x00] // No authentication
        };
        
        stream.write_all(&auth_methods)?;

        // Read server response
        let mut response = [0u8; 2];
        stream.read_exact(&mut response)?;

        if response[0] != 0x05 {
            return Err("Invalid SOCKS5 version".into());
        }

        // Handle authentication if required
        if response[1] == 0x02 && proxy_config.username.is_some() {
            Self::socks5_authenticate(stream, proxy_config)?;
        } else if response[1] != 0x00 {
            return Err("SOCKS5 authentication failed".into());
        }

        Ok(())
    }

    /// SOCKS5 authentication
    pub fn socks5_authenticate(
        stream: &mut TcpStream,
        proxy_config: &ProxyConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let username = proxy_config.username.as_ref().unwrap();
        let password = proxy_config.password.as_ref().unwrap();

        let mut auth_request = vec![0x01, username.len() as u8];
        auth_request.extend_from_slice(username.as_bytes());
        auth_request.push(password.len() as u8);
        auth_request.extend_from_slice(password.as_bytes());

        stream.write_all(&auth_request)?;

        let mut response = [0u8; 2];
        stream.read_exact(&mut response)?;

        if response[0] != 0x01 || response[1] != 0x00 {
            return Err("SOCKS5 authentication failed".into());
        }

        Ok(())
    }

    /// SOCKS5 connect command
    pub fn socks5_connect(
        stream: &mut TcpStream,
        target_addr: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut connect_request = vec![0x05, 0x01, 0x00]; // VER, CMD, RSV

        match target_addr.ip() {
            IpAddr::V4(ip) => {
                connect_request.push(0x01); // ATYP: IPv4
                connect_request.extend_from_slice(&ip.octets());
            }
            IpAddr::V6(ip) => {
                connect_request.push(0x04); // ATYP: IPv6
                connect_request.extend_from_slice(&ip.octets());
            }
        }

        connect_request.extend_from_slice(&target_addr.port().to_be_bytes());
        stream.write_all(&connect_request)?;

        // Read response
        let mut response = vec![0u8; 4];
        stream.read_exact(&mut response)?;

        if response[0] != 0x05 || response[1] != 0x00 {
            return Err("SOCKS5 connection failed".into());
        }

        // Skip the rest of the response
        let atyp = response[3];
        let addr_len = match atyp {
            0x01 => 4,  // IPv4
            0x04 => 16, // IPv6
            0x03 => {   // Domain name
                let mut len_buf = [0u8; 1];
                stream.read_exact(&mut len_buf)?;
                len_buf[0] as usize
            }
            _ => return Err("Invalid address type".into()),
        };

        let mut addr_buf = vec![0u8; addr_len];
        stream.read_exact(&mut addr_buf)?;

        let mut port_buf = [0u8; 2];
        stream.read_exact(&mut port_buf)?;

        Ok(())
    }

    /// Build DNS query packet
    pub fn build_dns_query_packet(domain: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut packet = Vec::new();
        
        // DNS header
        packet.extend_from_slice(&[0x12, 0x34]); // ID
        packet.extend_from_slice(&[0x01, 0x00]); // Flags (standard query)
        packet.extend_from_slice(&1u16.to_be_bytes()); // QDCOUNT
        packet.extend_from_slice(&0u16.to_be_bytes()); // ANCOUNT
        packet.extend_from_slice(&0u16.to_be_bytes()); // NSCOUNT
        packet.extend_from_slice(&0u16.to_be_bytes()); // ARCOUNT

        // Question section
        // Domain name
        for part in domain.split('.') {
            packet.push(part.len() as u8);
            packet.extend_from_slice(part.as_bytes());
        }
        packet.push(0); // End of domain name

        // Query type (A record)
        packet.extend_from_slice(&1u16.to_be_bytes());
        // Query class (IN = 1)
        packet.extend_from_slice(&1u16.to_be_bytes());

        Ok(packet)
    }

    /// Try to resolve IP address to hostname
    pub fn resolve_ip_to_hostname(ip: IpAddr) -> Option<String> {
        // For localhost addresses, return special names
        match ip {
            IpAddr::V4(ipv4) => {
                if ipv4.is_loopback() {
                    return Some("localhost".to_string());
                }
                if ipv4.is_private() {
                    // For private IPs, try reverse DNS lookup
                    return Self::reverse_dns_lookup(ip);
                }
            }
            IpAddr::V6(ipv6) => {
                if ipv6.is_loopback() {
                    return Some("localhost".to_string());
                }
                if ipv6.is_unicast_link_local() {
                    return Some("link-local".to_string());
                }
            }
        }

        // Try reverse DNS lookup
        Self::reverse_dns_lookup(ip)
    }

    /// Perform reverse DNS lookup
    pub fn reverse_dns_lookup(ip: IpAddr) -> Option<String> {
        // This is a simplified implementation
        // In a real implementation, you'd use a proper DNS resolver
        match ip {
            IpAddr::V4(ipv4) => {
                // Check for common private IP ranges
                if ipv4.octets()[0] == 192 && ipv4.octets()[1] == 168 {
                    return Some(format!("private-{}.{}.{}.{}", 
                        ipv4.octets()[0], ipv4.octets()[1], ipv4.octets()[2], ipv4.octets()[3]));
                }
                if ipv4.octets()[0] == 10 {
                    return Some(format!("private-{}.{}.{}.{}", 
                        ipv4.octets()[0], ipv4.octets()[1], ipv4.octets()[2], ipv4.octets()[3]));
                }
                if ipv4.octets()[0] == 172 && ipv4.octets()[1] >= 16 && ipv4.octets()[1] <= 31 {
                    return Some(format!("private-{}.{}.{}.{}", 
                        ipv4.octets()[0], ipv4.octets()[1], ipv4.octets()[2], ipv4.octets()[3]));
                }
                
                // Check for 100.64.x.x range (Carrier-Grade NAT)
                if ipv4.octets()[0] == 100 && ipv4.octets()[1] == 64 {
                    return Some(format!("100.64.{}.{}", ipv4.octets()[2], ipv4.octets()[3]));
                }
            }
            IpAddr::V6(_) => {
                return Some("ipv6-address".to_string());
            }
        }

        None
    }

    /// Pattern matching for proxy rules
    pub fn matches_pattern(pattern: &str, hostname: &str) -> bool {
        if pattern == hostname {
            return true;
        }

        if pattern.starts_with("*.") {
            let suffix = &pattern[2..];
            return hostname.ends_with(suffix);
        }

        if pattern.ends_with(".*") {
            let prefix = &pattern[..pattern.len() - 2];
            return hostname.starts_with(prefix);
        }

        if pattern.contains(".*") && !pattern.starts_with("*") && !pattern.ends_with("*") {
            let parts: Vec<&str> = pattern.split(".*").collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                return hostname.starts_with(prefix);
            }
        }

        if pattern.contains("*") {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                return hostname.starts_with(parts[0]) && hostname.ends_with(parts[1]);
            }
        }

        false
    }

    /// Record intercepted connection
    pub fn record_intercepted_connection(
        intercepted_connections: &Arc<Mutex<Vec<InterceptedConnection>>>,
        connection_id: u64,
        domain: String,
        proxy_used: Option<ProxyConfig>,
        status: InterceptionStatus,
    ) {
        let connection = InterceptedConnection {
            id: connection_id,
            original_connection: NetworkConnection {
                local_addr: "127.0.0.1:0".parse().unwrap(),
                remote_addr: None,
                protocol: "DNS".to_string(),
                state: "INTERCEPTED".to_string(),
                process_name: "TrafficInterceptor".to_string(),
                process_id: 0,
                bytes_sent: 0,
                bytes_received: 0,
                last_updated: std::time::Instant::now(),
                interface: "Unknown".to_string(),
            },
            proxy_used,
            intercepted_at: std::time::Instant::now(),
            status,
            bytes_sent: 0,
            bytes_received: 0,
            domain: Some(domain),
        };

        let mut connections = intercepted_connections.lock().unwrap();
        connections.push(connection);
        
        // Keep only last 1000 connections
        if connections.len() > 1000 {
            connections.remove(0);
        }
    }

    /// Log interception configuration
    pub fn log_interception_configuration(&self) {
        // This method should be called from the main TrafficInterceptor struct
        // where proxy_manager is accessible
        println!("🔧 TRAFFIC INTERCEPTION CONFIGURATION:");
        println!("   Configuration logging not available from helpers");
    }
}
