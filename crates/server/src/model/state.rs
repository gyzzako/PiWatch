use crate::pihole::client::PiholeClient;
use std::{
    time::{Instant, SystemTime}
};
use dashmap::DashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct AppState {
    pub agents: Arc<Agents>,
    pub pihole_client: Arc<PiholeClient>,
}

pub(crate) type Agents = DashMap<String, AgentState>;

pub(crate) struct AgentState {
    pub hostname: String,
    pub uuid: Uuid,
    pub agent_version: String,
    pub ipv4: String,
    pub registered_at: SystemTime,
    pub last_seen: Instant,
}