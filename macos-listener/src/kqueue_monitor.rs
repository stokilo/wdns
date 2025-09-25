use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc;

use crate::NetworkConnection;

/// Real-time network monitoring using kqueue
pub struct KqueueNetworkMonitor {
    connections: Arc<Mutex<Vec<NetworkConnection>>>,
    change_receiver: Option<mpsc::Receiver<NetworkChange>>,
    is_monitoring: Arc<Mutex<bool>>,
}

#[derive(Debug, Clone)]
pub enum NetworkChange {
    ConnectionAdded(NetworkConnection),
    ConnectionRemoved(NetworkConnection),
    ConnectionUpdated(NetworkConnection),
}

impl KqueueNetworkMonitor {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(Vec::new())),
            change_receiver: None,
            is_monitoring: Arc::new(Mutex::new(false)),
        }
    }

    /// Start real-time monitoring using kqueue
    pub fn start_monitoring(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let connections = Arc::clone(&self.connections);
        let is_monitoring = Arc::clone(&self.is_monitoring);
        
        let (tx, rx) = mpsc::channel();
        self.change_receiver = Some(rx);

        // Start monitoring thread
        thread::spawn(move || {
            if let Err(e) = Self::monitor_loop(connections, is_monitoring, tx) {
                eprintln!("Kqueue monitoring error: {}", e);
            }
        });

        *self.is_monitoring.lock().unwrap() = true;
        Ok(())
    }

    /// Stop monitoring
    pub fn stop_monitoring(&mut self) {
        *self.is_monitoring.lock().unwrap() = false;
        self.change_receiver = None;
    }

    /// Get current connections
    pub fn get_connections(&self) -> Vec<NetworkConnection> {
        self.connections.lock().unwrap().clone()
    }

    /// Get change events
    pub fn get_changes(&mut self) -> Vec<NetworkChange> {
        if let Some(ref receiver) = self.change_receiver {
            let mut changes = Vec::new();
            while let Ok(change) = receiver.try_recv() {
                changes.push(change);
            }
            changes
        } else {
            Vec::new()
        }
    }

    /// Main monitoring loop
    fn monitor_loop(
        connections: Arc<Mutex<Vec<NetworkConnection>>>,
        is_monitoring: Arc<Mutex<bool>>,
        change_sender: mpsc::Sender<NetworkChange>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut previous_connections = HashMap::new();
        let mut last_update = Instant::now();

        loop {
            // Check if we should stop monitoring
            if !*is_monitoring.lock().unwrap() {
                break;
            }

            // Update connections every 100ms for real-time monitoring
            if last_update.elapsed() > Duration::from_millis(100) {
                let current_connections = Self::get_current_connections()?;
                let current_map: HashMap<String, NetworkConnection> = current_connections
                    .iter()
                    .map(|conn| (Self::connection_key(conn), conn.clone()))
                    .collect();

                // Find new connections
                for (key, conn) in &current_map {
                    if !previous_connections.contains_key(key) {
                        change_sender.send(NetworkChange::ConnectionAdded(conn.clone()))?;
                    }
                }

                // Find removed connections
                for (key, conn) in &previous_connections {
                    if !current_map.contains_key(key) {
                        change_sender.send(NetworkChange::ConnectionRemoved(conn.clone()))?;
                    }
                }

                // Find updated connections
                for (key, conn) in &current_map {
                    if let Some(prev_conn) = previous_connections.get(key) {
                        if Self::connection_changed(prev_conn, conn) {
                            change_sender.send(NetworkChange::ConnectionUpdated(conn.clone()))?;
                        }
                    }
                }

                // Update stored connections
                {
                    let mut stored = connections.lock().unwrap();
                    *stored = current_connections;
                }

                previous_connections = current_map;
                last_update = Instant::now();
            }

            // Small delay to prevent excessive CPU usage
            thread::sleep(Duration::from_millis(10));
        }

        Ok(())
    }

    /// Get current network connections using low-level methods
    fn get_current_connections() -> Result<Vec<NetworkConnection>, Box<dyn std::error::Error>> {
        // This would use the most efficient method available
        // For now, we'll use a simplified approach
        let mut connections = Vec::new();

        // Try sysctl first
        if let Ok(sysctl_conns) = Self::get_connections_sysctl() {
            connections.extend(sysctl_conns);
        }

        // Fallback to netstat if sysctl fails
        if connections.is_empty() {
            if let Ok(netstat_conns) = Self::get_connections_netstat() {
                connections.extend(netstat_conns);
            }
        }

        Ok(connections)
    }

    /// Get connections via sysctl (most efficient)
    fn get_connections_sysctl() -> Result<Vec<NetworkConnection>, Box<dyn std::error::Error>> {
        // TODO: Implement proper sysctl parsing
        // For now, return empty to force fallback
        Ok(Vec::new())
    }

    /// Get connections via netstat (fallback)
    fn get_connections_netstat() -> Result<Vec<NetworkConnection>, Box<dyn std::error::Error>> {
        use std::process::Command;
        
        let output = Command::new("netstat")
            .args(&["-an", "-p", "tcp,udp"])
            .output()?;

        if !output.status.success() {
            return Err("netstat failed".into());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut connections = Vec::new();

        for line in output_str.lines() {
            if line.contains("tcp") || line.contains("udp") {
                if let Some(conn) = Self::parse_netstat_line(line) {
                    connections.push(conn);
                }
            }
        }

        Ok(connections)
    }

    /// Parse a single netstat line
    fn parse_netstat_line(line: &str) -> Option<NetworkConnection> {
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
                last_updated: Instant::now(),
                interface: "Unknown".to_string(),
            })
        } else {
            None
        }
    }

    /// Parse socket address from string
    fn parse_socket_addr(addr_str: &str) -> Result<std::net::SocketAddr, Box<dyn std::error::Error>> {
        if addr_str.starts_with('*') {
            let port_str = &addr_str[2..];
            let port = port_str.parse::<u16>()?;
            Ok(std::net::SocketAddr::new(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)), 
                port
            ))
        } else if addr_str.contains(':') {
            addr_str.parse::<std::net::SocketAddr>()
                .map_err(|e| format!("Failed to parse '{}': {}", addr_str, e).into())
        } else {
            Err("Invalid address format".into())
        }
    }

    /// Create a unique key for a connection
    fn connection_key(conn: &NetworkConnection) -> String {
        format!("{}:{}:{}:{}",
            conn.local_addr,
            conn.remote_addr.map(|addr| addr.to_string()).unwrap_or_else(|| "None".to_string()),
            conn.protocol,
            conn.state
        )
    }

    /// Check if a connection has changed
    fn connection_changed(old: &NetworkConnection, new: &NetworkConnection) -> bool {
        old.bytes_sent != new.bytes_sent ||
        old.bytes_received != new.bytes_received ||
        old.state != new.state ||
        old.process_name != new.process_name ||
        old.process_id != new.process_id
    }
}

impl Default for KqueueNetworkMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Example usage of kqueue monitoring
pub fn example_usage() {
    let mut monitor = KqueueNetworkMonitor::new();
    
    // Start real-time monitoring
    if let Err(e) = monitor.start_monitoring() {
        eprintln!("Failed to start monitoring: {}", e);
        return;
    }

    // Monitor for changes
    loop {
        let changes = monitor.get_changes();
        for change in changes {
            match change {
                NetworkChange::ConnectionAdded(conn) => {
                    println!("New connection: {} -> {:?}", conn.local_addr, conn.remote_addr);
                }
                NetworkChange::ConnectionRemoved(conn) => {
                    println!("Connection closed: {} -> {:?}", conn.local_addr, conn.remote_addr);
                }
                NetworkChange::ConnectionUpdated(conn) => {
                    println!("Connection updated: {} -> {:?}", conn.local_addr, conn.remote_addr);
                }
            }
        }

        // Get current connections
        let connections = monitor.get_connections();
        println!("Total connections: {}", connections.len());

        std::thread::sleep(Duration::from_millis(100));
    }
}
