use serde::{Deserialize};

#[derive(Deserialize)]
pub(crate) struct AuthResponse {
    pub(crate) session: AuthSession,
}

#[derive(Deserialize)]
pub (crate) struct AuthSession {
    pub(crate) valid: bool,
    pub(crate) sid: Option<String>,
}