use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::process::Command;
use std::net::{IpAddr, SocketAddr};

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

pub struct MacosListenerApp {
    connections: Arc<Mutex<Vec<NetworkConnection>>>,
    last_update: Instant,
    update_interval: Duration,
    selected_connection: Option<usize>,
    filter_text: String,
    show_local_only: bool,
    show_remote_only: bool,
    sort_by: SortBy,
    sort_ascending: bool,
    stats: NetworkStats,
}

impl Default for MacosListenerApp {
    fn default() -> Self {
        Self {
            connections: Arc::new(Mutex::new(Vec::new())),
            last_update: Instant::now(),
            update_interval: Duration::from_secs(2),
            selected_connection: None,
            filter_text: String::new(),
            show_local_only: false,
            show_remote_only: false,
            sort_by: SortBy::LocalAddr,
            sort_ascending: true,
            stats: NetworkStats::default(),
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
        if let Ok(mut conns) = self.connections.lock() {
            *conns = connections;
        }
        self.update_stats();
    }

    fn get_network_connections(&self) -> Vec<NetworkConnection> {
        let mut connections = Vec::new();

        // Get TCP connections using netstat
        if let Ok(tcp_connections) = self.get_tcp_connections() {
            connections.extend(tcp_connections);
        }

        // Get UDP connections using netstat
        if let Ok(udp_connections) = self.get_udp_connections() {
            connections.extend(udp_connections);
        }

        // Get process information for each connection
        let process_map = self.get_process_map();
        
        for conn in &mut connections {
            if let Some(process_info) = process_map.get(&conn.process_id) {
                conn.process_name = process_info.name.clone();
            }
        }

        connections
    }

    fn get_tcp_connections(&self) -> Result<Vec<NetworkConnection>, Box<dyn std::error::Error>> {
        let output = Command::new("netstat")
            .args(&["-an", "-p", "tcp"])
            .output()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut connections = Vec::new();

        for line in output_str.lines() {
            if line.contains("tcp") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    if let Ok(local_addr) = self.parse_socket_addr(parts[3]) {
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

    fn get_process_map(&self) -> std::collections::HashMap<u32, ProcessInfo> {
        let mut process_map = std::collections::HashMap::new();
        
        if let Ok(output) = Command::new("lsof").args(&["-i", "-P", "-n"]).output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            for line in output_str.lines().skip(1) { // Skip header
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(pid) = parts[1].parse::<u32>() {
                        let name = parts[0].to_string();
                        process_map.insert(pid, ProcessInfo { name });
                    }
                }
            }
        }
        
        process_map
    }

    fn parse_socket_addr(&self, addr_str: &str) -> Result<SocketAddr, Box<dyn std::error::Error>> {
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

        Err("Invalid socket address".into())
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
        egui::CentralPanel::default().show(ctx, |ui| {
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
            
            ui.separator();
            
            // Connections table
            self.render_connections_table(ui);
        });
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
            .num_columns(8)
            .spacing([4.0, 2.0])
            .show(ui, |ui| {
                ui.label("Local Address");
                ui.label("Remote Address");
                ui.label("Protocol");
                ui.label("State");
                ui.label("Process");
                ui.label("PID");
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
}

#[derive(Debug, Clone)]
struct ProcessInfo {
    name: String,
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