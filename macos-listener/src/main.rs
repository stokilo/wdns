use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::process::Command;
use std::net::{IpAddr, SocketAddr};
use std::collections::VecDeque;

mod network_monitor;
mod socks5_client;
mod traffic_interceptor;
use network_monitor::LowLevelNetworkMonitor;
use traffic_interceptor::{TrafficInterceptor, SystemTrafficInterceptor};

#[derive(Debug, Clone)]
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
pub struct ConnectionLogEntry {
    pub connection: NetworkConnection,
    pub timestamp: SystemTime,
    pub event_type: ConnectionEvent,
    pub id: u64,
}

#[derive(Debug, Clone)]
pub enum ConnectionEvent {
    New,
    Updated,
    Closed,
    Established,
}

#[derive(Debug, Clone)]
pub struct ProxyRule {
    pub id: u32,
    pub name: String,
    pub pattern: String,  // e.g., "*.kion.cloud", "100.64.1.*", "*.kiongroup.net"
    pub enabled: bool,
    pub proxy_id: u32,
}

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub id: u32,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub proxy_type: ProxyType,
    pub username: Option<String>,
    pub password: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProxyType {
    Socks5,
    Http,
    Socks4,
}

impl Default for ProxyType {
    fn default() -> Self {
        ProxyType::Socks5
    }
}

impl std::fmt::Display for ProxyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyType::Socks5 => write!(f, "SOCKS5"),
            ProxyType::Http => write!(f, "HTTP"),
            ProxyType::Socks4 => write!(f, "SOCKS4"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProxyManager {
    pub proxies: Vec<ProxyConfig>,
    pub rules: Vec<ProxyRule>,
    pub next_proxy_id: u32,
    pub next_rule_id: u32,
    pub global_enabled: bool,
}

impl Default for ProxyManager {
    fn default() -> Self {
        Self {
            proxies: Vec::new(),
            rules: Vec::new(),
            next_proxy_id: 1,
            next_rule_id: 1,
            global_enabled: false,
        }
    }
}

impl ProxyManager {
    pub fn add_proxy(&mut self, name: String, host: String, port: u16, proxy_type: ProxyType) -> u32 {
        let id = self.next_proxy_id;
        self.next_proxy_id += 1;
        
        let proxy = ProxyConfig {
            id,
            name,
            host,
            port,
            proxy_type,
            username: None,
            password: None,
            enabled: true,
        };
        
        self.proxies.push(proxy);
        id
    }
    
    pub fn add_rule(&mut self, name: String, pattern: String, proxy_id: u32) -> u32 {
        let id = self.next_rule_id;
        self.next_rule_id += 1;
        
        let rule = ProxyRule {
            id,
            name,
            pattern,
            enabled: true,
            proxy_id,
        };
        
        self.rules.push(rule);
        id
    }
    
    pub fn remove_proxy(&mut self, id: u32) -> bool {
        if let Some(pos) = self.proxies.iter().position(|p| p.id == id) {
            self.proxies.remove(pos);
            // Remove rules that use this proxy
            self.rules.retain(|r| r.proxy_id != id);
            true
        } else {
            false
        }
    }
    
    pub fn remove_rule(&mut self, id: u32) -> bool {
        if let Some(pos) = self.rules.iter().position(|r| r.id == id) {
            self.rules.remove(pos);
            true
        } else {
            false
        }
    }
    
    pub fn get_proxy_for_connection(&self, remote_addr: &SocketAddr) -> Option<&ProxyConfig> {
        if !self.global_enabled {
            return None;
        }
        
        let hostname = match remote_addr.ip() {
            IpAddr::V4(ip) => ip.to_string(),
            IpAddr::V6(ip) => ip.to_string(),
        };
        
        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }
            
            if self.matches_pattern(&rule.pattern, &hostname) {
                return self.proxies.iter().find(|p| p.id == rule.proxy_id && p.enabled);
            }
        }
        
        None
    }
    
    fn matches_pattern(&self, pattern: &str, hostname: &str) -> bool {
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
        
        // Simple wildcard matching
        if pattern.contains("*") {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                return hostname.starts_with(parts[0]) && hostname.ends_with(parts[1]);
            }
        }
        
        false
    }
}

pub struct MacosListenerApp {
    connections: Arc<Mutex<Vec<NetworkConnection>>>,
    connection_log: Arc<Mutex<VecDeque<ConnectionLogEntry>>>,
    last_update: Instant,
    update_interval: Duration,
    selected_connection: Option<usize>,
    filter_text: String,
    show_local_only: bool,
    show_remote_only: bool,
    sort_by: SortBy,
    sort_ascending: bool,
    stats: NetworkStats,
    log_filter_text: String,
    show_log_dialog: bool,
    selected_log_entry: Option<usize>,
    log_entry_id_counter: u64,
    previous_connections: Vec<NetworkConnection>,
    network_monitor: LowLevelNetworkMonitor,
    use_low_level: bool,
    proxy_manager: ProxyManager,
    show_proxy_config: bool,
    show_proxy_rules: bool,
    new_proxy_name: String,
    new_proxy_host: String,
    new_proxy_port: String,
    new_proxy_type: ProxyType,
    new_rule_name: String,
    new_rule_pattern: String,
    selected_proxy_for_rule: Option<u32>,
    traffic_interceptor: Option<TrafficInterceptor>,
    system_interceptor: SystemTrafficInterceptor,
    show_intercepted_traffic: bool,
}

