use axum::{
    extract::State,
    Json,
};
use crate::model::state::AppState;
use crate::dto::agent_summary::AgentSummary;

pub(crate) async fn list_agents(
    State(state): State<AppState>,
) -> Json<Vec<AgentSummary>> {
    let agents = state
        .agents
        .iter()
        .map(|entry| {
            let last_seen = entry.last_seen.elapsed().as_secs();
            AgentSummary {
                node_id: entry.key().clone(),
                hostname: entry.hostname.clone(),
                agent_version: entry.agent_version.clone(),
                ipv4: entry.ipv4.clone(),
                online: last_seen < 120,
                last_seen_sec: last_seen,
                registered_at: entry.registered_at,
            }
        })
        .collect();

    Json(agents)
}

pub(crate) async fn stats(State(state): State<AppState>) -> Json<serde_json::Value> {
    let total = state.agents.len();
    let online = state
        .agents
        .iter()
        .filter(|a| a.last_seen.elapsed().as_secs() < 120)
        .count();

    Json(serde_json::json!({
        "agents_total": total,
        "agents_online": online,
        "agents_offline": total - online,
    }))
}