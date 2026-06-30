use thiserror::Error;
use sqlx::Error as SqlxError;

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("Database error: {0}")]
    Database(#[from] SqlxError),

    #[allow(dead_code)]
    #[error("Not found")]
    NotFound,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Version mismatch: {0}")]
    VersionMismatch(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Conflict: {0}")]
    Conflict(String),
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