impl Default for MacosListenerApp {
    fn default() -> Self {
        Self {
            connections: Arc::new(Mutex::new(Vec::new())),
            connection_log: Arc::new(Mutex::new(VecDeque::new())),
            last_update: Instant::now(),
            update_interval: Duration::from_secs(2),
            selected_connection: None,
            filter_text: String::new(),
            show_local_only: false,
            show_remote_only: false,
            sort_by: SortBy::LocalAddr,
            sort_ascending: true,
            stats: NetworkStats::default(),
            log_filter_text: String::new(),
            show_log_dialog: false,
            selected_log_entry: None,
            log_entry_id_counter: 0,
            previous_connections: Vec::new(),
            network_monitor: LowLevelNetworkMonitor::new(),
            use_low_level: true,
            proxy_manager: ProxyManager::default(),
            show_proxy_config: false,
            show_proxy_rules: false,
            new_proxy_name: String::new(),
            new_proxy_host: String::new(),
            new_proxy_port: String::new(),
            new_proxy_type: ProxyType::Socks5,
            new_rule_name: String::new(),
            new_rule_pattern: String::new(),
            selected_proxy_for_rule: None,
            traffic_interceptor: None,
            system_interceptor: SystemTrafficInterceptor::new(),
            show_intercepted_traffic: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortBy {
    LocalAddr,
    RemoteAddr,
    Process,
    Protocol,
    State,
    BytesSent,
    BytesReceived,
}

impl Default for SortBy {
    fn default() -> Self {
        SortBy::LocalAddr
    }
}

#[derive(Debug, Clone)]
struct NetworkStats {
    pub total_connections: usize,
    pub tcp_connections: usize,
    pub udp_connections: usize,
    pub listening_ports: usize,
    pub established_connections: usize,
    pub last_updated: Instant,
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self {
            total_connections: 0,
            tcp_connections: 0,
            udp_connections: 0,
            listening_ports: 0,
            established_connections: 0,
            last_updated: Instant::now(),
        }
    }
}

impl eframe::App for MacosListenerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update connections periodically
        if self.last_update.elapsed() > self.update_interval {
            self.update_connections();
            self.last_update = Instant::now();
        }

        // Request repaint for smooth updates
        ctx.request_repaint_after(Duration::from_millis(100));

        self.render_ui(ctx);
    }
}

impl MacosListenerApp {
    pub fn new() -> Self {
        let mut app = Self {
            update_interval: Duration::from_secs(2),
            ..Default::default()
        };
        app.update_connections();
        app
    }

    fn update_connections(&mut self) {
        let connections = self.get_network_connections();
        
        // Log connection changes
        self.log_connection_changes(&connections);
        
        if let Ok(mut conns) = self.connections.lock() {
            *conns = connections;
        }
        self.update_stats();
    }

    fn log_connection_changes(&mut self, new_connections: &[NetworkConnection]) {
        let mut log = if let Ok(log) = self.connection_log.lock() {
            log.clone()
        } else {
            return;
        };

        // Find new connections
        for new_conn in new_connections {
            let is_new = !self.previous_connections.iter().any(|prev_conn| {
                prev_conn.local_addr == new_conn.local_addr && 
                prev_conn.remote_addr == new_conn.remote_addr &&
                prev_conn.protocol == new_conn.protocol
            });

            if is_new {
                self.log_entry_id_counter += 1;
                let log_entry = ConnectionLogEntry {
                    connection: new_conn.clone(),
                    timestamp: SystemTime::now(),
                    event_type: ConnectionEvent::New,
                    id: self.log_entry_id_counter,
                };
                log.push_back(log_entry);
            }
        }

        // Find closed connections
        for prev_conn in &self.previous_connections {
            let is_closed = !new_connections.iter().any(|new_conn| {
                new_conn.local_addr == prev_conn.local_addr && 
                new_conn.remote_addr == prev_conn.remote_addr &&
                new_conn.protocol == prev_conn.protocol
            });

            if is_closed {
                self.log_entry_id_counter += 1;
                let log_entry = ConnectionLogEntry {
                    connection: prev_conn.clone(),
                    timestamp: SystemTime::now(),
                    event_type: ConnectionEvent::Closed,
                    id: self.log_entry_id_counter,
                };
                log.push_back(log_entry);
            }
        }

        // Update previous connections
        self.previous_connections = new_connections.to_vec();

        // Keep only last 1000 entries
        while log.len() > 1000 {
            log.pop_front();
        }

        // Update the shared log
        if let Ok(mut shared_log) = self.connection_log.lock() {
            *shared_log = log;
        }
    }

    fn get_network_connections(&mut self) -> Vec<NetworkConnection> {
        if self.use_low_level {
            // Use low-level network monitor
            match self.network_monitor.get_connections() {
                Ok(connections) => connections,
                Err(e) => {
                    eprintln!("Low-level monitor failed: {}, falling back to traditional methods", e);
                    self.use_low_level = false;
                    self.get_network_connections_traditional()
                }
            }
        } else {
            self.get_network_connections_traditional()
        }
    }

