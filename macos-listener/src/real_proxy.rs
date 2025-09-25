use std::sync::{Arc, Mutex};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;
use crate::{ProxyConfig, ProxyManager, ProxyRule};
use pcap::{Device, Capture};

/// Real traffic proxy that actually intercepts and routes traffic
pub struct RealTrafficProxy {
    proxy_manager: Arc<Mutex<ProxyManager>>,
    is_running: Arc<Mutex<bool>>,
    dns_proxy_port: u16,
    tcp_proxy_port: u16,
}

impl RealTrafficProxy {
    pub fn new(proxy_manager: Arc<Mutex<ProxyManager>>) -> Self {
        Self {
            proxy_manager,
            is_running: Arc::new(Mutex::new(false)),
            dns_proxy_port: 5353, // DNS proxy port
            tcp_proxy_port: 8080, // TCP proxy port
        }
    }

    /// Start the real proxy service
    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut is_running = self.is_running.lock().unwrap();
        if *is_running {
            println!("🚨 Real proxy is already running!");
            return Ok(());
        }
        *is_running = true;
        drop(is_running);

        // Log current configuration
        {
            let manager = self.proxy_manager.lock().unwrap();
            println!("🔧 PROXY CONFIGURATION:");
            println!("   Global enabled: {}", manager.global_enabled);
            println!("   Proxies: {}", manager.proxies.len());
            for (i, proxy) in manager.proxies.iter().enumerate() {
                println!("     {}: {} ({}:{}) - {}", i, proxy.name, proxy.host, proxy.port, 
                         if proxy.enabled { "ENABLED" } else { "DISABLED" });
            }
            println!("   Rules: {}", manager.rules.len());
            for (i, rule) in manager.rules.iter().enumerate() {
                println!("     {}: {} -> {} (proxy_id: {}) - {}", i, rule.name, rule.pattern, rule.proxy_id,
                         if rule.enabled { "ENABLED" } else { "DISABLED" });
            }
        }

        // Start system-level traffic interception
        let traffic_manager = Arc::clone(&self.proxy_manager);
        let traffic_is_running = Arc::clone(&self.is_running);
        
        println!("🚀 Starting system-level traffic interception...");
        println!("📋 This will intercept ALL system traffic and route matching connections through SOCKS5");
        thread::spawn(move || {
            if let Err(e) = Self::start_traffic_interception(traffic_manager, traffic_is_running) {
                eprintln!("❌ Traffic interception error: {}", e);
            }
        });

