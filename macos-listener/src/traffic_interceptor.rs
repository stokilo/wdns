use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::net::UdpSocket;
use crate::{ProxyConfig, ProxyManager, NetworkConnection};

/// Low-level traffic interceptor that captures and routes traffic through external SOCKS5 proxy
pub struct TrafficInterceptor {
    proxy_manager: Arc<Mutex<ProxyManager>>,
    is_running: Arc<Mutex<bool>>,
    intercepted_connections: Arc<Mutex<Vec<InterceptedConnection>>>,
    connection_counter: Arc<Mutex<u64>>,
}

#[derive(Debug, Clone)]
pub struct InterceptedConnection {
    pub id: u64,
    pub original_connection: NetworkConnection,
    pub proxy_used: Option<ProxyConfig>,
    pub intercepted_at: std::time::Instant,
    pub status: InterceptionStatus,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub domain: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterceptionStatus {
    Pending,
    Proxied,
    Failed,
    Direct,
    Timeout,
}

impl TrafficInterceptor {
    pub fn new(proxy_manager: Arc<Mutex<ProxyManager>>) -> Self {
        Self {
            proxy_manager,
            is_running: Arc::new(Mutex::new(false)),
            intercepted_connections: Arc::new(Mutex::new(Vec::new())),
            connection_counter: Arc::new(Mutex::new(0)),
        }
    }

    /// Start low-level traffic interception
    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut is_running = self.is_running.lock().unwrap();
        if *is_running {
            return Ok(());
        }
        *is_running = true;
        drop(is_running);

        println!("🚀 Starting low-level traffic interception...");
        println!("📋 This will intercept ALL system traffic and route matching connections through SOCKS5 proxy");
        
        // Log current configuration
        self.log_interception_configuration();

        // Start system-level traffic interception
        let proxy_manager = Arc::clone(&self.proxy_manager);
        let is_running = Arc::clone(&self.is_running);
        let intercepted_connections = Arc::clone(&self.intercepted_connections);
        let connection_counter = Arc::clone(&self.connection_counter);

        thread::spawn(move || {
            if let Err(e) = Self::interception_loop(
                proxy_manager,
                is_running,
                intercepted_connections,
                connection_counter,
            ) {
                eprintln!("❌ Traffic interception error: {}", e);
            }
        });

        println!("✅ Traffic interceptor started successfully");
        Ok(())
    }

    /// Stop traffic interception
    pub fn stop(&self) {
        let mut is_running = self.is_running.lock().unwrap();
        *is_running = false;
        println!("🛑 Traffic interceptor stopped");
    }

    /// Get intercepted connections
    pub fn get_intercepted_connections(&self) -> Vec<InterceptedConnection> {
        self.intercepted_connections.lock().unwrap().clone()
    }

    /// Main interception loop
    fn interception_loop(
        proxy_manager: Arc<Mutex<ProxyManager>>,
        is_running: Arc<Mutex<bool>>,
        intercepted_connections: Arc<Mutex<Vec<InterceptedConnection>>>,
        connection_counter: Arc<Mutex<u64>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔍 Starting system-level traffic interception loop...");
        
        // Start DNS interception
        let dns_manager = Arc::clone(&proxy_manager);
        let dns_running = Arc::clone(&is_running);
        let dns_connections = Arc::clone(&intercepted_connections);
        let dns_counter = Arc::clone(&connection_counter);
        
        thread::spawn(move || {
            if let Err(e) = Self::intercept_dns_traffic(dns_manager, dns_running, dns_connections, dns_counter) {
                eprintln!("❌ DNS interception error: {}", e);
            }
        });

        // Start TCP interception
        let tcp_manager = Arc::clone(&proxy_manager);
        let tcp_running = Arc::clone(&is_running);
        let tcp_connections = Arc::clone(&intercepted_connections);
        let tcp_counter = Arc::clone(&connection_counter);
        
        thread::spawn(move || {
            if let Err(e) = Self::intercept_tcp_traffic(tcp_manager, tcp_running, tcp_connections, tcp_counter) {
                eprintln!("❌ TCP interception error: {}", e);
            }
        });

        // Start UDP interception
        let udp_manager = Arc::clone(&proxy_manager);
        let udp_running = Arc::clone(&is_running);
        let udp_connections = Arc::clone(&intercepted_connections);
        let udp_counter = Arc::clone(&connection_counter);
        
        thread::spawn(move || {
            if let Err(e) = Self::intercept_udp_traffic(udp_manager, udp_running, udp_connections, udp_counter) {
                eprintln!("❌ UDP interception error: {}", e);
            }
        });

        // Monitor system traffic
        while *is_running.lock().unwrap() {
            thread::sleep(Duration::from_millis(100));
        }
        
        println!("🛑 Traffic interception loop stopped");
        Ok(())
    }

