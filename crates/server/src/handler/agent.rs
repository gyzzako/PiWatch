use core_watch::dto::http_payloads::{IpReconciliationPayload, RegisterPayload, RegisterResponse};
use core_watch::dto::api::{ApiResponse, DefaultApiResponse};
use crate::AppState;
use crate::domain::security::{generate_secret, generate_salt, hash_with_salt};
use crate::extractor::AuthenticatedAgent;
use axum::{
    extract::State,
    Json,
    http::StatusCode
};
use core_watch::logging::{error, info, warn, debug};
use uuid::Uuid;

pub(crate) async fn register(
    State(state): State<AppState>, 
    Json(req): Json<RegisterPayload>
) -> ApiResponse<RegisterResponse> {
    let valid_token = state.db.validate_install_token(&req.install_token).await.unwrap_or(false);

    if !valid_token {
        warn!("Registration rejected: invalid install token for hostname={}", req.hostname);
        return ApiResponse::Error(StatusCode::UNAUTHORIZED, Json(DefaultApiResponse {
            success: false,
            message: Some("Invalid install token".to_string()),
        }));
    }

    let existing = state.db.get_agent_by_hostname(&req.hostname).await;
    if let Ok(Some(a)) = existing {
        warn!("Registration rejected: hostname {} already registered as agent_id={}", req.hostname, a.agent_id);
        return ApiResponse::Error(StatusCode::CONFLICT, Json(DefaultApiResponse {
            success: false,
            message: Some("Hostname already registered".to_string()),
        }));
    }

    let agent_id = Uuid::new_v4().to_string();
    let agent_secret = generate_secret();
    let salt = generate_salt();
    let secret_hash = hash_with_salt(&agent_secret, &salt);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let agent = crate::domain::state::Agent {
        agent_id: agent_id.clone(),
        secret_hash,
        salt,
        hostname: Some(req.hostname.clone()),
        agent_version: req.agent_version,
        ipv4: None,
        last_seen_secs: now,
        created_at: std::time::SystemTime::now(),
        revoked: false,
        deactivated_at: None,
    };

    if let Err(e) = state.db.create_agent(&agent).await {
        error!("Failed to create agent: {}", e);
        return ApiResponse::Error(StatusCode::INTERNAL_SERVER_ERROR, Json(DefaultApiResponse {
            success: false,
            message: Some("Registration failed".to_string()),
        }));
    }

    debug!("Agent {} registered successfully", agent.name());

    ApiResponse::Success(StatusCode::CREATED, Json(RegisterResponse {
        agent_id,
        agent_secret,
    }))
}

pub(crate) async fn reconcile_ip(
    AuthenticatedAgent(agent): AuthenticatedAgent,
    State(state): State<AppState>, 
    Json(req): Json<IpReconciliationPayload>
) -> ApiResponse<DefaultApiResponse> {
    let Some(ip) = req.ipv4 else {
        warn!("Received IP reconciliation with no IPv4 for agent={}", agent.name());
        return ApiResponse::Error(StatusCode::BAD_REQUEST, Json(DefaultApiResponse {
            success: false,
            message: Some("Missing IPv4 address".to_string()),
        }));
    };

    info!("Received IP reconciliation for agent={} ip={}", agent.name(), ip);
    
    if let Err(e) = state.pihole_client.reconcile_ip_for_hostname(
        agent.hostname.as_deref().unwrap_or("unknown"), 
        &ip
    ).await {
        error!("Failed to reconcile IP for agent {}: {}", agent.name(), e);
        return ApiResponse::Error(StatusCode::BAD_GATEWAY, Json(DefaultApiResponse {
            success: false,
            message: Some("DNS reconciliation failed".to_string()),
        }));
    };

    if let Err(e) = state.db.update_agent_ip(&agent.agent_id, &ip).await {
        error!("Failed to update agent IP: {}", e);
    }
    
    info!("Reconciled IP for agent={} ip={}", agent.name(), ip);
    ApiResponse::StatusOnly(StatusCode::OK)
}