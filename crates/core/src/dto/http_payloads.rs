use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct Heartbeat {
    pub hostname: String,
    pub uuid: Uuid,
}

#[derive(Deserialize, Serialize)]
pub struct IpReconciliationPayload {
    pub hostname: String,
    pub uuid: Uuid,
    pub ipv4: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct RegisterPayload {
    pub hostname: String,
    pub agent_version: String,
    pub ipv4: Option<String>,
    pub uuid: Uuid,
}