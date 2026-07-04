use std::time::SystemTime;

#[derive(Clone)]
pub(crate) struct Agent {
    pub agent_id: String,
    pub secret_hash: String,
    pub salt: String,
    pub hostname: String,
    pub agent_version: String,
    pub ipv4: Option<String>,
    pub last_seen_secs: u64,
    pub created_at: SystemTime,
    pub revoked: bool,
    pub deactivated_at: Option<SystemTime>,
}

impl Agent {
    pub fn name(&self) -> String {
        self.hostname.clone()
    }
}

#[derive(Clone)]
pub(crate) struct InstallToken {
    pub token_hash: String,
    pub salt: String,
    pub expires_at: Option<u64>,
}
