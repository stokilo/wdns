use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::process::Command;
use std::net::{IpAddr, SocketAddr};
use std::collections::VecDeque;

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