mod model;
mod handler;
mod dto;
mod pihole;
mod config;
mod background;

use axum::{routing::{get, post},Router};
use dashmap::DashMap;
use std::{sync::Arc};
use core_watch::logging::{info, error};
use crate::{
    config::load_config, handler::{
        agent::{register, reconcile_ip},
        heartbeat::heartbeat,
        metric::{list_agents, stats},
    }, model::state::AppState
};
use pihole::client::PiholeClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = match load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return Err(e.into());
        }
    };

    core_watch::logging::init(&config.log_level);

    let http_client = reqwest::Client::builder()
        .user_agent("PiWatch-server/v0.1.0")
        .build()?;

    let state = AppState {
        agents: Arc::new(DashMap::new()),
        pihole_client: Arc::new(PiholeClient::new(http_client, config.clone())),
    };

    // TODO: API versioning
    let app = Router::new()
        .route("/register", post(register))
        .route("/reconcile", post(reconcile_ip))
        .route("/heartbeat", post(heartbeat))
        .route("/agents", get(list_agents))
        .route("/stats", get(stats))
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:".to_owned() + &config.bind_port.to_string()).await?;
    info!("PiWatch server listening on http://localhost:{}", &config.bind_port);

    let server = axum::serve(listener, app);
    let monitor_handle = background::start_agent_monitor(state.agents);

    tokio::select! {
        _ = server => {},
        _ = monitor_handle => {},
    }

    error!("A critical task stopped, shutting down PiWatch");
    std::process::exit(1);
}
