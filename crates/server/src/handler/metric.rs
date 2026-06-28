use axum::{
    extract::State,
    Json,
};
use crate::domain::state::AppState;
use crate::dto::agent_summary::AgentSummary;
use core_watch::logging::error;

pub(crate) async fn list_agents(
    State(state): State<AppState>,
) -> Json<Vec<AgentSummary>> {
    match state.agent_service.list_agent_summaries().await {
        Ok(summaries) => Json(summaries),
        Err(e) => {
            error!("Failed to list agents: {}", e);
            Json(vec![])
        }
    }
}

pub(crate) async fn stats(State(state): State<AppState>) -> Json<serde_json::Value> {
    match state.agent_service.list_agents().await {
        Ok(agents) => {
            let total = agents.len();
            let online = agents
                .iter()
                .filter(|a| a.deactivated_at.is_none())
                .count();

            Json(serde_json::json!({
                "agents_total": total,
                "agents_online": online,
                "agents_offline": total - online,
            }))
        }
        Err(_) => Json(serde_json::json!({"agents_total": 0, "agents_online": 0, "agents_offline": 0})),
    }
}
