use std::net::{IpAddr, SocketAddr};
use std::process::Command;
use std::collections::HashMap;
use std::time::Instant;

// Import the main NetworkConnection type
use crate::NetworkConnection;

pub struct LowLevelNetworkMonitor {
    process_cache: HashMap<u32, String>,
    last_cache_update: Instant,
}

impl LowLevelNetworkMonitor {
    pub fn new() -> Self {
        Self {
            process_cache: HashMap::new(),
            last_cache_update: Instant::now(),
        }
    }

    /// Get network connections using low-level system APIs
    pub fn get_connections(&mut self) -> Result<Vec<NetworkConnection>, Box<dyn std::error::Error>> {
        let mut connections = Vec::new();

        // Method 1: Use netstat with optimized flags (most reliable on macOS)
        if let Ok(netstat_conns) = self.get_connections_netstat_optimized() {
            connections.extend(netstat_conns);
        }

        // Method 2: Use sysctl for direct kernel data access (if netstat fails)
        if connections.is_empty() {
            if let Ok(sysctl_conns) = self.get_connections_sysctl() {
                connections.extend(sysctl_conns);
            }
        }

        // Method 3: Use /proc/net/* files (if available on macOS)
        if connections.is_empty() {
            if let Ok(proc_conns) = self.get_connections_procfs() {
                connections.extend(proc_conns);
            }
        }

        // Update process cache periodically
        if self.last_cache_update.elapsed() > std::time::Duration::from_secs(5) {
            self.update_process_cache()?;
        }

        Ok(connections)
    }

    /// Get connections using sysctl - most efficient method
    fn get_connections_sysctl(&self) -> Result<Vec<NetworkConnection>, Box<dyn std::error::Error>> {
        let mut connections = Vec::new();

        // Get TCP connections via sysctl
        let tcp_output = Command::new("sysctl")
            .args(&["-n", "net.inet.tcp.pcblist"])
            .output()?;

        if tcp_output.status.success() {
            // Parse binary sysctl output
            // Note: This requires parsing the raw binary data structure
            // For now, we'll use a hybrid approach
            connections.extend(self.parse_sysctl_tcp_output(&tcp_output.stdout)?);
        }

        // Get UDP connections via sysctl
        let udp_output = Command::new("sysctl")
            .args(&["-n", "net.inet.udp.pcblist"])
            .output()?;

        if udp_output.status.success() {
            connections.extend(self.parse_sysctl_udp_output(&udp_output.stdout)?);
        }

        Ok(connections)
    }

    /// Parse sysctl TCP output (simplified - real implementation would parse binary data)
    fn parse_sysctl_tcp_output(&self, _data: &[u8]) -> Result<Vec<NetworkConnection>, Box<dyn std::error::Error>> {
        // TODO: Implement proper binary parsing of sysctl output
        // This is a placeholder that falls back to other methods
        Ok(Vec::new())
    }

    /// Parse sysctl UDP output (simplified - real implementation would parse binary data)
    fn parse_sysctl_udp_output(&self, _data: &[u8]) -> Result<Vec<NetworkConnection>, Box<dyn std::error::Error>> {
        // TODO: Implement proper binary parsing of sysctl output
        // This is a placeholder that falls back to other methods
        Ok(Vec::new())
    }

    /// Get connections using /proc/net/* files (Linux-style, may work on some macOS versions)
    fn get_connections_procfs(&self) -> Result<Vec<NetworkConnection>, Box<dyn std::error::Error>> {
        let mut connections = Vec::new();

        // Try to read /proc/net/tcp
        if let Ok(tcp_data) = std::fs::read_to_string("/proc/net/tcp") {
            connections.extend(self.parse_procfs_tcp(&tcp_data)?);
        }

        // Try to read /proc/net/udp
        if let Ok(udp_data) = std::fs::read_to_string("/proc/net/udp") {
            connections.extend(self.parse_procfs_udp(&udp_data)?);
        }

        Ok(connections)
    }

