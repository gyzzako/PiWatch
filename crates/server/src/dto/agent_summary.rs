use serde::{Serialize};
use std::{
    time::{SystemTime},
};

#[derive(Serialize)]
pub(crate) struct AgentSummary {
    pub node_id: String,
    pub hostname: String,
    pub agent_version: String,
    pub ipv4: String,
    pub online: bool,
    pub registered_at: SystemTime,
    pub last_seen_sec: u64,
}