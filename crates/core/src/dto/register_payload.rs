use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct RegisterPayload {
    pub node_id: String,
    pub hostname: String,
    pub agent_version: String,
    pub ipv4: String,
}