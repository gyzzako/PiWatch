use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Heartbeat {
    pub node_id: String,
}