    /// Intercept DNS traffic at system level
    fn intercept_dns_traffic(
        proxy_manager: Arc<Mutex<ProxyManager>>,
        is_running: Arc<Mutex<bool>>,
        intercepted_connections: Arc<Mutex<Vec<InterceptedConnection>>>,
        connection_counter: Arc<Mutex<u64>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("🌐 Intercepting DNS traffic at system level...");
        
        // Create DNS interceptor socket
        let dns_socket = UdpSocket::bind("127.0.0.1:5353")?;
        println!("📡 DNS interceptor listening on 127.0.0.1:5353");
        
        let mut buffer = [0u8; 512];
        
        while *is_running.lock().unwrap() {
            match dns_socket.recv_from(&mut buffer) {
                Ok((size, client_addr)) => {
                    let mut counter = connection_counter.lock().unwrap();
                    *counter += 1;
                    let connection_id = *counter;
                    drop(counter);

                    println!("📨 DNS query #{} from {} ({} bytes)", connection_id, client_addr, size);
                    
                    // Parse DNS query
                    if let Some(domain) = Self::extract_domain_from_dns_packet(&buffer[..size]) {
                        println!("🔍 DNS query for domain: {}", domain);
                        
                        // Check if this domain should be proxied
                        if let Some(proxy_config) = Self::should_proxy_domain(&proxy_manager, &domain) {
                            println!("✅ DNS RULE MATCH! '{}' -> {} (proxy: {}:{})", 
                                     domain, proxy_config.name, proxy_config.host, proxy_config.port);
                            
                            // Route DNS query through SOCKS5 proxy
                            if let Ok(response) = Self::route_dns_through_socks5(&domain, &proxy_config) {
                                dns_socket.send_to(&response, client_addr)?;
                                println!("✅ DNS response sent to {}", client_addr);
                                
                                // Record intercepted connection
                                Self::record_intercepted_connection(
                                    &intercepted_connections,
                                    connection_id,
                                    domain,
                                    Some(proxy_config),
                                    InterceptionStatus::Proxied,
                                );
                            } else {
                                println!("❌ Failed to route DNS query through SOCKS5");
                                Self::record_intercepted_connection(
                                    &intercepted_connections,
                                    connection_id,
                                    domain,
                                    Some(proxy_config),
                                    InterceptionStatus::Failed,
                                );
                            }
                        } else {
                            println!("❌ No rule match for DNS domain: {}", domain);
                            // Forward to system DNS
                            if let Ok(response) = Self::forward_to_system_dns(&buffer[..size]) {
                                dns_socket.send_to(&response, client_addr)?;
                                println!("🔗 DNS forwarded to system DNS");
                                
                                Self::record_intercepted_connection(
                                    &intercepted_connections,
                                    connection_id,
                                    domain,
                                    None,
                                    InterceptionStatus::Direct,
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    if *is_running.lock().unwrap() {
                        eprintln!("❌ DNS socket error: {}", e);
                    }
                    break;
                }
            }
        }
        
        println!("🛑 DNS interception stopped");
        Ok(())
    }
}