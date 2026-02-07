use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct IpUpdatePayload {
    pub hostname: String,
    pub ipv4: Option<String>,
    pub event: String, // "add" | "del"
}