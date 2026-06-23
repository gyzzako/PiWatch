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
    if let Some(mut agent) = state.agents.get_mut(&req.hostname) {
        agent.last_seen = Instant::now();
        return ApiResponse::StatusOnly(StatusCode::OK)
    }

    warn!("HEARTBEAT from unknown node={}. Suggesting registration...", req.hostname);

    ApiResponse::Error(StatusCode::NOT_FOUND, Json(DefaultApiResponse {
        success: false,
        message: Some("Agent not registered. Please register first.".to_string()),
    }))
}