use core::dto::{register_payload::RegisterPayload, update_id::IpUpdatePayload};
use crate::AppState;
use crate::model::state::AgentState;
use axum::{
    extract::State,
    Json,
};
use core::logging::{error, info, warn};
use std::time::{SystemTime, Instant};

// TODO: return proper response
pub(crate) async fn register(State(state): State<AppState>, Json(req): Json<RegisterPayload>) {
    if req.ipv4.is_none() {
        warn!("REGISTER received with no IPv4 for hostname {}", req.hostname);
        return;
    }
    
    let ip = req.ipv4.unwrap();
    
    if let Err(e) = state.pihole_client.put_ip(&req.hostname, &ip).await {
        error!("Failed to register IP for hostname={}: {}", req.hostname, e);
        return;
    };

    // TODO: move to DB
    state.agents.insert(
        req.hostname.to_string(),
        AgentState {
            hostname: req.hostname.to_string(),
            agent_version: req.agent_version,
            ipv4: ip,
            registered_at: SystemTime::now(),
            last_seen: Instant::now(),
        },
    );

    info!("REGISTER hostname={}", req.hostname);
}

pub(crate) async fn update_ip(State(state): State<AppState>, Json(req): Json<IpUpdatePayload>) {
    if req.ipv4.is_none() {
        warn!("UPDATE received with no IPv4 for hostname {}", req.hostname);
        return;
    }

    // TODO: handle delete
    if req.event != "add" {
        warn!("Skipping UPDATE received with event={} for hostname {}", req.event, req.hostname);
        return;
    }

    let ip = req.ipv4.unwrap();
    
    if let Err(e) = state.pihole_client.put_ip(&req.hostname, &ip).await {
        error!("Failed to register IP for hostname {}: {}", req.hostname, e);
        return;
    };

    if let Some(mut agent) = state.agents.get_mut(&req.hostname) {
        agent.last_seen = Instant::now();
        agent.ipv4 = ip.to_string();

        info!("UPDATE hostname={} event={} ip={}", req.hostname, req.event, ip);
    } else {
        warn!("UPDATE from unknown hostname {}", req.hostname);
    }
}