    fn get_network_connections_traditional(&self) -> Vec<NetworkConnection> {
        let mut connections = Vec::new();

        // Try low-level sysctl approach first
        if let Ok(sysctl_connections) = self.get_connections_via_sysctl() {
            connections.extend(sysctl_connections);
        } else {
            // Fallback to lsof/netstat if sysctl fails
        if let Ok(tcp_connections) = self.get_tcp_connections() {
            connections.extend(tcp_connections);
        }

        if let Ok(udp_connections) = self.get_udp_connections() {
            connections.extend(udp_connections);
            }
        }

        connections
    }

    fn get_tcp_connections(&self) -> Result<Vec<NetworkConnection>, Box<dyn std::error::Error>> {
        // Use lsof for better process information
        let output = Command::new("lsof")
            .args(&["-i", "tcp", "-P", "-n"])
            .output()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut connections = Vec::new();

        for line in output_str.lines().skip(1) { // Skip header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 9 {
                let process_name = parts[0].to_string();
                let pid = parts[1].parse::<u32>().unwrap_or(0);
                let node = parts[4];
                let name = parts[8];
                
                if node == "IPv4" || node == "IPv6" {
                    if name.contains("->") {
                        // Established connection
                        let addresses: Vec<&str> = name.split("->").collect();
                        if addresses.len() == 2 {
                            let local_str = addresses[0].trim();
                            let remote_str = addresses[1].trim();
                            
                            match (self.parse_socket_addr(local_str), self.parse_socket_addr(remote_str)) {
                                (Ok(local_addr), Ok(remote_addr)) => {
                                    let connection = NetworkConnection {
                                        local_addr,
                                        remote_addr: Some(remote_addr),
                                        protocol: "TCP".to_string(),
                                        state: "ESTABLISHED".to_string(),
                                        process_name,
                                        process_id: pid,
                                        bytes_sent: 0,
                                        bytes_received: 0,
                                        last_updated: Instant::now(),
                                        interface: "Unknown".to_string(),
                                    };
                                    connections.push(connection);
                                    println!("Added connection: {} -> {}", local_str, remote_str);
                                },
                                (Err(e1), _) => {
                                    println!("Failed to parse local '{}': {}", local_str, e1);
                                },
                                (_, Err(e2)) => {
                                    println!("Failed to parse remote '{}': {}", remote_str, e2);
                                }
                            }
                        }
                    } else {
                        // Listening connection
                        match self.parse_socket_addr(name) {
                            Ok(local_addr) => {
                                let connection = NetworkConnection {
                                    local_addr,
                                    remote_addr: None,
                                    protocol: "TCP".to_string(),
                                    state: "LISTEN".to_string(),
                                    process_name,
                                    process_id: pid,
                                    bytes_sent: 0,
                                    bytes_received: 0,
                                    last_updated: Instant::now(),
                                    interface: "Unknown".to_string(),
                                };
                                connections.push(connection);
                                println!("Added listening connection: {}", name);
                            },
                            Err(e) => {
                                println!("Failed to parse listening addr '{}': {}", name, e);
                            }
                        }
                    }
                }
            }
        }

