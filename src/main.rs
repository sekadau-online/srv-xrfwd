mod config;
mod ssh_server;
mod tunnel_manager;
mod web_interface;

use anyhow::Result;
use config::ServerConfig;
use ssh_server::SSHServer;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();
    
    log::info!("Starting SSH Forwarder Server...");
    
    // Load configuration
    let config = Arc::new(ServerConfig::from_env()?);
    log::debug!("Server configuration: {:?}", config);
    
    // Create and start SSH server
    let server = SSHServer::new(config.clone());
    
    // Start server
    if let Err(e) = server.start().await {
        log::error!("Server error: {}", e);
        return Err(e);
    }
    
    Ok(())
}
