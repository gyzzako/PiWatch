use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};
use core_watch::{dto::api::{ApiResponse, DefaultApiResponse}};
use core_watch::logging::{warn, debug};
use crate::domain::state::AppState;

pub(crate) struct AuthenticatedAgent(pub(crate) crate::domain::model::Agent);

impl<S> FromRequestParts<S> for AuthenticatedAgent
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = ApiResponse<DefaultApiResponse>;

    fn from_request_parts(parts: &mut Parts, state: &S) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            let app_state = AppState::from_ref(state);

            let agent_id = get_agent_id_header(parts)?;
            let agent_version = get_agent_version_header(parts);
            let agent_secret = get_agent_secret_header(parts, agent_id)?;

            let agent = match app_state.agent_service.authenticate(agent_id, agent_secret).await {
                Ok(a) => a,
                Err(_) => {
                    warn!("Authentication failed for agent_id={}", agent_id);
                    return Err(ApiResponse::Error(StatusCode::UNAUTHORIZED, axum::Json(DefaultApiResponse {
                        success: false,
                        message: Some("Invalid agent credentials".to_string()),
                    })));
                }
            };

            let agent = if let Some(claimed_version) = agent_version {
                if claimed_version != agent.agent_version {
                    if let Err(e) = app_state.agent_service.update_agent_version(&agent.agent_id, &claimed_version).await {
                        warn!("Failed to update version for agent {}: {}", agent.name(), e);
                    }
                }
                crate::domain::model::Agent { agent_version: claimed_version, ..agent }
            } else {
                agent
            };

            if let Err(e) = crate::version::check_compatible(&agent.agent_version, env!("CARGO_PKG_VERSION"))
            {
                warn!("Agent {} has incompatible version {} (server requires {})", agent.name(), agent.agent_version, env!("CARGO_PKG_VERSION"));
                return Err(ApiResponse::Error(StatusCode::UPGRADE_REQUIRED, axum::Json(DefaultApiResponse {
                    success: false,
                    message: Some(format!("Version mismatch: {}", e)),
                })));
            }

            if agent.deactivated_at.is_some() && let Err(e) = app_state.agent_service.heartbeat(&agent.agent_id).await {
                warn!("Failed to update agent={} last seen: {}", agent.name(), e);
            }

            Ok(AuthenticatedAgent(agent))
        }
    }
}

fn get_agent_id_header(parts: &Parts) -> Result<&str, ApiResponse<DefaultApiResponse>> {
    parts.headers.get("X-Agent-Id")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            debug!("Auth attempt: missing X-Agent-Id header");
            ApiResponse::Error(StatusCode::UNAUTHORIZED, axum::Json(DefaultApiResponse {
                success: false,
                message: Some("Missing X-Agent-Id header".to_string()),
            }))
        })
}

fn get_agent_version_header(parts: &Parts) -> Option<String> {
    parts.headers.get("X-Agent-Version")
        .and_then(|v| v.to_str().ok())
        .filter(|v| !v.is_empty())
        .map(|v| v.to_string())
}

fn get_agent_secret_header<'p>(parts: &'p Parts, agent_id: &str) -> Result<&'p str, ApiResponse<DefaultApiResponse>> {
    parts.headers.get("X-Agent-Secret")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            debug!("Auth attempt: missing X-Agent-Secret header for agent_id={}", agent_id);
            ApiResponse::Error(StatusCode::UNAUTHORIZED, axum::Json(DefaultApiResponse {
                success: false,
                message: Some("Missing X-Agent-Secret header".to_string()),
            }))
        })
}