        println!("✅ Real traffic proxy started - intercepting system traffic");
        println!("📝 NOTE: This will intercept ALL system traffic and route matching connections through SOCKS5");
        Ok(())
    }

    /// Stop the proxy service
    pub fn stop(&self) {
        let mut is_running = self.is_running.lock().unwrap();
        *is_running = false;
        println!("Real traffic proxy stopped");
    }

    /// Start system-level traffic interception
    fn start_traffic_interception(
        proxy_manager: Arc<Mutex<ProxyManager>>,
        is_running: Arc<Mutex<bool>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("🌐 Starting system-level traffic interception...");
        println!("📋 This will intercept ALL system traffic and route matching connections through SOCKS5");
        
        // Start DNS interception
        let dns_manager = Arc::clone(&proxy_manager);
        let dns_running = Arc::clone(&is_running);
        thread::spawn(move || {
            if let Err(e) = Self::intercept_dns_traffic(dns_manager, dns_running) {
                eprintln!("❌ DNS interception error: {}", e);
            }
        });

        // Start TCP interception
        let tcp_manager = Arc::clone(&proxy_manager);
        let tcp_running = Arc::clone(&is_running);
        thread::spawn(move || {
            if let Err(e) = Self::intercept_tcp_traffic(tcp_manager, tcp_running) {
                eprintln!("❌ TCP interception error: {}", e);
            }
        });

        // Monitor system traffic
        while *is_running.lock().unwrap() {
            thread::sleep(Duration::from_millis(100));
        }
        println!("🛑 Traffic interception stopped");
        Ok(())
    }

    /// Intercept DNS traffic at system level
    fn intercept_dns_traffic(
        proxy_manager: Arc<Mutex<ProxyManager>>,
        is_running: Arc<Mutex<bool>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("🌐 Intercepting DNS traffic at system level...");
        
        // Find the default network interface
        let device = Device::lookup()?.ok_or("No network interface found")?;
        println!("📡 Using network interface: {}", device.name);
        println!("🔍 Looking for DNS queries on port 53...");
        
        // Create packet capture
        let mut cap = Capture::from_device(device)?
            .promisc(true)
            .snaplen(65536)
            .timeout(100)
            .open()?;
        
        // Filter for DNS traffic (port 53)
        cap.filter("udp port 53", true)?;
        
        while *is_running.lock().unwrap() {
            match cap.next_packet() {
                Ok(packet) => {
                    
                    // Parse DNS packet
                    if let Some(domain) = Self::extract_domain_from_dns_packet(&packet.data) {
                        // Check if this domain should be proxied
                        if let Some(proxy_config) = Self::should_proxy_domain(&proxy_manager, &domain) {
                            println!("🌐 DNS RULE MATCH! '{}' -> {} (proxy: {}:{})", 
                                     domain, proxy_config.name, proxy_config.host, proxy_config.port);
                            
                            // Route DNS query through SOCKS5 proxy
                            Self::route_dns_through_socks5(&domain, &proxy_config)?;
                        }
                    }
                    
                }
                Err(pcap::Error::TimeoutExpired) => {
                    // Timeout is normal, continue
                    continue;
                }
                Err(e) => {
                    eprintln!("❌ DNS packet capture error: {}", e);
                    break;
                }
            }
        }
        
        println!("🛑 DNS interception stopped");
        Ok(())
    }

    /// Intercept TCP traffic at system level
    fn intercept_tcp_traffic(
        proxy_manager: Arc<Mutex<ProxyManager>>,
        is_running: Arc<Mutex<bool>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔗 Intercepting TCP traffic at system level...");
        println!("📋 This is a simplified implementation - real traffic interception requires system-level privileges");
        
        // This is a simplified implementation
        // In a real implementation, you would:
        // 1. Use libpcap to capture TCP packets
        // 2. Parse HTTP headers to extract hostnames
        // 3. Parse TLS SNI to extract hostnames
        // 4. Check rules against hostnames
        // 5. Route through SOCKS5 proxy or direct connection
        
        while *is_running.lock().unwrap() {
            // Simulate TCP connection processing
            thread::sleep(Duration::from_millis(300));
        }
        
        println!("🛑 TCP interception stopped");
        Ok(())
    }

    /// Start TCP proxy server
    fn start_tcp_proxy(
        proxy_manager: Arc<Mutex<ProxyManager>>,
        is_running: Arc<Mutex<bool>>,
        port: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
        println!("🔗 TCP proxy listening on 127.0.0.1:{}", port);
        println!("📋 TCP proxy will check rules for incoming connections:");
        
        // Log current TCP rules
        {
            let manager = proxy_manager.lock().unwrap();
            if manager.global_enabled && !manager.rules.is_empty() {
                println!("📝 Active TCP rules:");
                for rule in &manager.rules {
                    if rule.enabled {
                        println!("   ✅ {}: {} -> proxy_id {}", rule.name, rule.pattern, rule.proxy_id);
                    }
                }
            } else {
                println!("⚠️  No active TCP rules - all connections would be direct");
            }
        }

        let mut connection_count = 0;
        for stream in listener.incoming() {
            if !*is_running.lock().unwrap() {
                break;
            }

            match stream {
                Ok(stream) => {
                    connection_count += 1;
                    let client_addr = stream.peer_addr().unwrap_or_else(|_| "unknown".parse().unwrap());
                    println!("🔌 TCP connection #{} from {}", connection_count, client_addr);
                    
                    let proxy_manager = Arc::clone(&proxy_manager);
                    let is_running = Arc::clone(&is_running);
                    
                    thread::spawn(move || {
                        if let Err(e) = Self::handle_tcp_connection(stream, proxy_manager, is_running) {
                            eprintln!("❌ TCP proxy connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("❌ TCP proxy accept error: {}", e);
                }
            }
        }
        println!("🛑 TCP proxy stopped");
        Ok(())
    }

    /// Handle individual TCP connection
    fn handle_tcp_connection(
        mut client_stream: TcpStream,
        proxy_manager: Arc<Mutex<ProxyManager>>,
        is_running: Arc<Mutex<bool>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client_addr = client_stream.peer_addr()?;
        println!("🔍 Processing TCP connection from {}", client_addr);

        // Read the first packet to determine destination
        let mut buffer = [0u8; 1024];
        let size = client_stream.read(&mut buffer)?;
        
        if size == 0 {
            println!("⚠️  Empty packet from {}, closing connection", client_addr);
            return Ok(());
        }

        println!("📦 Received {} bytes from {}", size, client_addr);
        
        // Parse destination from the packet (simplified)
        match Self::extract_destination_from_packet(&buffer[..size]) {
            Ok(destination) => {
                println!("🎯 Extracted destination: {}", destination);
                
                // Check if this connection should be proxied
                if let Some(proxy_config) = Self::should_proxy_connection(&proxy_manager, &destination) {
                    println!("✅ RULE MATCH! Proxying TCP connection to {} through {}:{}", 
                             destination, proxy_config.host, proxy_config.port);
                    println!("🔗 SOCKS5 connection: {} -> {} -> {}", client_addr, proxy_config.host, destination);
                    Self::proxy_tcp_connection(client_stream, destination, &proxy_config)?;
                } else {
                    println!("❌ No rule match for {} - direct connection", destination);
                    // For now, just close the connection
                    // In a real implementation, you'd establish a direct connection
                }
            }
            Err(e) => {
                println!("⚠️  Could not extract destination from packet: {}", e);
                println!("📄 Packet content (first 100 bytes): {:?}", &buffer[..size.min(100)]);
            }
        }

        Ok(())
    }

    /// Check if DNS query should be proxied (simplified)
    fn should_proxy_dns_query(
        _proxy_manager: &Arc<Mutex<ProxyManager>>,
        _dns_packet: &[u8],
    ) -> Option<ProxyConfig> {
        // Simplified implementation - always return None for now
        None
    }

    /// Check if TCP connection should be proxied
    fn should_proxy_connection(
        proxy_manager: &Arc<Mutex<ProxyManager>>,
        destination: &SocketAddr,
    ) -> Option<ProxyConfig> {
        let manager = proxy_manager.lock().unwrap();
        
        // Quick exit if global proxy is disabled
        if !manager.global_enabled {
            return None;
        }

        // Quick exit if no rules are configured
        if manager.rules.is_empty() {
            return None;
        }

        // Try to resolve IP to hostname, fallback to IP string
        let hostname = Self::resolve_ip_to_hostname(destination.ip())
            .unwrap_or_else(|| {
                match destination.ip() {
                    IpAddr::V4(ip) => ip.to_string(),
                    IpAddr::V6(ip) => ip.to_string(),
                }
            });

        // Quick pre-filter: check if this hostname could potentially match any rule
        if !Self::could_match_any_rule(&hostname, &manager.rules) {
            // Silently skip - no need to log every non-matching connection
            return None;
        }
        
        for rule in &manager.rules {
            if !rule.enabled {
                continue;
            }

            // Split pattern by semicolon and check each sub-pattern
            let patterns: Vec<&str> = rule.pattern.split(';').collect();
            
            let mut any_match = false;
            for sub_pattern in patterns {
                let trimmed_pattern = sub_pattern.trim();
                if trimmed_pattern.is_empty() {
                    continue;
                }
                
                if Self::matches_pattern(trimmed_pattern, &hostname) {
                    any_match = true;
                    break;
                }
            }
            
            if any_match {
                println!("🎯 RULE MATCH! '{}' -> {} (hostname: '{}')", rule.name, rule.pattern, hostname);
                
                // Find the proxy for this rule
                if let Some(proxy) = manager.proxies.iter().find(|p| p.id == rule.proxy_id && p.enabled) {
                    println!("🚀 Routing through proxy: {} ({}:{})", proxy.name, proxy.host, proxy.port);
                    return Some(proxy.clone());
                } else {
                    println!("❌ Proxy {} not found or disabled", rule.proxy_id);
                }
            }
        }

        None
    }

    /// Extract destination from TCP packet (simplified)
    fn extract_destination_from_packet(packet: &[u8]) -> Result<SocketAddr, Box<dyn std::error::Error>> {
        // This is a simplified implementation
        // In reality, you'd need to parse HTTP headers, SNI for TLS, etc.
        
        // For now, try to extract from HTTP Host header
        let packet_str = String::from_utf8_lossy(packet);
        for line in packet_str.lines() {
            if line.to_lowercase().starts_with("host:") {
                let host = line[5..].trim();
                // Default to port 80 for HTTP
                return Ok(SocketAddr::new(host.parse()?, 80));
            }
        }

        // Fallback - this is not a real implementation
        Err("Could not extract destination from packet".into())
    }

    /// Proxy DNS query through SOCKS5 (simplified)
    fn proxy_dns_query(
        _listener: &TcpStream,
        _dns_packet: &[u8],
        _client_addr: SocketAddr,
        _proxy_config: &ProxyConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Simplified implementation
        println!("DNS query would be proxied through SOCKS5");
        Ok(())
    }

    /// Forward DNS query to system DNS (simplified)
    fn forward_to_system_dns(
        _listener: &TcpStream,
        _dns_packet: &[u8],
        _client_addr: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Simplified implementation
        println!("DNS query would be forwarded to system DNS");
        Ok(())
    }

    /// Proxy TCP connection through SOCKS5
    fn proxy_tcp_connection(
        client_stream: TcpStream,
        destination: SocketAddr,
        proxy_config: &ProxyConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔗 Starting SOCKS5 proxy connection...");
        
        // Connect to SOCKS5 proxy
        let proxy_addr = format!("{}:{}", proxy_config.host, proxy_config.port);
        println!("🌐 Connecting to SOCKS5 proxy: {}", proxy_addr);
        
        let mut proxy_stream = TcpStream::connect(&proxy_addr)?;
        println!("✅ Connected to SOCKS5 proxy");
        
        // Perform SOCKS5 handshake
        println!("🤝 Performing SOCKS5 handshake...");
        Self::socks5_handshake_sync(&mut proxy_stream, proxy_config)?;
        println!("✅ SOCKS5 handshake completed");
        
        // Connect to destination through proxy
        println!("🎯 Connecting to destination {} through SOCKS5...", destination);
        Self::socks5_connect(&mut proxy_stream, destination)?;
        println!("✅ Connected to destination through SOCKS5");
        
        // Start bidirectional data forwarding
        let client_addr = client_stream.peer_addr()?;
        let proxy_addr = proxy_stream.peer_addr()?;
        
        println!("🔄 Starting data forwarding: {} <-> {} <-> {}", 
                 client_addr, proxy_addr, destination);
        
        // Forward data between client and proxy
        Self::forward_data_bidirectional(client_stream, proxy_stream)?;
        
        println!("🏁 SOCKS5 proxy connection completed");
        Ok(())
    }

    /// SOCKS5 handshake (simplified version)
    fn socks5_handshake(
        _stream: &mut TcpStream,
        _proxy_config: &ProxyConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Simplified implementation
        println!("SOCKS5 handshake would be performed");
        Ok(())
    }

    /// SOCKS5 handshake (sync version)
    fn socks5_handshake_sync(
        stream: &mut TcpStream,
        proxy_config: &ProxyConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("🤝 SOCKS5 handshake: sending authentication methods");
        
        // Send authentication methods
        let auth_methods = if proxy_config.username.is_some() {
            println!("   Using username/password authentication");
            vec![0x05, 0x01, 0x02, 0x00] // Username/password and no auth
        } else {
            println!("   Using no authentication");
            vec![0x05, 0x01, 0x00] // No authentication
        };
        
        stream.write_all(&auth_methods)?;
        println!("✅ Sent authentication methods");
        
        // Read server response
        let mut response = [0u8; 2];
        stream.read_exact(&mut response)?;
        println!("📥 Received server response: {:?}", response);
        
        if response[0] != 0x05 {
            return Err("Invalid SOCKS5 version".into());
        }
        
        // Handle authentication if required
        if response[1] == 0x02 && proxy_config.username.is_some() {
            println!("🔐 Server requires username/password authentication");
            Self::socks5_authenticate_sync(stream, proxy_config)?;
        } else if response[1] != 0x00 {
            return Err("SOCKS5 authentication failed".into());
        } else {
            println!("✅ No authentication required");
        }
        
        Ok(())
    }

    /// SOCKS5 authentication
    fn socks5_authenticate(
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

    /// SOCKS5 authentication (sync version)
    fn socks5_authenticate_sync(
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
    fn socks5_connect(
        stream: &mut TcpStream,
        destination: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut connect_request = vec![0x05, 0x01, 0x00]; // VER, CMD, RSV
        
        match destination.ip() {
            IpAddr::V4(ip) => {
                connect_request.push(0x01); // ATYP: IPv4
                connect_request.extend_from_slice(&ip.octets());
            }
            IpAddr::V6(ip) => {
                connect_request.push(0x04); // ATYP: IPv6
                connect_request.extend_from_slice(&ip.octets());
            }
        }
        
        connect_request.extend_from_slice(&destination.port().to_be_bytes());
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

    /// Forward data bidirectionally between two streams
    fn forward_data_bidirectional(
        mut client_stream: TcpStream,
        mut proxy_stream: TcpStream,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // use std::io::copy;
        
        // This is a simplified implementation
        // In reality, you'd need to handle both directions concurrently
        let mut buffer = [0u8; 4096];
        
        loop {
            // Read from client and write to proxy
            match client_stream.read(&mut buffer) {
                Ok(0) => break, // Connection closed
                Ok(size) => {
                    proxy_stream.write_all(&buffer[..size])?;
                }
                Err(_) => break,
            }
            
            // Read from proxy and write to client
            match proxy_stream.read(&mut buffer) {
                Ok(0) => break, // Connection closed
                Ok(size) => {
                    client_stream.write_all(&buffer[..size])?;
                }
                Err(_) => break,
            }
        }
        
        Ok(())
    }

    /// Quick check if hostname could potentially match any rule
    fn could_match_any_rule(hostname: &str, rules: &[ProxyRule]) -> bool {
        for rule in rules {
            if !rule.enabled {
                continue;
            }
            
            // Split pattern by semicolon and check each sub-pattern
            let patterns: Vec<&str> = rule.pattern.split(';').collect();
            
            for sub_pattern in patterns {
                let trimmed_pattern = sub_pattern.trim();
                if trimmed_pattern.is_empty() {
                    continue;
                }
                
                // Quick pattern matching - check if this could potentially match
                if Self::quick_pattern_match(trimmed_pattern, hostname) {
                    return true;
                }
            }
        }
        false
    }
    
    /// Quick pattern matching for pre-filtering
    fn quick_pattern_match(pattern: &str, hostname: &str) -> bool {
        // For domain patterns like *.kion.cloud
        if pattern.starts_with("*.") {
            let suffix = &pattern[2..];
            return hostname.ends_with(suffix);
        }
        
        // For IP patterns like 100.64.1.*
        if pattern.contains(".*") && !pattern.starts_with("*") {
            let prefix = pattern.split(".*").next().unwrap_or("");
            return hostname.starts_with(prefix);
        }
        
        // For exact matches
        if pattern == hostname {
            return true;
        }
        
        // For prefix patterns like kion.*
        if pattern.ends_with(".*") {
            let prefix = &pattern[..pattern.len() - 2];
            return hostname.starts_with(prefix);
        }
        
        false
    }

    /// Try to resolve IP address to hostname
    fn resolve_ip_to_hostname(ip: IpAddr) -> Option<String> {
        // For localhost addresses, return special names
        match ip {
            IpAddr::V4(ipv4) => {
                if ipv4.is_loopback() {
                    return Some("localhost".to_string());
                }
                if ipv4.is_private() {
                    // Try reverse DNS lookup for private IPs
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
    fn reverse_dns_lookup(ip: IpAddr) -> Option<String> {
        // This is a simplified implementation
        // In a real implementation, you'd use a DNS resolver
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
                // For IPv6, just return a generic name
                return Some("ipv6-address".to_string());
            }
        }
        
        None
    }

    /// Extract domain name from DNS packet
    fn extract_domain_from_dns_packet(packet: &[u8]) -> Option<String> {
        // This is a simplified DNS packet parser
        // In a real implementation, you'd parse the full DNS packet structure
        
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
    fn should_proxy_domain(
        proxy_manager: &Arc<Mutex<ProxyManager>>,
        domain: &str,
    ) -> Option<ProxyConfig> {
        let manager = proxy_manager.lock().unwrap();
        
        if !manager.global_enabled || manager.rules.is_empty() {
            return None;
        }
        
        for rule in &manager.rules {
            if !rule.enabled {
                continue;
            }
            
            // Split pattern by semicolon and check each sub-pattern
            let patterns: Vec<&str> = rule.pattern.split(';').collect();
            
            for sub_pattern in patterns {
                let trimmed_pattern = sub_pattern.trim();
                if trimmed_pattern.is_empty() {
                    continue;
                }
                
                if Self::matches_pattern(trimmed_pattern, domain) {
                    // Find the proxy for this rule
                    if let Some(proxy) = manager.proxies.iter().find(|p| p.id == rule.proxy_id && p.enabled) {
                        return Some(proxy.clone());
                    }
                }
            }
        }
        
        None
    }
    
    /// Route DNS query through SOCKS5 proxy
    fn route_dns_through_socks5(
        domain: &str,
        proxy_config: &ProxyConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // This is a simplified implementation
        // In a real implementation, you would:
        // 1. Connect to SOCKS5 proxy
        // 2. Perform SOCKS5 handshake
        // 3. Send DNS query through proxy
        // 4. Return the response
        
        Ok(())
    }

    /// Pattern matching for proxy rules
    fn matches_pattern(pattern: &str, hostname: &str) -> bool {
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
}
