use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    // Server Configuration
    pub server_host: String,
    pub server_port: u16,
    pub web_interface: String,
    pub web_port: u16,
    
    // SSH Configuration
    pub ssh_authorized_keys_path: String,
    pub ssh_server_key_path: String,
    
    // Forwarding Configuration
    pub allowed_ports: String,
    pub max_connections: u32,
    pub session_timeout: u64,
    
    // Security
    pub allowed_users: String,
    pub require_pubkey_auth: bool,
}

impl ServerConfig {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let mut cfg = config::Config::builder();
        cfg = cfg.add_source(config::Environment::default());
        let cfg = cfg.build()?;
        cfg.try_deserialize()
    }
    
    pub fn get_allowed_ports(&self) -> HashSet<u16> {
        self.allowed_ports
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect()
    }
    
    pub fn get_allowed_users(&self) -> HashSet<String> {
        self.allowed_users
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    }
    
    pub fn get_server_bind(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
    
    pub fn get_web_bind(&self) -> String {
        format!("{}:{}", self.web_interface, self.web_port)
    }
}
