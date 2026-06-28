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
    pub install_token: String,
    pub hostname: String,
    pub agent_version: String,
}

#[derive(Deserialize, Serialize)]
pub struct RegisterResponse {
    pub agent_id: String,
    pub agent_secret: String,
}