        Ok(connections)
    }

    fn get_udp_connections(&self) -> Result<Vec<NetworkConnection>, Box<dyn std::error::Error>> {
        let output = Command::new("netstat")
            .args(&["-an", "-p", "udp"])
            .output()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut connections = Vec::new();

        for line in output_str.lines() {
            if line.contains("udp") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    if let Ok(local_addr) = self.parse_socket_addr(parts[3]) {
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

    fn get_connections_via_sysctl(&self) -> Result<Vec<NetworkConnection>, Box<dyn std::error::Error>> {
        let mut connections = Vec::new();
        
        // Get TCP connections via sysctl
        if let Ok(tcp_conns) = self.get_tcp_connections_sysctl() {
            connections.extend(tcp_conns);
        }
        
        // Get UDP connections via sysctl  
        if let Ok(udp_conns) = self.get_udp_connections_sysctl() {
            connections.extend(udp_conns);
        }
        
        Ok(connections)
    }

    fn get_tcp_connections_sysctl(&self) -> Result<Vec<NetworkConnection>, Box<dyn std::error::Error>> {
        // Use sysctl to get TCP connection table
        // This is much more efficient than spawning external processes
        let output = Command::new("sysctl")
            .args(&["-n", "net.inet.tcp.pcblist"])
            .output()?;
            
        if !output.status.success() {
            return Err("Failed to get TCP connections via sysctl".into());
        }
        
        // Parse the output - this is a simplified version
        // In a real implementation, you'd parse the binary data structure
        let _output_str = String::from_utf8_lossy(&output.stdout);
        
        // For now, fall back to netstat for parsing
        // TODO: Implement proper binary parsing of sysctl output
        self.get_tcp_connections()
    }

    fn get_udp_connections_sysctl(&self) -> Result<Vec<NetworkConnection>, Box<dyn std::error::Error>> {
        // Use sysctl to get UDP connection table
        let output = Command::new("sysctl")
            .args(&["-n", "net.inet.udp.pcblist"])
            .output()?;
            
        if !output.status.success() {
            return Err("Failed to get UDP connections via sysctl".into());
        }
        
        // Parse the output - this is a simplified version
        // In a real implementation, you'd parse the binary data structure
        let _output_str = String::from_utf8_lossy(&output.stdout);
        
        // For now, fall back to netstat for parsing
        // TODO: Implement proper binary parsing of sysctl output
        self.get_udp_connections()
    }

    fn parse_socket_addr(&self, addr_str: &str) -> Result<SocketAddr, Box<dyn std::error::Error>> {
        // Handle addresses like "127.0.0.1:8080" or "*:8080" or "[::1]:8080"
        if addr_str.starts_with('*') {
            let port_str = &addr_str[2..]; // Remove "*:"
            let port = port_str.parse::<u16>()?;
            Ok(SocketAddr::new(IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)), port))
        } else if addr_str.starts_with('[') && addr_str.contains("]:") {
            // IPv6 address in brackets like [::1]:8080
            let end_bracket = addr_str.find("]:").ok_or("Invalid IPv6 format")?;
            let ip_str = &addr_str[1..end_bracket]; // Remove [ and ]
            let port_str = &addr_str[end_bracket + 2..]; // Remove ]:
            let ip = ip_str.parse::<std::net::Ipv6Addr>()?;
            let port = port_str.parse::<u16>()?;
            Ok(SocketAddr::new(IpAddr::V6(ip), port))
        } else if addr_str.contains(':') && !addr_str.starts_with('[') {
            // IPv4 address like 127.0.0.1:8080
            let parts: Vec<&str> = addr_str.rsplitn(2, ':').collect();
            if parts.len() == 2 {
                let port = parts[0].parse::<u16>()?;
                let ip_str = parts[1];
                let ip = ip_str.parse::<std::net::Ipv4Addr>()?;
                Ok(SocketAddr::new(IpAddr::V4(ip), port))
            } else {
                Err("Invalid IPv4 address format".into())
            }
        } else {
            // Try to parse as regular socket address
            addr_str.parse::<SocketAddr>().map_err(|e| format!("Failed to parse '{}': {}", addr_str, e).into())
        }
    }

    fn update_stats(&mut self) {
        if let Ok(connections) = self.connections.lock() {
            self.stats.total_connections = connections.len();
            self.stats.tcp_connections = connections.iter().filter(|c| c.protocol == "TCP").count();
            self.stats.udp_connections = connections.iter().filter(|c| c.protocol == "UDP").count();
            self.stats.listening_ports = connections.iter().filter(|c| c.state == "LISTEN").count();
            self.stats.established_connections = connections.iter().filter(|c| c.state == "ESTABLISHED").count();
            self.stats.last_updated = Instant::now();
        }
    }

    fn render_ui(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.heading("🔍 macOS Network Connection Monitor");
            
            // Stats panel
            ui.horizontal(|ui| {
                ui.label(format!("Total: {}", self.stats.total_connections));
                ui.label(format!("TCP: {}", self.stats.tcp_connections));
                ui.label(format!("UDP: {}", self.stats.udp_connections));
                ui.label(format!("Listening: {}", self.stats.listening_ports));
                ui.label(format!("Established: {}", self.stats.established_connections));
            });
            
            ui.separator();
            
            // Control panel
            ui.horizontal(|ui| {
                ui.label("Update interval:");
                let mut secs = self.update_interval.as_secs() as f32;
                ui.add(egui::Slider::new(&mut secs, 1.0..=10.0)
                    .text("seconds"));
                self.update_interval = Duration::from_secs(secs as u64);
                
                ui.separator();
                
                ui.checkbox(&mut self.show_local_only, "Local only");
                ui.checkbox(&mut self.show_remote_only, "Remote only");
                
                ui.separator();
                
                ui.label("Method:");
                ui.checkbox(&mut self.use_low_level, "Low-level API");
                if ui.button("Force Traditional").clicked() {
                    self.use_low_level = false;
                }
                
                ui.separator();
                
                ui.label("Proxy:");
                ui.checkbox(&mut self.proxy_manager.global_enabled, "Enable Proxy Routing");
                if ui.button("Configure Proxies").clicked() {
                    self.show_proxy_config = true;
                }
                if ui.button("Manage Rules").clicked() {
                    self.show_proxy_rules = true;
                }
                
                if ui.button("Start Traffic Interception").clicked() {
                    if self.traffic_interceptor.is_none() {
                        let proxy_manager = Arc::new(Mutex::new(self.proxy_manager.clone()));
                        self.traffic_interceptor = Some(TrafficInterceptor::new(proxy_manager));
                        if let Some(ref interceptor) = self.traffic_interceptor {
                            let _ = interceptor.start();
                        }
                    }
                }
                
                if ui.button("Stop Traffic Interception").clicked() {
                    if let Some(ref interceptor) = self.traffic_interceptor {
                        interceptor.stop();
                    }
                    self.traffic_interceptor = None;
                }
                
                if ui.button("View Intercepted Traffic").clicked() {
                    self.show_intercepted_traffic = true;
                }
                
                ui.separator();
                
                ui.label("Sort by:");
                egui::ComboBox::from_id_salt("sort_by")
                    .selected_text(format!("{:?}", self.sort_by))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.sort_by, SortBy::LocalAddr, "Local Address");
                        ui.selectable_value(&mut self.sort_by, SortBy::RemoteAddr, "Remote Address");
                        ui.selectable_value(&mut self.sort_by, SortBy::Process, "Process");
                        ui.selectable_value(&mut self.sort_by, SortBy::Protocol, "Protocol");
                        ui.selectable_value(&mut self.sort_by, SortBy::State, "State");
                        ui.selectable_value(&mut self.sort_by, SortBy::BytesSent, "Bytes Sent");
                        ui.selectable_value(&mut self.sort_by, SortBy::BytesReceived, "Bytes Received");
                    });
                
                ui.checkbox(&mut self.sort_ascending, "Ascending");
            });
            
            ui.separator();
            
            // Filter text input
            ui.horizontal(|ui| {
                ui.label("Filter:");
                ui.text_edit_singleline(&mut self.filter_text);
                if ui.button("Clear").clicked() {
                    self.filter_text.clear();
                }
            });
        });

        // Split the main area into two panels
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Left panel - Current connections
                ui.vertical(|ui| {
                    ui.heading("📊 Current Connections");
                    self.render_connections_table(ui);
                });
                
                ui.separator();
                
                // Right panel - Connection log
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading("📝 Connection Log");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Clear Log").clicked() {
                                if let Ok(mut log) = self.connection_log.lock() {
                                    log.clear();
                                }
                            }
                        });
                    });
                    
                    // Log filter
                    ui.horizontal(|ui| {
                        ui.label("Log Filter:");
                        ui.text_edit_singleline(&mut self.log_filter_text);
                        if ui.button("Clear").clicked() {
                            self.log_filter_text.clear();
                        }
                    });
                    
                    self.render_connection_log(ui);
                });
            });
        });

        // Connection details dialog
        if self.show_log_dialog {
            self.render_connection_dialog(ctx);
        }
        
        // Proxy configuration dialog
        if self.show_proxy_config {
            self.render_proxy_config_dialog(ctx);
        }
        
        // Proxy rules dialog
        if self.show_proxy_rules {
            self.render_proxy_rules_dialog(ctx);
        }
        
        // Intercepted traffic dialog
        if self.show_intercepted_traffic {
            self.render_intercepted_traffic_dialog(ctx);
        }
    }

    fn render_connections_table(&mut self, ui: &mut egui::Ui) {
        let connections = if let Ok(conns) = self.connections.lock() {
            conns.clone()
        } else {
            return;
        };

        let filtered_connections: Vec<_> = connections
            .iter()
            .filter(|conn| {
                // Apply filters
                if self.show_local_only && conn.remote_addr.is_some() {
                    return false;
                }
                if self.show_remote_only && conn.remote_addr.is_none() {
                    return false;
                }
                if !self.filter_text.is_empty() {
                    let filter_lower = self.filter_text.to_lowercase();
                    conn.local_addr.to_string().to_lowercase().contains(&filter_lower)
                        || conn.remote_addr.map(|addr| addr.to_string().to_lowercase().contains(&filter_lower)).unwrap_or(false)
                        || conn.process_name.to_lowercase().contains(&filter_lower)
                        || conn.protocol.to_lowercase().contains(&filter_lower)
                        || conn.state.to_lowercase().contains(&filter_lower)
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        // Sort connections
        let mut sorted_connections = filtered_connections;
        sorted_connections.sort_by(|a, b| {
            let ordering = match self.sort_by {
                SortBy::LocalAddr => a.local_addr.cmp(&b.local_addr),
                SortBy::RemoteAddr => {
                    match (a.remote_addr, b.remote_addr) {
                        (Some(addr_a), Some(addr_b)) => addr_a.cmp(&addr_b),
                        (Some(_), None) => std::cmp::Ordering::Greater,
                        (None, Some(_)) => std::cmp::Ordering::Less,
                        (None, None) => std::cmp::Ordering::Equal,
                    }
                }
                SortBy::Process => a.process_name.cmp(&b.process_name),
                SortBy::Protocol => a.protocol.cmp(&b.protocol),
                SortBy::State => a.state.cmp(&b.state),
                SortBy::BytesSent => a.bytes_sent.cmp(&b.bytes_sent),
                SortBy::BytesReceived => a.bytes_received.cmp(&b.bytes_received),
            };
            
            if self.sort_ascending {
                ordering
            } else {
                ordering.reverse()
            }
        });

        // Table header
        egui::Grid::new("connections_grid")
            .num_columns(9)
            .spacing([4.0, 2.0])
            .show(ui, |ui| {
                ui.label("Local Address");
                ui.label("Remote Address");
                ui.label("Protocol");
                ui.label("State");
                ui.label("Process");
                ui.label("PID");
                ui.label("Proxy");
                ui.label("Bytes Sent");
                ui.label("Bytes Received");
                ui.end_row();

                // Connection rows
                for (idx, conn) in sorted_connections.iter().enumerate() {
                    let is_selected = self.selected_connection == Some(idx);
                    
                    if ui.selectable_label(is_selected, &conn.local_addr.to_string()).clicked() {
                        self.selected_connection = Some(idx);
                    }
                    
                    ui.label(conn.remote_addr.map(|addr| addr.to_string()).unwrap_or_else(|| "N/A".to_string()));
                    ui.label(&conn.protocol);
                    ui.label(&conn.state);
                    ui.label(&conn.process_name);
                    ui.label(conn.process_id.to_string());
                    
                    // Show proxy info
                    let proxy_info = if let Some(remote_addr) = conn.remote_addr {
                        if let Some(proxy) = self.proxy_manager.get_proxy_for_connection(&remote_addr) {
                            format!("{}:{}", proxy.host, proxy.port)
                        } else {
                            "Direct".to_string()
                        }
                    } else {
                        "N/A".to_string()
                    };
                    ui.label(proxy_info);
                    
                    ui.label(format!("{}", conn.bytes_sent));
                    ui.label(format!("{}", conn.bytes_received));
                    ui.end_row();
                }
            });

        // Connection details
        if let Some(selected_idx) = self.selected_connection {
            if let Some(conn) = sorted_connections.get(selected_idx) {
                ui.separator();
                ui.group(|ui| {
                    ui.heading("Connection Details");
                    ui.label(format!("Local Address: {}", conn.local_addr));
                    if let Some(remote) = conn.remote_addr {
                        ui.label(format!("Remote Address: {}", remote));
                    }
                    ui.label(format!("Protocol: {}", conn.protocol));
                    ui.label(format!("State: {}", conn.state));
                    ui.label(format!("Process: {} (PID: {})", conn.process_name, conn.process_id));
                    ui.label(format!("Bytes Sent: {}", conn.bytes_sent));
                    ui.label(format!("Bytes Received: {}", conn.bytes_received));
                    ui.label(format!("Last Updated: {:?}", conn.last_updated.elapsed()));
                });
            }
        }
    }

    fn render_connection_log(&mut self, ui: &mut egui::Ui) {
        let log_entries = if let Ok(log) = self.connection_log.lock() {
            log.clone()
        } else {
            return;
        };

        let filtered_entries: Vec<_> = log_entries
            .iter()
            .filter(|entry| {
                if !self.log_filter_text.is_empty() {
                    let filter_lower = self.log_filter_text.to_lowercase();
                    entry.connection.local_addr.to_string().to_lowercase().contains(&filter_lower)
                        || entry.connection.remote_addr.map(|addr| addr.to_string().to_lowercase().contains(&filter_lower)).unwrap_or(false)
                        || entry.connection.process_name.to_lowercase().contains(&filter_lower)
                        || entry.connection.protocol.to_lowercase().contains(&filter_lower)
                        || entry.connection.state.to_lowercase().contains(&filter_lower)
                        || format!("{:?}", entry.event_type).to_lowercase().contains(&filter_lower)
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        // Log entries table
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("log_grid")
                .num_columns(6)
                .spacing([4.0, 2.0])
                .show(ui, |ui| {
                    ui.label("Time");
                    ui.label("Event");
                    ui.label("Local Address");
                    ui.label("Remote Address");
                    ui.label("Process");
                    ui.label("Protocol");
                    ui.end_row();

                    for (idx, entry) in filtered_entries.iter().enumerate() {
                        let is_selected = self.selected_log_entry == Some(idx);
                        
                        // Format timestamp
                        let timestamp = entry.timestamp.duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        let time_str = format!("{}", timestamp % 86400); // Show seconds since midnight
                        
                        // Event type with color
                        let event_color = match entry.event_type {
                            ConnectionEvent::New => egui::Color32::GREEN,
                            ConnectionEvent::Closed => egui::Color32::RED,
                            ConnectionEvent::Updated => egui::Color32::YELLOW,
                            ConnectionEvent::Established => egui::Color32::BLUE,
                        };
                        
                        if ui.selectable_label(is_selected, &time_str).clicked() {
                            self.selected_log_entry = Some(idx);
                            self.show_log_dialog = true;
                        }
                        
                        ui.colored_label(event_color, format!("{:?}", entry.event_type));
                        ui.label(&entry.connection.local_addr.to_string());
                        ui.label(entry.connection.remote_addr.map(|addr| addr.to_string()).unwrap_or_else(|| "N/A".to_string()));
                        ui.label(&entry.connection.process_name);
                        ui.label(&entry.connection.protocol);
                        ui.end_row();
                    }
                });
        });
    }

    fn render_connection_dialog(&mut self, ctx: &egui::Context) {
        let log_entries = if let Ok(log) = self.connection_log.lock() {
            log.clone()
        } else {
            return;
        };

        let filtered_entries: Vec<_> = log_entries
            .iter()
            .filter(|entry| {
                if !self.log_filter_text.is_empty() {
                    let filter_lower = self.log_filter_text.to_lowercase();
                    entry.connection.local_addr.to_string().to_lowercase().contains(&filter_lower)
                        || entry.connection.remote_addr.map(|addr| addr.to_string().to_lowercase().contains(&filter_lower)).unwrap_or(false)
                        || entry.connection.process_name.to_lowercase().contains(&filter_lower)
                        || entry.connection.protocol.to_lowercase().contains(&filter_lower)
                        || entry.connection.state.to_lowercase().contains(&filter_lower)
                        || format!("{:?}", entry.event_type).to_lowercase().contains(&filter_lower)
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        if let Some(selected_idx) = self.selected_log_entry {
            if let Some(entry) = filtered_entries.get(selected_idx) {
                let mut close_dialog = false;
                egui::Window::new("Connection Details")
                    .open(&mut self.show_log_dialog)
                    .show(ctx, |ui| {
                        ui.heading("Connection Details");
                        ui.separator();
                        
                        ui.horizontal(|ui| {
                            ui.label("Event Type:");
                            let event_color = match entry.event_type {
                                ConnectionEvent::New => egui::Color32::GREEN,
                                ConnectionEvent::Closed => egui::Color32::RED,
                                ConnectionEvent::Updated => egui::Color32::YELLOW,
                                ConnectionEvent::Established => egui::Color32::BLUE,
                            };
                            ui.colored_label(event_color, format!("{:?}", entry.event_type));
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Timestamp:");
                            ui.label(format!("{:?}", entry.timestamp));
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Local Address:");
                            ui.label(&entry.connection.local_addr.to_string());
                        });
                        
                        if let Some(remote) = entry.connection.remote_addr {
                            ui.horizontal(|ui| {
                                ui.label("Remote Address:");
                                ui.label(&remote.to_string());
                            });
                        }
                        
                        ui.horizontal(|ui| {
                            ui.label("Protocol:");
                            ui.label(&entry.connection.protocol);
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("State:");
                            ui.label(&entry.connection.state);
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Process:");
                            ui.label(&entry.connection.process_name);
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Process ID:");
                            ui.label(entry.connection.process_id.to_string());
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Bytes Sent:");
                            ui.label(entry.connection.bytes_sent.to_string());
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Bytes Received:");
                            ui.label(entry.connection.bytes_received.to_string());
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Interface:");
                            ui.label(&entry.connection.interface);
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Log Entry ID:");
                            ui.label(entry.id.to_string());
                        });
                        
                        ui.separator();
                        
                        ui.horizontal(|ui| {
                            if ui.button("Close").clicked() {
                                close_dialog = true;
                            }
                        });
                    });
                
                if close_dialog {
                    self.show_log_dialog = false;
                }
            }
        }
    }
    
    fn render_proxy_config_dialog(&mut self, ctx: &egui::Context) {
        let mut close_dialog = false;
        
        egui::Window::new("Proxy Configuration")
            .open(&mut self.show_proxy_config)
            .show(ctx, |ui| {
                ui.heading("Proxy Servers");
                ui.separator();
                
                // List existing proxies
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut proxies_to_remove = Vec::new();
                    let mut proxies_to_toggle = Vec::new();
                    
                    for proxy in &self.proxy_manager.proxies {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}: {}:{} ({})", 
                                proxy.name, proxy.host, proxy.port, proxy.proxy_type));
                            
                            let mut enabled = proxy.enabled;
                            ui.checkbox(&mut enabled, "Enabled");
                            if enabled != proxy.enabled {
                                proxies_to_toggle.push(proxy.id);
                            }
                            
                            if ui.button("Remove").clicked() {
                                proxies_to_remove.push(proxy.id);
                            }
                        });
                    }
                    
                    // Apply changes after iteration
                    for proxy_id in proxies_to_remove {
                        self.proxy_manager.remove_proxy(proxy_id);
                    }
                    for proxy_id in proxies_to_toggle {
                        if let Some(proxy) = self.proxy_manager.proxies.iter_mut().find(|p| p.id == proxy_id) {
                            proxy.enabled = !proxy.enabled;
                        }
                    }
                });
                
                ui.separator();
                
                // Add new proxy
                ui.heading("Add New Proxy");
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.new_proxy_name);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Host:");
                    ui.text_edit_singleline(&mut self.new_proxy_host);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Port:");
                    ui.text_edit_singleline(&mut self.new_proxy_port);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Type:");
                    egui::ComboBox::from_id_salt("proxy_type")
                        .selected_text(format!("{}", self.new_proxy_type))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.new_proxy_type, ProxyType::Socks5, "SOCKS5");
                            ui.selectable_value(&mut self.new_proxy_type, ProxyType::Http, "HTTP");
                            ui.selectable_value(&mut self.new_proxy_type, ProxyType::Socks4, "SOCKS4");
                        });
                });
                
                if ui.button("Add Proxy").clicked() {
                    if !self.new_proxy_name.is_empty() && !self.new_proxy_host.is_empty() {
                        if let Ok(port) = self.new_proxy_port.parse::<u16>() {
                            self.proxy_manager.add_proxy(
                                self.new_proxy_name.clone(),
                                self.new_proxy_host.clone(),
                                port,
                                self.new_proxy_type.clone()
                            );
                            
                            // Clear form
                            self.new_proxy_name.clear();
                            self.new_proxy_host.clear();
                            self.new_proxy_port.clear();
                        }
                    }
                }
                
                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        close_dialog = true;
                    }
                });
            });
        
        if close_dialog {
            self.show_proxy_config = false;
        }
    }
    
    fn render_proxy_rules_dialog(&mut self, ctx: &egui::Context) {
        let mut close_dialog = false;
        
        egui::Window::new("Proxy Rules")
            .open(&mut self.show_proxy_rules)
            .show(ctx, |ui| {
                ui.heading("Routing Rules");
                ui.separator();
                
                // List existing rules
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut rules_to_remove = Vec::new();
                    let mut rules_to_toggle = Vec::new();
                    
                    for rule in &self.proxy_manager.rules {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}: {} -> Proxy {}", 
                                rule.name, rule.pattern, rule.proxy_id));
                            
                            let mut enabled = rule.enabled;
                            ui.checkbox(&mut enabled, "Enabled");
                            if enabled != rule.enabled {
                                rules_to_toggle.push(rule.id);
                            }
                            
                            if ui.button("Remove").clicked() {
                                rules_to_remove.push(rule.id);
                            }
                        });
                    }
                    
                    // Apply changes after iteration
                    for rule_id in rules_to_remove {
                        self.proxy_manager.remove_rule(rule_id);
                    }
                    for rule_id in rules_to_toggle {
                        if let Some(rule) = self.proxy_manager.rules.iter_mut().find(|r| r.id == rule_id) {
                            rule.enabled = !rule.enabled;
                        }
                    }
                });
                
                ui.separator();
                
                // Add new rule
                ui.heading("Add New Rule");
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.new_rule_name);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Pattern:");
                    ui.text_edit_singleline(&mut self.new_rule_pattern);
                    ui.label("(e.g., *.kion.cloud, 100.64.1.*)");
                });
                
                ui.horizontal(|ui| {
                    ui.label("Proxy:");
                    egui::ComboBox::from_id_salt("proxy_selection")
                        .selected_text(if let Some(proxy_id) = self.selected_proxy_for_rule {
                            self.proxy_manager.proxies.iter()
                                .find(|p| p.id == proxy_id)
                                .map(|p| p.name.clone())
                                .unwrap_or_else(|| "Select proxy".to_string())
                        } else {
                            "Select proxy".to_string()
                        })
                        .show_ui(ui, |ui| {
                            for proxy in &self.proxy_manager.proxies {
                                ui.selectable_value(&mut self.selected_proxy_for_rule, Some(proxy.id), &proxy.name);
                            }
                        });
                });
                
                if ui.button("Add Rule").clicked() {
                    if !self.new_rule_name.is_empty() && !self.new_rule_pattern.is_empty() {
                        if let Some(proxy_id) = self.selected_proxy_for_rule {
                            self.proxy_manager.add_rule(
                                self.new_rule_name.clone(),
                                self.new_rule_pattern.clone(),
                                proxy_id
                            );
                            
                            // Clear form
                            self.new_rule_name.clear();
                            self.new_rule_pattern.clear();
                            self.selected_proxy_for_rule = None;
                        }
                    }
                }
                
                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        close_dialog = true;
                    }
                });
            });
        
        if close_dialog {
            self.show_proxy_rules = false;
        }
    }
    
    fn render_intercepted_traffic_dialog(&mut self, ctx: &egui::Context) {
        let mut close_dialog = false;
        
        egui::Window::new("Intercepted Traffic")
            .open(&mut self.show_intercepted_traffic)
            .show(ctx, |ui| {
                ui.heading("Traffic Interception Results");
                ui.separator();
                
                if let Some(ref interceptor) = self.traffic_interceptor {
                    let intercepted_connections = interceptor.get_intercepted_connections();
                    
                    ui.label(format!("Total intercepted connections: {}", intercepted_connections.len()));
                    ui.separator();
                    
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (idx, conn) in intercepted_connections.iter().enumerate() {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(format!("Connection {}: {} -> {:?}", 
                                        idx + 1,
                                        conn.original_connection.local_addr,
                                        conn.original_connection.remote_addr
                                    ));
                                    
                                    let status_color = match conn.status {
                                        traffic_interceptor::InterceptionStatus::Proxied => egui::Color32::GREEN,
                                        traffic_interceptor::InterceptionStatus::Direct => egui::Color32::BLUE,
                                        traffic_interceptor::InterceptionStatus::Failed => egui::Color32::RED,
                                        traffic_interceptor::InterceptionStatus::Pending => egui::Color32::YELLOW,
                                    };
                                    
                                    ui.colored_label(status_color, format!("{:?}", conn.status));
                                });
                                
                                if let Some(ref proxy) = conn.proxy_used {
                                    ui.label(format!("Proxy: {}:{} ({})", 
                                        proxy.host, proxy.port, proxy.proxy_type));
                                } else {
                                    ui.label("Direct connection");
                                }
                                
                                ui.label(format!("Intercepted at: {:?}", conn.intercepted_at.elapsed()));
                            });
                        }
                    });
                } else {
                    ui.label("Traffic interception is not active.");
                    ui.label("Click 'Start Traffic Interception' to begin monitoring.");
                }
                
                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        close_dialog = true;
                    }
                });
            });
        
        if close_dialog {
            self.show_intercepted_traffic = false;
        }
    }
}


fn main() -> Result<(), eframe::Error> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "macOS Network Connection Monitor",
        options,
        Box::new(|_cc| Ok(Box::new(MacosListenerApp::new()))),
    )
}