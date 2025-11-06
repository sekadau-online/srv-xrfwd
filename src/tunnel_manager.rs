use anyhow::Result;
use async_std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::config::ServerConfig;

pub struct TunnelManager {
    config: Arc<ServerConfig>,
    tunnels: Arc<Mutex<HashMap<u16, TcpListener>>>,
}

impl TunnelManager {
    pub fn new(config: Arc<ServerConfig>) -> Self {
        Self {
            config,
            tunnels: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub async fn create_tunnel(&self, remote_port: u16, local_host: &str, local_port: u16) -> Result<()> {
        let mut tunnels = self.tunnels.lock().await;
        
        if tunnels.contains_key(&remote_port) {
            return Err(anyhow::anyhow!("Port {} already in use", remote_port));
        }
        
        // Create listener for remote connections
        let listener = TcpListener::bind(format!("0.0.0.0:{}", remote_port)).await?;
        log::info!("Listening for remote connections on port {}", remote_port);
        
        // Start handling connections for this tunnel
        let local_target = format!("{}:{}", local_host, local_port);
        let tunnels_ref = self.tunnels.clone();
        
        tokio::spawn(async move {
            Self::handle_tunnel_connections(listener, local_target, remote_port, tunnels_ref).await;
        });
        
        tunnels.insert(remote_port, listener);
        
        Ok(())
    }
    
    async fn handle_tunnel_connections(
        listener: TcpListener,
        local_target: String,
        remote_port: u16,
        tunnels: Arc<Mutex<HashMap<u16, TcpListener>>>,
    ) {
        log::info!("Started handling connections for tunnel {}", remote_port);
        
        while let Ok((incoming_stream, addr)) = listener.accept().await {
            log::debug!("New connection on tunnel {} from {}", remote_port, addr);
            
            let local_target = local_target.clone();
            
            tokio::spawn(async move {
                if let Err(e) = Self::forward_connection(incoming_stream, &local_target).await {
                    log::error!("Forwarding error for tunnel {}: {}", remote_port, e);
                }
            });
        }
        
        log::info!("Stopped handling connections for tunnel {}", remote_port);
        
        // Remove from active tunnels
        tunnels.lock().await.remove(&remote_port);
    }
    
    async fn forward_connection(mut incoming_stream: TcpStream, local_target: &str) -> Result<()> {
        // Connect to local target
        let mut target_stream = TcpStream::connect(local_target).await?;
        
        // Simple stream forwarding
        let (mut ri, mut wi) = incoming_stream.split();
        let (mut ro, mut wo) = target_stream.split();
        
        let client_to_target = async_std::io::copy(&mut ri, &mut wo);
        let target_to_client = async_std::io::copy(&mut ro, &mut wi);
        
        tokio::try_join!(client_to_target, target_to_client)?;
        
        log::debug!("Forwarding completed for {}", local_target);
        Ok(())
    }
    
    pub async fn list_tunnels(&self) -> Vec<u16> {
        let tunnels = self.tunnels.lock().await;
        tunnels.keys().cloned().collect()
    }
    
    pub async fn close_tunnel(&self, port: u16) -> Result<()> {
        let mut tunnels = self.tunnels.lock().await;
        tunnels.remove(&port);
        log::info!("Closed tunnel on port {}", port);
        Ok(())
    }
}
