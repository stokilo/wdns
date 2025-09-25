use anyhow::Result;
use std::sync::Arc;
use tokio::process::Command as TokioCommand;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::config::SshTunnelConfig;

pub struct SshTunnelManager {
    config: SshTunnelConfig,
    process: Arc<Mutex<Option<tokio::process::Child>>>,
    is_connected: Arc<Mutex<bool>>,
}

impl SshTunnelManager {
    pub fn new(config: SshTunnelConfig) -> Self {
        Self {
            config,
            process: Arc::new(Mutex::new(None)),
            is_connected: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting SSH tunnel to {}:{}", self.config.host, self.config.port);
        
        // Build SSH command for dynamic port forwarding
        let ssh_cmd = format!(
            "ssh -D {} -N -f {}@{} -p {}",
            self.config.local_port,
            self.config.username,
            self.config.host,
            self.config.port
        );

        info!("Executing SSH command: {}", ssh_cmd);

        // Start SSH tunnel process
        let mut cmd = TokioCommand::new("ssh");
        cmd.args(&[
            "-D", &self.config.local_port.to_string(),
            "-N", "-f",
            &format!("{}@{}", self.config.username, self.config.host),
            "-p", &self.config.port.to_string(),
        ]);

        // Add authentication
        if let Some(password) = &self.config.password {
            // Use sshpass for password authentication
            let mut sshpass_cmd = TokioCommand::new("sshpass");
            sshpass_cmd.args(&["-p", password]);
            sshpass_cmd.arg("ssh");
            sshpass_cmd.args(&[
                "-D", &self.config.local_port.to_string(),
                "-N", "-f",
                &format!("{}@{}", self.config.username, self.config.host),
                "-p", &self.config.port.to_string(),
            ]);
            
            let child = sshpass_cmd.spawn()?;
            {
                let mut process_guard = self.process.lock().await;
                *process_guard = Some(child);
            }
        } else if let Some(key_path) = &self.config.key_path {
            cmd.arg("-i").arg(key_path);
            let child = cmd.spawn()?;
            {
                let mut process_guard = self.process.lock().await;
                *process_guard = Some(child);
            }
        } else {
            // Try without authentication (key-based)
            let child = cmd.spawn()?;
            {
                let mut process_guard = self.process.lock().await;
                *process_guard = Some(child);
            }
        }

        // Wait a moment for SSH to establish
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Mark as connected
        {
            let mut connected_guard = self.is_connected.lock().await;
            *connected_guard = true;
        }

        info!("SSH tunnel established on port {}", self.config.local_port);
        info!("SOCKS5 proxy available at 127.0.0.1:{}", self.config.local_port);

        // Keep the tunnel running
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            
            // Check if process is still running
            let mut process_guard = self.process.lock().await;
            if let Some(process) = process_guard.as_mut() {
                if let Ok(Some(status)) = process.try_wait() {
                    if !status.success() {
                        error!("SSH tunnel process exited with status: {:?}", status);
                        break;
                    }
                }
            } else {
                error!("SSH tunnel process not found");
                break;
            }
        }

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        info!("Stopping SSH tunnel");
        
        {
            let mut process_guard = self.process.lock().await;
            if let Some(mut process) = process_guard.take() {
                let _ = process.kill().await;
            }
        }

        {
            let mut connected_guard = self.is_connected.lock().await;
            *connected_guard = false;
        }

        info!("SSH tunnel stopped");
        Ok(())
    }

    pub async fn is_connected(&self) -> bool {
        let connected_guard = self.is_connected.lock().await;
        *connected_guard
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SshTunnelConfig;

    #[test]
    fn test_ssh_tunnel_manager_creation() {
        let config = SshTunnelConfig {
            host: "example.com".to_string(),
            port: 22,
            username: "user".to_string(),
            password: Some("password".to_string()),
            key_path: None,
            local_port: 1080,
        };
        
        let manager = SshTunnelManager::new(config);
        assert_eq!(manager.config.host, "example.com");
    }
}