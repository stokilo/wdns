use anyhow::Result;
use hyper::client::HttpConnector;
use hyper::http::{HeaderValue, Method, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Request, Response, Server};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tracing::{debug, error, info};

pub struct ProxyServer {
    pub bind_addr: SocketAddr,
    client: Client<HttpConnector>,
}

impl ProxyServer {
    pub fn new(bind_addr: SocketAddr) -> Self {
        let client = Client::builder()
            .http1_title_case_headers(true)
            .http1_allow_obsolete_multiline_headers_in_responses(true)
            .build_http();

        Self { bind_addr, client }
    }

    pub async fn run(self) -> Result<()> {
        info!("Starting proxy server on {}", self.bind_addr);

        let client = Arc::new(self.client);

        let make_svc = make_service_fn(move |_conn| {
            let client = client.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    let client = client.clone();
                    handle_request(req, client)
                }))
            }
        });

        let server = Server::bind(&self.bind_addr).serve(make_svc);

        info!("Proxy server listening on {}", self.bind_addr);

        if let Err(e) = server.await {
            error!("Proxy server error: {}", e);
        }

        Ok(())
    }
}

async fn handle_request(
    req: Request<Body>,
    client: Arc<Client<HttpConnector>>,
) -> Result<Response<Body>, Infallible> {
    debug!("Received request: {} {}", req.method(), req.uri());

    // Handle CONNECT method for HTTPS tunneling
    if req.method() == Method::CONNECT {
        return handle_connect(req).await;
    }

    // Handle regular HTTP requests
    handle_http_request(req, client).await
}

async fn handle_connect(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let authority = match req.uri().authority() {
        Some(auth) => auth.clone(),
        None => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("Missing authority"))
                .unwrap());
        }
    };

    debug!("CONNECT request to: {}", authority);

    // Parse the target address
    let port = authority.port().map(|p| p.as_str().to_string()).unwrap_or("443".to_string());
    let target_addr = format!("{}:{}", authority.host(), port);

    // Connect to the target server
    match TcpStream::connect(&target_addr).await {
        Ok(_target_stream) => {
            debug!("Connected to target: {}", target_addr);

            // Send 200 Connection Established response
            let response = Response::builder()
                .status(StatusCode::OK)
                .body(Body::from("Connection established"))
                .unwrap();

            Ok(response)
        }
        Err(e) => {
            error!("Failed to connect to target {}: {}", target_addr, e);
            Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Body::from(format!("Failed to connect to target: {}", e)))
                .unwrap())
        }
    }
}

async fn handle_http_request(
    mut req: Request<Body>,
    client: Arc<Client<HttpConnector>>,
) -> Result<Response<Body>, Infallible> {
    // Remove proxy-specific headers
    req.headers_mut().remove("proxy-connection");
    req.headers_mut().remove("proxy-authorization");

    // Set the correct host header
    if let Some(host) = req.uri().host() {
        if let Ok(host_value) = HeaderValue::from_str(host) {
            req.headers_mut().insert("host", host_value);
        }
    }

    debug!("Forwarding request to: {}", req.uri());

    // Forward the request
    match client.request(req).await {
        Ok(response) => {
            debug!("Received response: {}", response.status());
            Ok(response)
        }
        Err(e) => {
            error!("Request failed: {}", e);
            Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Body::from(format!("Proxy error: {}", e)))
                .unwrap())
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    #[tokio::test]
    async fn test_proxy_server_creation() {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let proxy = ProxyServer::new(addr);
        assert_eq!(proxy.bind_addr, addr);
    }

    #[tokio::test]
    async fn test_proxy_server_bind() {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let proxy = ProxyServer::new(addr);
        
        // Test that we can create the server (though we won't run it in tests)
        // This is mainly to ensure the struct can be created
        assert_eq!(proxy.bind_addr, addr);
    }

    #[test]
    fn test_proxy_server_new() {
        let addr: SocketAddr = "127.0.0.1:9701".parse().unwrap();
        let proxy = ProxyServer::new(addr);
        assert_eq!(proxy.bind_addr, addr);
    }
}
