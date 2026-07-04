mod database;
mod domain;
mod handler;
mod dto;
mod pihole;
mod config;
mod service;
mod error;
mod extractor;
mod version;

use axum::{routing::{get, post}, Router};
use std::sync::Arc;
use core_watch::logging::{info, error, warn};
use crate::{
    config::load_config, database::sqlite::SqliteDatabase, domain::{model::InstallToken, repository::AgentRepository}, handler::{
        agent::{reconcile_ip, register}, heartbeat::heartbeat, metric::{list_agents, stats},
    }, pihole::client::PiholeClient, service::AgentService,
};

const DB_PATH: &str = "piwatch.db";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = match load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return Err(e);
        }
    };

    core_watch::logging::init(&config.log_level);

    let http_client = reqwest::Client::builder()
        .user_agent(format!("PiWatch-server/{}", env!("CARGO_PKG_VERSION")))
        .build()?;

    let db = SqliteDatabase::new(DB_PATH).await?;
    self::setup_install_token(&db.agent_repository()).await;

    let agent_service = Arc::new(AgentService::new(
        Arc::new(db.agent_repository()),
        Arc::new(db.auth_repository()),
        Arc::new(PiholeClient::new(http_client, config.clone())),
    ));

    let state = crate::domain::state::AppState {
        agent_service,
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
    let monitor_handle = state.agent_service.start_monitor();

    tokio::select! {
        _ = server => {},
        _ = monitor_handle => {},
    }

    error!("A critical task stopped, shutting down PiWatch");
    std::process::exit(1);
}

async fn setup_install_token(repo: &impl AgentRepository) {
    let tokens_env = std::env::var("PIWATCH_INSTALL_TOKEN");
    let token = match tokens_env {
        Ok(env) => env.trim().to_string(),
        Err(_) => {
            let token = crate::domain::security::CryptoService::generate_secret();
            warn!("PIWATCH_INSTALL_TOKEN not set. Using randomly generated token: {}", token);
            token
        }
    };

    if !token.is_empty() {
        let salt = crate::domain::security::CryptoService::generate_salt();
        let install_token = InstallToken {
            token_hash: crate::domain::security::CryptoService::hash_with_salt(&token, &salt),
            salt,
            expires_at: None,
        };
        if let Err(e) = repo.create_install_token(&install_token).await {
            error!("Failed to create install token: {}", e);
        }
    }   
}
