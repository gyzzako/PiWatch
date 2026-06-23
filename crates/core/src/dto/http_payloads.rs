use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Heartbeat {
    pub hostname: String,
}

#[derive(Deserialize, Serialize)]
pub struct IpReconciliationPayload {
    pub hostname: String,
    pub ipv4: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct RegisterPayload {
    pub hostname: String,
    pub agent_version: String,
    pub ipv4: Option<String>,
}