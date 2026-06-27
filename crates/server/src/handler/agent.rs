use core_watch::dto::http_payloads::{IpReconciliationPayload, RegisterPayload};
use core_watch::dto::api::{ApiResponse, DefaultApiResponse};
use crate::AppState;
use crate::model::state::AgentState;
use axum::{
    extract::State,
    Json,
    http::StatusCode
};
use core_watch::logging::{error, info, warn};
use std::time::{SystemTime, Instant};

pub(crate) async fn register(State(state): State<AppState>, Json(req): Json<RegisterPayload>) -> ApiResponse {
    let hostname = req.hostname.clone();
    info!("Received REGISTER hostname={} ip={:?}", hostname, req.ipv4);

    let agent = AgentState {
        hostname: hostname.clone(),
        agent_version: req.agent_version,
        ipv4: req.ipv4.clone().unwrap_or_default(),
        registered_at: SystemTime::now(),
        last_seen: Instant::now(),
    };
    state.agents.insert(hostname.clone(), agent);

    // try to reconcile the IP address with Pi-hole if provided
    if let Some(ip) = &req.ipv4 {
        if let Err(e) = state.pihole_client.reconcile_ip_for_hostname(&hostname, ip).await {
            warn!("Registered agent but failed to reconcile hostname={} ip={}: {}", hostname, ip, e);
            return ApiResponse::StatusOnly(StatusCode::CREATED);
        }

        info!("Registered and reconciled hostname={} ip={}", hostname, ip);
    } else {
        warn!("Registered hostname={} without IPv4", hostname);
    }

    ApiResponse::StatusOnly(StatusCode::CREATED)
}

pub(crate) async fn reconcile_ip(State(state): State<AppState>, Json(req): Json<IpReconciliationPayload>) -> ApiResponse {
    let Some(ip) = req.ipv4 else {
        warn!("Received IP reconciliation with no IPv4 for hostname {}", req.hostname);
        return ApiResponse::Error(StatusCode::BAD_REQUEST, Json(DefaultApiResponse {
            success: false,
            message: Some("Missing IPv4 address".to_string()),
        }));
    };

    info!("Received IP reconciliation for hostname={} ip={}", req.hostname, ip);
    if let Err(e) = state.pihole_client.reconcile_ip_for_hostname(&req.hostname, &ip).await {
        error!("Failed to reconcile IP for hostname {}: {}", req.hostname, e);
        return ApiResponse::Error(StatusCode::BAD_GATEWAY, Json(DefaultApiResponse {
            success: false,
            message: Some("DNS reconciliation failed".to_string()),
        }));
    };

    info!("Reconciled IP for hostname={} ip={}", req.hostname, ip);
    ApiResponse::StatusOnly(StatusCode::OK)
}
