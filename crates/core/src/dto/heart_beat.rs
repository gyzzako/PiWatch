use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Heartbeat {
    pub hostname: String,
}