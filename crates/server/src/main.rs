mod model;
mod handler;
mod dto;
mod pihole;
mod config;

use axum::{routing::{get, post},Router};
use dashmap::DashMap;
use std::{sync::Arc,time::{Duration},};
use core::logging::{info, warn};
use crate::{
    config::load_config, handler::{
        agent::{register, update_ip},
        heart_beat::heartbeat,
        metric::{list_agents, stats},
    }, model::state::AppState
};
use pihole::client::PiholeClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = match load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            return Err(e.into());
        }
    };

    core::logging::init(&config.log_level);

    let http_client = reqwest::Client::new();

    let state = AppState {
        agents: Arc::new(DashMap::new()),
        pihole_client: Arc::new(PiholeClient::new(http_client, &config.pihole_url, &config.pihole_pass)),
    };

    // Background cleanup / logging task
    {
        let agents = state.agents.clone();
        tokio::spawn(async move {
            loop {
                for entry in agents.iter() {
                    if entry.last_seen.elapsed() > Duration::from_secs(300) {
                        warn!("Node {} has been offline for >5m", entry.key());
                    }
                }
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        });
    }

    // TODO: API versioning
    let app = Router::new()
        .route("/register", post(register))
        .route("/update", post(update_ip))
        .route("/heartbeat", post(heartbeat))
        .route("/agents", get(list_agents))
        .route("/stats", get(stats))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:".to_owned() + &config.bind_port.to_string()).await.unwrap();
    info!("PiWatch server listening on http://localhost:{}", &config.bind_port);

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
