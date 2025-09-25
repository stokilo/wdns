use anyhow::Result;
use std::sync::Arc;
use tracing::info;
use warp::Filter;

mod dns;
mod service;
mod config;
mod api;
mod proxy;
mod socks5;
mod ssh_tunnel;

use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("wdns=debug,warp=info")
        .init();

    info!("Starting WDNS Service...");

    // Load configuration
    let config = Config::load()?;
    info!("Configuration loaded: {:?}", config);

    // Check if running as Windows service
    if service::is_service_mode() {
        service::run_as_service().await?;
    } else {
        // Run as standalone application
        run_standalone(config).await?;
    }

    Ok(())
}

async fn run_standalone(config: Config) -> Result<()> {
    let dns_resolver = Arc::new(dns::DnsResolver::new()?);
    
    info!("DNS service listening on {}", config.bind_address);

    // Health check endpoint
    let health = warp::path("health")
        .and(warp::get())
        .map(|| warp::reply::json(&serde_json::json!({
            "status": "healthy",
            "service": "wdns"
        })));

    // Root endpoint
    let proxy_enabled = config.proxy_enabled;
    let socks5_enabled = config.socks5_enabled;
    let root = warp::path::end()
        .and(warp::get())
        .map(move || warp::reply::json(&serde_json::json!({
            "service": "WDNS",
            "version": "0.1.0",
            "endpoints": ["/health", "/api/dns/resolve"],
            "proxy_enabled": proxy_enabled,
            "proxy_port": if proxy_enabled { Some(9701) } else { None },
            "socks5_enabled": socks5_enabled,
            "socks5_port": if socks5_enabled { Some(9702) } else { None }
        })));

    // DNS resolution endpoint
    let dns_resolver_filter = warp::any().map(move || dns_resolver.clone());
    
    let dns_resolve = warp::path("api")
        .and(warp::path("dns"))
        .and(warp::path("resolve"))
        .and(warp::post())
        .and(warp::body::json())
        .and(dns_resolver_filter)
        .and_then(handle_dns_resolve);

    let routes = health.or(root).or(dns_resolve);

    // Start DNS service
    let dns_server = warp::serve(routes).run(config.bind_addr()?);

    // Start proxy servers if enabled
    let mut tasks = vec![];
    
    if config.proxy_enabled {
        info!("HTTP Proxy server listening on {}", config.proxy_bind_address);
        let proxy_server = proxy::ProxyServer::new(config.proxy_bind_addr()?);
        tasks.push(tokio::spawn(async move {
            if let Err(e) = proxy_server.run().await {
                tracing::error!("HTTP Proxy server error: {}", e);
            }
        }));
    }

    if config.socks5_enabled {
        info!("SOCKS5 server listening on {}", config.socks5_bind_address);
        let socks5_server = socks5::Socks5Server::new(config.socks5_bind_addr()?)?;
        tasks.push(tokio::spawn(async move {
            if let Err(e) = socks5_server.run().await {
                tracing::error!("SOCKS5 server error: {}", e);
            }
        }));
    }

    // Start SSH tunnel if configured
    if let Some(ssh_config) = config.ssh_tunnel_config.clone() {
        info!("Starting SSH tunnel to {}:{}", ssh_config.host, ssh_config.port);
        let ssh_tunnel = ssh_tunnel::SshTunnelManager::new(ssh_config);
        tasks.push(tokio::spawn(async move {
            if let Err(e) = ssh_tunnel.start().await {
                tracing::error!("SSH tunnel error: {}", e);
            }
        }));
    }

    // Run all servers concurrently
    if tasks.is_empty() {
        info!("No proxy servers enabled");
        dns_server.await;
    } else {
        tasks.push(tokio::spawn(async move {
            dns_server.await;
        }));

        tokio::select! {
            _ = futures::future::join_all(tasks) => {
                info!("All servers stopped");
            }
        }
    }

    Ok(())
}

async fn handle_dns_resolve(
    request: dns::DnsRequest,
    dns_resolver: Arc<dns::DnsResolver>,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Validate request
    if request.hosts.is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "error": "No hosts provided"
            })),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Resolve DNS
    let dns_response = dns_resolver.resolve_hosts(request.hosts).await;

    Ok(warp::reply::with_status(
        warp::reply::json(&dns_response),
        warp::http::StatusCode::OK,
    ))
}