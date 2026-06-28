mod domain;
mod handler;
mod dto;
mod pihole;
mod config;
mod background;
mod extractor;

use axum::{routing::{get, post},Router};
use std::sync::Arc;
use core_watch::logging::{info, error, warn};
use crate::{
    config::load_config, handler::{
        agent::{register, reconcile_ip},
        heartbeat::heartbeat,
        metric::{list_agents, stats},
    }, domain::state::{AppState, InstallToken}
};
use crate::domain::db::DbStore;
use pihole::client::PiholeClient;

const DB_PATH: &'static str = "piwatch.db";

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
        .user_agent(format!("PiWatch-server/v{}", env!("CARGO_PKG_VERSION")))
        .build()?;
    
    let db = DbStore::new(DB_PATH).await?;
    
    self::setup_install_token(&db).await;

    let state = AppState {
        db: Arc::new(db),
        pihole_client: Arc::new(PiholeClient::new(http_client, config.clone())),
    };

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
    let monitor_handle = background::start_agent_monitor(state.db.clone());

    tokio::select! {
        _ = server => {},
        _ = monitor_handle => {},
    }

    error!("A critical task stopped, shutting down PiWatch");
    std::process::exit(1);
}

async fn setup_install_token(db: &DbStore)  {
    let tokens_env = std::env::var("PIWATCH_INSTALL_TOKEN");
    let token = match tokens_env {
        Ok(env) => env.trim().to_string(),
        Err(_) => {
            let token = crate::domain::security::generate_secret();
            warn!("PIWATCH_INSTALL_TOKEN not set. Using randomly generated token: {}", token);
            token
        }
    };

    if !token.is_empty() {
        let salt = crate::domain::security::generate_salt();
        let install_token = InstallToken {
            token_hash: crate::domain::security::hash_with_salt(&token, &salt),
            salt,
        };
        if let Err(e) = db.create_install_token(&install_token).await {
            error!("Failed to create install token: {}", e);
        }
    }   
}