mod model;
mod handler;
mod dto;
use axum::{
    routing::{get, post},
    Router
};
use dashmap::DashMap;
use std::{
    sync::Arc,
    time::{Duration},
};
use tracing::{info, warn};
use crate::{
    handler::{
        agent::{register, update_ip},
        heart_beat::heartbeat,
        metric::list_agents,
        metric::stats,
    }, 
    model::state::AppState
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState {
        agents: Arc::new(DashMap::new()),
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

    let app = Router::new()
        .route("/register", post(register))
        .route("/update", post(update_ip))
        .route("/heartbeat", post(heartbeat))
        .route("/agents", get(list_agents))
        .route("/stats", get(stats))
        .with_state(state);

    let port = 8080;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:".to_owned() + &port.to_string()).await.unwrap();
    info!("Central server listening on http://localhost:{}", port);

    axum::serve(listener, app).await.unwrap();
}
