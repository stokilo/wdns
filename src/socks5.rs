use anyhow::Result;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info};

#[derive(Debug, Clone)]
pub struct Socks5Server {
    pub bind_addr: SocketAddr,
}

impl Socks5Server {
    pub fn new(bind_addr: SocketAddr) -> Self {
        Self { bind_addr }
    }

    pub async fn run(self) -> Result<()> {
        info!("Starting SOCKS5 server on {}", self.bind_addr);

        let listener = TcpListener::bind(self.bind_addr).await?;
        info!("SOCKS5 server listening on {}", self.bind_addr);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    debug!("New SOCKS5 connection from {}", addr);
                    tokio::spawn(async move {
                        if let Err(e) = handle_socks5_connection(stream).await {
                            error!("SOCKS5 connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept SOCKS5 connection: {}", e);
                }
            }
        }
    }
}

async fn handle_socks5_connection(mut stream: TcpStream) -> Result<()> {
    let mut buffer = [0u8; 1024];
    
    // Read SOCKS5 greeting
    let n = stream.read(&mut buffer).await?;
    if n < 3 {
        return Err(anyhow::anyhow!("Invalid SOCKS5 greeting"));
    }

    let version = buffer[0];
    let nmethods = buffer[1] as usize;
    
    if version != 5 {
        return Err(anyhow::anyhow!("Unsupported SOCKS version: {}", version));
    }

    if n < 2 + nmethods {
        return Err(anyhow::anyhow!("Invalid SOCKS5 greeting length"));
    }

    // Check if no authentication is supported
    let mut no_auth_supported = false;
    for i in 0..nmethods {
        if buffer[2 + i] == 0 {
            no_auth_supported = true;
            break;
        }
    }

    if !no_auth_supported {
        // Send "no acceptable methods" response
        stream.write_all(&[5, 0xFF]).await?;
        return Err(anyhow::anyhow!("No acceptable authentication methods"));
    }

    // Send "no authentication required" response
    stream.write_all(&[5, 0]).await?;

    // Read connection request
    let n = stream.read(&mut buffer).await?;
    if n < 10 {
        return Err(anyhow::anyhow!("Invalid SOCKS5 request"));
    }

    let version = buffer[0];
    let cmd = buffer[1];
    let _rsv = buffer[2];
    let atyp = buffer[3];

    if version != 5 {
        return Err(anyhow::anyhow!("Invalid SOCKS5 version in request"));
    }

    if cmd != 1 {
        // Only support CONNECT command
        stream.write_all(&[5, 7, 0, 1, 0, 0, 0, 0, 0, 0]).await?;
        return Err(anyhow::anyhow!("Unsupported SOCKS5 command: {}", cmd));
    }

    // Parse destination address
    let (dest_addr, _addr_len) = match atyp {
        1 => {
            // IPv4
            if n < 10 {
                return Err(anyhow::anyhow!("Invalid IPv4 address length"));
            }
            let ip = Ipv4Addr::new(buffer[4], buffer[5], buffer[6], buffer[7]);
            let port = u16::from_be_bytes([buffer[8], buffer[9]]);
            (SocketAddr::new(IpAddr::V4(ip), port), 6)
        }
        3 => {
            // Domain name
            let domain_len = buffer[4] as usize;
            if n < 5 + domain_len + 2 {
                return Err(anyhow::anyhow!("Invalid domain name length"));
            }
            let _domain = String::from_utf8_lossy(&buffer[5..5 + domain_len]);
            let port = u16::from_be_bytes([buffer[5 + domain_len], buffer[5 + domain_len + 1]]);
            (SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port), 5 + domain_len + 2)
        }
        4 => {
            // IPv6
            if n < 22 {
                return Err(anyhow::anyhow!("Invalid IPv6 address length"));
            }
            let mut ip_bytes = [0u8; 16];
            ip_bytes.copy_from_slice(&buffer[4..20]);
            let ip = Ipv6Addr::from(ip_bytes);
            let port = u16::from_be_bytes([buffer[20], buffer[21]]);
            (SocketAddr::new(IpAddr::V6(ip), port), 18)
        }
        _ => {
            stream.write_all(&[5, 8, 0, 1, 0, 0, 0, 0, 0, 0]).await?;
            return Err(anyhow::anyhow!("Unsupported address type: {}", atyp));
        }
    };

    debug!("SOCKS5 request to connect to: {}", dest_addr);

    // Attempt to connect to destination
    match TcpStream::connect(dest_addr).await {
        Ok(dest_stream) => {
            debug!("Connected to destination: {}", dest_addr);
            
            // Send success response
            let mut response = vec![5, 0, 0, 1];
            response.extend_from_slice(&dest_addr.ip().to_string().as_bytes());
            response.extend_from_slice(&dest_addr.port().to_be_bytes());
            stream.write_all(&response).await?;

            // Start proxying data
            proxy_data(stream, dest_stream).await?;
        }
        Err(e) => {
            error!("Failed to connect to destination {}: {}", dest_addr, e);
            stream.write_all(&[5, 1, 0, 1, 0, 0, 0, 0, 0, 0]).await?;
            return Err(anyhow::anyhow!("Connection failed: {}", e));
        }
    }

    Ok(())
}

async fn proxy_data(
    mut client: TcpStream,
    mut dest: TcpStream,
) -> Result<()> {
    let (mut client_read, mut client_write) = client.split();
    let (mut dest_read, mut dest_write) = dest.split();

    // Create bidirectional proxy
    let client_to_dest = tokio::io::copy(&mut client_read, &mut dest_write);
    let dest_to_client = tokio::io::copy(&mut dest_read, &mut client_write);

    // Run both directions concurrently
    tokio::select! {
        result = client_to_dest => {
            if let Err(e) = result {
                debug!("Client to destination proxy error: {}", e);
            }
        }
        result = dest_to_client => {
            if let Err(e) = result {
                debug!("Destination to client proxy error: {}", e);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    #[test]
    fn test_socks5_server_creation() {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let server = Socks5Server::new(addr);
        assert_eq!(server.bind_addr, addr);
    }
}
