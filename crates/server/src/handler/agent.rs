use core::dto::{register_payload::RegisterPayload, update_id::IpUpdatePayload};
use crate::AppState;
use axum::{
    extract::State,
    Json,
};
use tracing::{info, warn};
use crate::model::state::AgentState;
use std::time::{SystemTime, Instant};

pub(crate) async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterPayload>,
) {
    info!("REGISTER node_id={} hostname={}", req.node_id, req.hostname);

    state.agents.insert(
        req.node_id,
        AgentState {
            hostname: req.hostname,
            agent_version: req.agent_version,
            ipv4: req.ipv4,
            registered_at: SystemTime::now(),
            last_seen: Instant::now(),
        },
    );
}

pub(crate) async fn update_ip(
    State(state): State<AppState>,
    Json(req): Json<IpUpdatePayload>,
) {
    if let Some(mut agent) = state.agents.get_mut(&req.node_id) {
        agent.last_seen = Instant::now();
        agent.ipv4 = req.ipv4.clone();

        info!(
            "UPDATE node={} event={} ip={:?}",
            req.node_id, req.event, req.ipv4,
        );
    } else {
        warn!("UPDATE from unknown node {}", req.node_id);
    }
}
