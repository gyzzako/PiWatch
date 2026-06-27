use axum::{
    extract::State,
    Json,
};
use core_watch::dto::api::{ApiResponse, DefaultApiResponse};
use core_watch::logging::warn;
use core_watch::dto::http_payloads::{Heartbeat};
use reqwest::StatusCode;
use std::time::Instant;
use crate::model::state::AppState;

pub(crate) async fn heartbeat(
    State(state): State<AppState>,
    Json(req): Json<Heartbeat>,
) -> ApiResponse {
    let Some(mut agent) = state.agents.get_mut(&req.hostname) else {
        warn!("HEARTBEAT from unknown node={}. Suggesting registration...", req.hostname);
        return ApiResponse::Error(StatusCode::NOT_FOUND, Json(DefaultApiResponse {
            success: false,
            message: Some("Agent not registered. Please register first.".to_string()),
        }));
    };

    if agent.uuid != req.uuid {
        warn!("HEARTBEAT UUID mismatch for hostname={}. Expected {} but got {}", req.hostname, agent.uuid, req.uuid);
        return ApiResponse::Error(StatusCode::FORBIDDEN, Json(DefaultApiResponse {
            success: false,
            message: Some("UUID does not match registered agent identity.".to_string()),
        }));
    }

    agent.last_seen = Instant::now();
    ApiResponse::StatusOnly(StatusCode::OK)
}