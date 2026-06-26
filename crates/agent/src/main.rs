mod network;
mod api_client;
mod config;
mod heartbeat;

use std::{sync::Arc};
use crate::config::load_config;
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
            error!("Failed to load configuration: {}", e);
            return Err(anyhow::anyhow!(e.to_string()));
        }
    };
    
    core_watch::logging::init(&config.log_level);

    let client = reqwest::Client::new();
    let api = ApiClient::new(client.clone(), &config.piwatch_server_url)?;
    let ip_listener = Arc::new(
        IpChangeListener::init(api.clone(), &config.listening_interface).await?
    );

    let ip = ip_listener.get_current_ip().await.map(|ip| ip.to_string());

    match api.register_agent(ip).await {
        Ok(_) => info!("Successfully registered agent."),
        Err(e) => {
            error!("Failed to register agent: {}", e);
            return Err(e.into());
        }
    };

    let heartbeat_handle = start_heartbeat(api.clone(), ip_listener.clone());
    let ip_listener_handle = ip_listener.start();
    
    info!("Node started");

    tokio::select! {
        _ = ip_listener_handle => {},
        _ = heartbeat_handle => {},
    }

    error!("A critical task stopped, shutting down agent");
    std::process::exit(1);
}
