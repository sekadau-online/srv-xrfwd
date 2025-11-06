use axum::{
    extract::State,
    response::Json,
    routing::get,
    Router,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::config::ServerConfig;
use crate::ssh_server::ClientInfo;

pub async fn start_web_interface(
    config: Arc<ServerConfig>,
    clients: Arc<RwLock<HashMap<String, ClientInfo>>>,
) -> Result<(), anyhow::Error> {
    let app = Router::new()
        .route("/", get(root))
        .route("/status", get(status))
        .route("/clients", get(list_clients))
        .route("/health", get(health))
        .with_state(AppState { config, clients });

    let listener = tokio::net::TcpListener::bind(config.get_web_bind()).await?;
    log::info!("Web interface running on http://{}", config.get_web_bind());
    
    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Clone)]
struct AppState {
    config: Arc<ServerConfig>,
    clients: Arc<RwLock<HashMap<String, ClientInfo>>>,
}

async fn root() -> &'static str {
    "SSH Forwarder Server - Status OK"
}

async fn health() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

async fn status(State(state): State<AppState>) -> Json<Value> {
    let clients = state.clients.read().await;
    
    Json(json!({
        "server_bind": state.config.get_server_bind(),
        "web_interface": state.config.get_web_bind(),
        "connected_clients": clients.len(),
        "max_connections": state.config.max_connections,
        "allowed_ports": state.config.allowed_ports,
        "allowed_users": state.config.allowed_users,
    }))
}

async fn list_clients(State(state): State<AppState>) -> Json<Value> {
    let clients = state.clients.read().await;
    let client_list: Vec<Value> = clients.values().map(|client| {
        json!({
            "username": client.username,
            "remote_addr": client.remote_addr,
            "connected_at": client.connected_at.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            "tunnels": client.tunnels.iter().map(|tunnel| {
                json!({
                    "remote_port": tunnel.remote_port,
                    "local_host": tunnel.local_host,
                    "local_port": tunnel.local_port,
                    "created_at": tunnel.created_at.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                })
            }).collect::<Vec<Value>>(),
        })
    }).collect();
    
    Json(json!(client_list))
}
