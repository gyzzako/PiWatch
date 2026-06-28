use core_watch::dto::http_payloads::{IpReconciliationPayload, RegisterPayload, RegisterResponse};
use core_watch::dto::api::{ApiResponse, DefaultApiResponse};
use crate::domain::state::AppState;
use crate::extractor::AuthenticatedAgent;
use crate::error::Error;
use axum::{
    extract::State,
    Json,
    http::StatusCode,
};

pub(crate) async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterPayload>,
) -> ApiResponse<RegisterResponse> {
    match state.agent_service.register(req).await {
        Ok(resp) => ApiResponse::Success(StatusCode::CREATED, Json(resp)),
        Err(Error::Unauthorized) => ApiResponse::Error(StatusCode::UNAUTHORIZED, Json(DefaultApiResponse {
            success: false,
            message: Some("Invalid install token".to_string()),
        })),
        Err(Error::Conflict(msg)) => ApiResponse::Error(StatusCode::CONFLICT, Json(DefaultApiResponse {
            success: false,
            message: Some(msg),
        })),
        Err(_) => ApiResponse::Error(StatusCode::INTERNAL_SERVER_ERROR, Json(DefaultApiResponse {
            success: false,
            message: Some("Registration failed".to_string()),
        })),
    }
}

pub(crate) async fn reconcile_ip(
    AuthenticatedAgent(agent): AuthenticatedAgent,
    State(state): State<AppState>,
    Json(req): Json<IpReconciliationPayload>,
) -> ApiResponse<DefaultApiResponse> {
    match state.agent_service.reconcile_ip(&agent, req.ipv4).await {
        Ok(_) => ApiResponse::StatusOnly(StatusCode::OK),
        Err(Error::BadRequest(msg)) => ApiResponse::Error(StatusCode::BAD_REQUEST, Json(DefaultApiResponse {
            success: false,
            message: Some(msg),
        })),
        Err(_) => ApiResponse::Error(StatusCode::BAD_GATEWAY, Json(DefaultApiResponse {
            success: false,
            message: Some("DNS reconciliation failed".to_string()),
        })),
    }
}
