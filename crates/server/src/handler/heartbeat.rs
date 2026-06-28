use axum::extract::State;
use core_watch::dto::api::{ApiResponse, DefaultApiResponse};
use core_watch::logging::error;
use reqwest::StatusCode;
use crate::domain::state::AppState;
use crate::extractor::AuthenticatedAgent;

pub(crate) async fn heartbeat(
    AuthenticatedAgent(agent): AuthenticatedAgent,
    State(state): State<AppState>,
) -> ApiResponse<DefaultApiResponse> {
    if let Err(e) = state.agent_service.heartbeat(&agent.agent_id).await {
        error!("Failed to update heartbeat for agent {}: {}", agent.name(), e);
        return ApiResponse::Error(StatusCode::INTERNAL_SERVER_ERROR, axum::Json(DefaultApiResponse {
            success: false,
            message: Some(format!("Failed to update heartbeat: {}", e)),
        }));
    }

    ApiResponse::StatusOnly(StatusCode::OK)
}
