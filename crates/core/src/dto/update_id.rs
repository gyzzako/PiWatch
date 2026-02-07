use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct IpUpdatePayload {
    pub node_id: String,
    pub ipv4: Option<String>,
    pub event: String, // "add" | "del"
}