use serde::{Deserialize};

#[derive(Deserialize)]
pub(crate) struct AuthResponse {
    pub(crate) session: AuthSession,
}

#[derive(Deserialize)]
pub (crate) struct AuthSession {
    pub(crate) sid: Option<String>,
}