use axum::{
    extract::State,
    Json,
};
use crate::domain::state::AppState;
use crate::dto::agent_summary::AgentSummary;
use std::time::SystemTime;

pub(crate) async fn list_agents(
    State(state): State<AppState>,
) -> Json<Vec<AgentSummary>> {
    let agents = match state.db.list_agents().await {
        Ok(agents) => agents,
        Err(e) => {
            core_watch::logging::error!("Failed to list agents: {}", e);
            return Json(vec![]);
        }
    };
    
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let summaries: Vec<AgentSummary> = agents
        .into_iter()
        .map(|agent| {
            let elapsed = now.saturating_sub(agent.last_seen_secs);
            
            AgentSummary {
                hostname: agent.name(),
                agent_version: agent.agent_version,
                ipv4: agent.ipv4.unwrap_or_default(),
                online: agent.deactivated_at == None,
                last_seen_sec: elapsed,
                registered_at: agent.created_at,
            }
        })
        .collect();

    Json(summaries)
}

pub(crate) async fn stats(State(state): State<AppState>) -> Json<serde_json::Value> {
    let agents = match state.db.list_agents().await {
        Ok(a) => a,
        Err(_) => return Json(serde_json::json!({"agents_total": 0, "agents_online": 0, "agents_offline": 0})),
    };
    
    let total = agents.len();
    let online = agents
        .iter()
        .filter(|a| a.deactivated_at == None)
        .count();

    Json(serde_json::json!({
        "agents_total": total,
        "agents_online": online,
        "agents_offline": total - online,
    }))
}