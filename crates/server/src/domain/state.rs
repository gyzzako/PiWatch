use crate::pihole::client::PiholeClient;
use std::sync::Arc;
use std::time::SystemTime;

#[derive(Clone)]
pub(crate) struct AppState {
    pub db: Arc<crate::domain::db::DbStore>,
    pub pihole_client: Arc<PiholeClient>,
}

#[derive(Clone)]
pub(crate) struct Agent {
    pub agent_id: String,
    pub secret_hash: String,
    pub salt: String,
    pub hostname: Option<String>,
    pub agent_version: String,
    pub ipv4: Option<String>,
    pub last_seen_secs: u64,
    pub created_at: SystemTime,
    pub revoked: bool,
    pub deactivated_at: Option<SystemTime>,
}

impl Agent {
    pub fn name(&self) -> String {
        self.hostname.clone().unwrap_or_else(|| self.agent_id.clone())
    }
}

#[derive(Clone)]
pub(crate) struct InstallToken {
    pub token_hash: String,
    pub salt: String,
}