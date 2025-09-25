use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::process::Command;
use std::str::FromStr;
use std::time::{Duration, Instant};
use tokio::time;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub local_addr: SocketAddr,
    pub remote_addr: Option<SocketAddr>,
    pub protocol: String,
    pub state: String,
    pub process_name: String,
    pub process_id: u32,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub last_updated: Instant,
    pub interface: String,
}

#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub total_connections: usize,
    pub tcp_connections: usize,
    pub udp_connections: usize,
    pub listening_ports: usize,
    pub established_connections: usize,
    pub last_updated: Instant,
}

#[derive(Clone)]
pub struct NetworkMonitor {
    connections: Vec<NetworkConnection>,
    stats: NetworkStats,
    last_update: Instant,
    update_interval: Duration,
}

impl NetworkMonitor {
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
            stats: NetworkStats {
                total_connections: 0,
                tcp_connections: 0,
                udp_connections: 0,
                listening_ports: 0,
                established_connections: 0,
                last_updated: Instant::now(),
            },
            last_update: Instant::now(),
            update_interval: Duration::from_secs(1),
        }
    }

    pub async fn start_monitoring(&mut self) -> Result<()> {
        let mut interval = time::interval(self.update_interval);
        
        loop {
            interval.tick().await;
            self.update_connections().await?;
        }
    }

    pub async fn update_connections(&mut self) -> Result<()> {
        self.connections = self.get_network_connections().await?;
        self.update_stats();
        self.last_update = Instant::now();
        Ok(())
    }

    pub fn get_connections(&self) -> &[NetworkConnection] {
        &self.connections
    }

    pub fn get_stats(&self) -> &NetworkStats {
        &self.stats
    }

    async fn get_network_connections(&self) -> Result<Vec<NetworkConnection>> {
        let mut connections = Vec::new();

        // Get TCP connections using netstat
        let tcp_connections = self.get_tcp_connections().await?;
        connections.extend(tcp_connections);

        // Get UDP connections using netstat
        let udp_connections = self.get_udp_connections().await?;
        connections.extend(udp_connections);

        // Get process information for each connection
        let process_map = self.get_process_map().await?;
        
        for conn in &mut connections {
            if let Some(process_info) = process_map.get(&conn.process_id) {
                conn.process_name = process_info.name.clone();
            }
        }

        Ok(connections)
    }

    async fn get_tcp_connections(&self) -> Result<Vec<NetworkConnection>> {
        let output = Command::new("netstat")
            .args(&["-an", "-p", "tcp"])
            .output()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut connections = Vec::new();

        for line in output_str.lines() {
            if line.contains("tcp") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let local_addr = self.parse_socket_addr(parts[3])?;
                    let state = if parts.len() > 4 { parts[4].to_string() } else { "UNKNOWN".to_string() };
                    let remote_addr = if parts.len() > 5 { 
                        self.parse_socket_addr(parts[5]).ok() 
                    } else { 
                        None 
                    };

                    let connection = NetworkConnection {
                        local_addr,
                        remote_addr,
                        protocol: "TCP".to_string(),
                        state,
                        process_name: "Unknown".to_string(),
                        process_id: 0, // Will be filled later
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

    async fn get_udp_connections(&self) -> Result<Vec<NetworkConnection>> {
        let output = Command::new("netstat")
            .args(&["-an", "-p", "udp"])
            .output()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut connections = Vec::new();

        for line in output_str.lines() {
            if line.contains("udp") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let local_addr = self.parse_socket_addr(parts[3])?;
                    let remote_addr = if parts.len() > 4 { 
                        self.parse_socket_addr(parts[4]).ok() 
                    } else { 
                        None 
                    };

                    let connection = NetworkConnection {
                        local_addr,
                        remote_addr,
                        protocol: "UDP".to_string(),
                        state: "UDP".to_string(),
                        process_name: "Unknown".to_string(),
                        process_id: 0, // Will be filled later
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

    async fn get_process_map(&self) -> Result<HashMap<u32, ProcessInfo>> {
        let output = Command::new("lsof")
            .args(&["-i", "-P", "-n"])
            .output()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut process_map = HashMap::new();

        for line in output_str.lines().skip(1) { // Skip header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(pid) = parts[1].parse::<u32>() {
                    let name = parts[0].to_string();
                    process_map.insert(pid, ProcessInfo { name });
                }
            }
        }

        Ok(process_map)
    }

    fn parse_socket_addr(&self, addr_str: &str) -> Result<SocketAddr> {
        // Handle addresses like "127.0.0.1.8080" or "*.8080"
        if addr_str.starts_with('*') {
            let port_str = &addr_str[2..];
            let port = port_str.parse::<u16>()?;
            return Ok(SocketAddr::new(IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), port));
        }

        if let Some(dot_pos) = addr_str.rfind('.') {
            let ip_part = &addr_str[..dot_pos];
            let port_part = &addr_str[dot_pos + 1..];
            
            if let Ok(port) = port_part.parse::<u16>() {
                if let Ok(ip) = ip_part.parse::<IpAddr>() {
                    return Ok(SocketAddr::new(ip, port));
                }
            }
        }

        Err(anyhow::anyhow!("Invalid socket address: {}", addr_str))
    }

    fn update_stats(&mut self) {
        self.stats.total_connections = self.connections.len();
        self.stats.tcp_connections = self.connections.iter().filter(|c| c.protocol == "TCP").count();
        self.stats.udp_connections = self.connections.iter().filter(|c| c.protocol == "UDP").count();
        self.stats.listening_ports = self.connections.iter().filter(|c| c.state == "LISTEN").count();
        self.stats.established_connections = self.connections.iter().filter(|c| c.state == "ESTABLISHED").count();
        self.stats.last_updated = Instant::now();
    }
}

#[derive(Debug, Clone)]
struct ProcessInfo {
    name: String,
}

impl Default for NetworkMonitor {
    fn default() -> Self {
        Self::new()
    }
}
