use axum::{
    extract::State,
    Json,
};
use core::logging::warn;
use core::dto::heart_beat::Heartbeat;
use std::time::Instant;
use crate::model::state::AppState;

pub(crate) async fn heartbeat(
    State(state): State<AppState>,
    Json(req): Json<Heartbeat>,
) {
    if let Some(mut agent) = state.agents.get_mut(&req.hostname) {
        agent.last_seen = Instant::now();
    } else {
        warn!("HEARTBEAT from unknown node {}", req.hostname);
    }
}