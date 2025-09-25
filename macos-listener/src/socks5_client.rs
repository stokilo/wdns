use std::net::{IpAddr, SocketAddr};
use std::io::{Read, Write};
use std::net::TcpStream;
use crate::{ProxyConfig, ProxyType};

#[derive(Debug)]
pub struct Socks5Client {
    proxy_config: ProxyConfig,
}

impl Socks5Client {
    pub fn new(proxy_config: ProxyConfig) -> Self {
        Self { proxy_config }
    }
    
    pub fn connect(&self, target_addr: SocketAddr) -> Result<TcpStream, Box<dyn std::error::Error>> {
        match self.proxy_config.proxy_type {
            ProxyType::Socks5 => self.connect_socks5(target_addr),
            ProxyType::Http => self.connect_http(target_addr),
            ProxyType::Socks4 => self.connect_socks4(target_addr),
        }
    }
    
    fn connect_socks5(&self, target_addr: SocketAddr) -> Result<TcpStream, Box<dyn std::error::Error>> {
        let proxy_addr = SocketAddr::new(
            self.proxy_config.host.parse::<IpAddr>()?,
            self.proxy_config.port
        );
        
        let mut stream = TcpStream::connect(proxy_addr)?;
        
        // SOCKS5 handshake
        // Step 1: Send authentication methods
        let auth_methods = if self.proxy_config.username.is_some() {
            vec![0x02, 0x00] // Username/password and no auth
        } else {
            vec![0x00] // No authentication
        };
        
        let mut request = vec![0x05, auth_methods.len() as u8];
        request.extend_from_slice(&auth_methods);
        stream.write_all(&request)?;
        
        // Step 2: Receive server's choice
        let mut response = [0u8; 2];
        stream.read_exact(&mut response)?;
        
        if response[0] != 0x05 {
            return Err("Invalid SOCKS5 version".into());
        }
        
        // Handle authentication if required
        if response[1] == 0x02 && self.proxy_config.username.is_some() {
            self.authenticate_socks5(&mut stream)?;
        } else if response[1] != 0x00 {
            return Err("Authentication failed".into());
        }
        
        // Step 3: Send connection request
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
        
        // Step 4: Receive connection response
        let mut response = vec![0u8; 4];
        stream.read_exact(&mut response)?;
        
        if response[0] != 0x05 || response[1] != 0x00 {
            return Err("SOCKS5 connection failed".into());
        }
        
        // Skip the rest of the response (bound address)
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
        
        Ok(stream)
    }
    
    fn authenticate_socks5(&self, stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        let username = self.proxy_config.username.as_ref().unwrap();
        let password = self.proxy_config.password.as_ref().unwrap();
        
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
    
    fn connect_http(&self, _target_addr: SocketAddr) -> Result<TcpStream, Box<dyn std::error::Error>> {
        // HTTP proxy implementation would go here
        Err("HTTP proxy not implemented yet".into())
    }
    
    fn connect_socks4(&self, _target_addr: SocketAddr) -> Result<TcpStream, Box<dyn std::error::Error>> {
        // SOCKS4 proxy implementation would go here
        Err("SOCKS4 proxy not implemented yet".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_socks5_client_creation() {
        let proxy_config = ProxyConfig {
            id: 1,
            name: "Test Proxy".to_string(),
            host: "127.0.0.1".to_string(),
            port: 1080,
            proxy_type: ProxyType::Socks5,
            username: None,
            password: None,
            enabled: true,
        };
        
        let client = Socks5Client::new(proxy_config);
        assert_eq!(client.proxy_config.host, "127.0.0.1");
        assert_eq!(client.proxy_config.port, 1080);
    }
}
