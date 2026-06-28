mod network;
mod api_client;
mod config;
mod heartbeat;

use std::{sync::Arc};
use crate::config::{load_config, AgentIdentity};
use crate::{api_client::ApiClient};
use crate::network::IpChangeListener;
use anyhow::Result;
use core_watch::logging::{error, info};
use crate::heartbeat::start_heartbeat;

#[tokio::main]
async fn main() -> Result<()> {
    let config = match load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to load configuration: {:#}", e);
            info!("Shutting down agent...");
            std::process::exit(1);
        }
    };
    
    core_watch::logging::init(&config.log_level);

    let identity = self::init_agent_identity(&config).await?;
    
    let client = reqwest::Client::new();
    let api = ApiClient::new(client.clone(), &config.piwatch_server_url, identity)?;
    let ip_listener = Arc::new(
        IpChangeListener::init(api.clone(), &config.listening_interface).await?
    );

    let ip = ip_listener.get_current_ip().await.map(|ip| ip.to_string());
    if let Err(e) = api.reconcile_ip(ip).await {
        error!("Failed to reconcile IP on startup: {:#}", e);
    }

    let heartbeat_handle = start_heartbeat(api.clone());
    let ip_listener_handle = ip_listener.start();
    
    info!("Node started");

    tokio::select! {
        _ = ip_listener_handle => {},
        _ = heartbeat_handle => {},
    }

    error!("A critical task stopped, shutting down agent");
    std::process::exit(1);
}

async fn init_agent_identity(config: &crate::config::Config) -> Result<AgentIdentity> {
    let identity = match AgentIdentity::load() {
        Ok(id) => id,
        Err(_) => {
            let client = reqwest::Client::new();
            let temp_api = ApiClient::new(client.clone(), &config.piwatch_server_url, AgentIdentity {
                agent_id: String::new(),
                agent_secret: String::new(),
            })?;
            match temp_api.register_agent().await {
                Ok(id) => id,
                Err(e) => {
                    error!("Failed to register agent: {:#}", e);
                    info!("Shutting down agent...");
                    std::process::exit(1);
                }
            }
        }
    };
    Ok(identity)
}