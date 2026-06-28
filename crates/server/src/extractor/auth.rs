use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};
use core_watch::{dto::api::{ApiResponse, DefaultApiResponse}};
use core_watch::logging::{warn, debug};
use crate::domain::security::CryptoService;
use crate::domain::state::AppState;
use crate::domain::model::Agent;

pub(crate) struct AuthenticatedAgent(pub(crate) Agent);

impl<S> FromRequestParts<S> for AuthenticatedAgent
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = ApiResponse<DefaultApiResponse>;

    fn from_request_parts(parts: &mut Parts, state: &S) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            let app_state = AppState::from_ref(state);

            let agent_id = parts.headers.get("X-Agent-Id")
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| {
                    debug!("Auth attempt: missing X-Agent-Id header");
                    ApiResponse::Error(StatusCode::UNAUTHORIZED, axum::Json(DefaultApiResponse {
                        success: false,
                        message: Some("Missing X-Agent-Id header".to_string()),
                    }))
                })?;

            let agent_secret = parts.headers.get("X-Agent-Secret")
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| {
                    debug!("Auth attempt: missing X-Agent-Secret header for agent_id={}", agent_id);
                    ApiResponse::Error(StatusCode::UNAUTHORIZED, axum::Json(DefaultApiResponse {
                        success: false,
                        message: Some("Missing X-Agent-Secret header".to_string()),
                    }))
                })?;

            let agent = match app_state.agent_service.get_agent(agent_id).await {
                Ok(Some(a)) => a,
                Ok(None) => {
                    warn!("Authentication failed: unknown agent_id={}", agent_id);
                    return Err(ApiResponse::Error(StatusCode::UNAUTHORIZED, axum::Json(DefaultApiResponse {
                        success: false,
                        message: Some("Unknown agent".to_string()),
                    })));
                }
                Err(e) => {
                    warn!("Database error during auth for agent_id={}: {}", agent_id, e);
                    return Err(ApiResponse::Error(StatusCode::INTERNAL_SERVER_ERROR, axum::Json(DefaultApiResponse {
                        success: false,
                        message: Some("Internal error".to_string()),
                    })));
                }
            };

            if agent.revoked {
                warn!("Authentication failed: agent {} is revoked", agent.name());
                return Err(ApiResponse::Error(StatusCode::UNAUTHORIZED, axum::Json(DefaultApiResponse {
                    success: false,
                    message: Some("Agent is revoked".to_string()),
                })));
            }

            let valid = CryptoService::verify(agent_secret, &agent.salt, &agent.secret_hash);
            if !valid {
                warn!("Authentication failed: invalid secret for agent={}", agent.name());
                return Err(ApiResponse::Error(StatusCode::UNAUTHORIZED, axum::Json(DefaultApiResponse {
                    success: false,
                    message: Some("Invalid agent secret".to_string()),
                })));
            }

            if let Err(e) = app_state.agent_service.heartbeat(&agent.agent_id).await {
                warn!("Failed to update agent={} last seen: {}", agent.name(), e);
            }

            Ok(AuthenticatedAgent(agent))
        }
    }
}