    /// Parse /proc/net/tcp file
    fn parse_procfs_tcp(&self, data: &str) -> Result<Vec<NetworkConnection>, Box<dyn std::error::Error>> {
        let mut connections = Vec::new();

        for line in data.lines().skip(1) { // Skip header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                // Parse local and remote addresses
                if let (Ok(local_addr), Ok(remote_addr)) = (
                    self.parse_hex_address(parts[1]),
                    self.parse_hex_address(parts[2])
                ) {
                    let state = self.parse_tcp_state(parts[3]);
                    let inode = parts[9].parse::<u64>().unwrap_or(0);
                    
                    // Get process info from inode
                    let (process_name, process_id) = self.get_process_info_from_inode(inode);

                    let connection = NetworkConnection {
                        local_addr,
                        remote_addr: if remote_addr.port() == 0 { None } else { Some(remote_addr) },
                        protocol: "TCP".to_string(),
                        state,
                        process_name,
                        process_id,
                        bytes_sent: 0,
                        bytes_received: 0,
                        last_updated: Instant::now(),
                        interface: "Unknown".to_string(),
                    };

                    connections.push(connection);
                }
            }
        }

        Ok(connections)
    }

    /// Parse /proc/net/udp file
    fn parse_procfs_udp(&self, data: &str) -> Result<Vec<NetworkConnection>, Box<dyn std::error::Error>> {
        let mut connections = Vec::new();

        for line in data.lines().skip(1) { // Skip header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                if let Ok(local_addr) = self.parse_hex_address(parts[1]) {
                    let inode = parts[9].parse::<u64>().unwrap_or(0);
                    
                    // Get process info from inode
                    let (process_name, process_id) = self.get_process_info_from_inode(inode);

                    let connection = NetworkConnection {
                        local_addr,
                        remote_addr: None,
                        protocol: "UDP".to_string(),
                        state: "UDP".to_string(),
                        process_name,
                        process_id,
                        bytes_sent: 0,
                        bytes_received: 0,
                        last_updated: Instant::now(),
                        interface: "Unknown".to_string(),
                    };

                    connections.push(connection);
                }
            }
        }

        Ok(connections)
    }

    /// Parse hexadecimal address from /proc/net/* files
    fn parse_hex_address(&self, hex_addr: &str) -> Result<SocketAddr, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = hex_addr.split(':').collect();
        if parts.len() != 2 {
            return Err("Invalid hex address format".into());
        }

        let ip_hex = parts[0];
        let port_hex = parts[1];

        // Parse IP address (4 bytes in hex)
        if ip_hex.len() != 8 {
            return Err("Invalid IP hex length".into());
        }

        let ip_bytes = u32::from_str_radix(ip_hex, 16)?;
        let ip = std::net::Ipv4Addr::new(
            (ip_bytes >> 24) as u8,
            (ip_bytes >> 16) as u8,
            (ip_bytes >> 8) as u8,
            ip_bytes as u8,
        );

        // Parse port (2 bytes in hex)
        let port = u16::from_str_radix(port_hex, 16)?;

        Ok(SocketAddr::new(IpAddr::V4(ip), port))
    }

    /// Parse TCP state from /proc/net/tcp
    fn parse_tcp_state(&self, state_hex: &str) -> String {
        match u8::from_str_radix(state_hex, 16).unwrap_or(0) {
            1 => "ESTABLISHED".to_string(),
            2 => "SYN_SENT".to_string(),
            3 => "SYN_RECV".to_string(),
            4 => "FIN_WAIT1".to_string(),
            5 => "FIN_WAIT2".to_string(),
            6 => "TIME_WAIT".to_string(),
            7 => "CLOSE".to_string(),
            8 => "CLOSE_WAIT".to_string(),
            9 => "LAST_ACK".to_string(),
            10 => "LISTEN".to_string(),
            11 => "CLOSING".to_string(),
            _ => "UNKNOWN".to_string(),
        }
    }

    /// Get process information from inode
    fn get_process_info_from_inode(&self, inode: u64) -> (String, u32) {
        // Try to find process by inode
        // This is a simplified implementation
        if let Ok(processes) = self.get_processes_by_inode(inode) {
            if let Some((name, pid)) = processes.first() {
                return (name.clone(), *pid);
            }
        }
        
        ("Unknown".to_string(), 0)
    }

    /// Get processes by inode (simplified)
    fn get_processes_by_inode(&self, _inode: u64) -> Result<Vec<(String, u32)>, Box<dyn std::error::Error>> {
        // This would require parsing /proc/*/fd/* files
        // For now, return empty
        Ok(Vec::new())
    }

    /// Optimized netstat approach with minimal overhead
    fn get_connections_netstat_optimized(&self) -> Result<Vec<NetworkConnection>, Box<dyn std::error::Error>> {
        let mut connections = Vec::new();

        // Use netstat with optimized flags
        let output = Command::new("netstat")
            .args(&["-an"])
            .output()?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            connections.extend(self.parse_netstat_output(&output_str)?);
        }

        Ok(connections)
    }

    /// Parse netstat output
    fn parse_netstat_output(&self, output: &str) -> Result<Vec<NetworkConnection>, Box<dyn std::error::Error>> {
        let mut connections = Vec::new();

        for line in output.lines() {
            if line.contains("tcp") || line.contains("udp") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let protocol = if line.contains("tcp") { "TCP" } else { "UDP" };
                    
                    if let Ok(local_addr) = self.parse_socket_addr(parts[3]) {
                        let remote_addr = if parts.len() > 4 && !parts[4].is_empty() {
                            self.parse_socket_addr(parts[4]).ok()
                        } else {
                            None
                        };

                        let state = if parts.len() > 5 { parts[5].to_string() } else { protocol.to_string() };

                        let connection = NetworkConnection {
                            local_addr,
                            remote_addr,
                            protocol: protocol.to_string(),
                            state,
                            process_name: "Unknown".to_string(),
                            process_id: 0,
                            bytes_sent: 0,
                            bytes_received: 0,
                            last_updated: Instant::now(),
                            interface: "Unknown".to_string(),
                        };

                        connections.push(connection);
                    }
                }
            }
        }

        Ok(connections)
    }

    /// Parse socket address from string
    fn parse_socket_addr(&self, addr_str: &str) -> Result<SocketAddr, Box<dyn std::error::Error>> {
        if addr_str.starts_with('*') {
            let port_str = &addr_str[2..];
            let port = port_str.parse::<u16>()?;
            Ok(SocketAddr::new(IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)), port))
        } else if addr_str == "*.*" {
            // Skip wildcard addresses
            Err("Wildcard address".into())
        } else {
            // Handle addresses like "192.168.0.136.51696" or "::1.4200"
            // We need to find the last dot and treat everything after it as port
            if let Some(last_dot) = addr_str.rfind('.') {
                let ip_part = &addr_str[..last_dot];
                let port_part = &addr_str[last_dot + 1..];
                
                if let Ok(port) = port_part.parse::<u16>() {
                    // Try to parse as IPv4 first
                    if let Ok(ipv4) = ip_part.parse::<std::net::Ipv4Addr>() {
                        return Ok(SocketAddr::new(IpAddr::V4(ipv4), port));
                    }
                    // Try to parse as IPv6
                    if let Ok(ipv6) = ip_part.parse::<std::net::Ipv6Addr>() {
                        return Ok(SocketAddr::new(IpAddr::V6(ipv6), port));
                    }
                }
            }
            
            // Fallback: try to parse as standard SocketAddr format
            addr_str.parse::<SocketAddr>().map_err(|e| format!("Failed to parse '{}': {}", addr_str, e).into())
        }
    }

    /// Update process cache
    fn update_process_cache(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Get process list using ps
        let output = Command::new("ps")
            .args(&["-ax", "-o", "pid,comm"])
            .output()?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            self.process_cache.clear();

            for line in output_str.lines().skip(1) { // Skip header
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(pid) = parts[0].parse::<u32>() {
                        let name = parts[1..].join(" ");
                        self.process_cache.insert(pid, name);
                    }
                }
            }
        }

        self.last_cache_update = Instant::now();
        Ok(())
    }

    /// Get process name by PID
    pub fn get_process_name(&self, pid: u32) -> String {
        self.process_cache.get(&pid).cloned().unwrap_or_else(|| "Unknown".to_string())
    }
}

impl Default for LowLevelNetworkMonitor {
    fn default() -> Self {
        Self::new()
    }
}
