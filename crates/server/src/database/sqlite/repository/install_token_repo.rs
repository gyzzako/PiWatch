use async_trait::async_trait;
use sqlx::SqlitePool;
use crate::domain::repository::InstallTokenRepository;
use crate::domain::model::InstallToken;
use crate::domain::security::CryptoService;
use crate::error::Result;
use std::time::SystemTime;

pub(crate) struct SqliteInstallTokenRepository {
    pool: SqlitePool,
}

impl SqliteInstallTokenRepository {
    pub(crate) fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    fn now_secs() -> i64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }
}

#[async_trait]
impl InstallTokenRepository for SqliteInstallTokenRepository {
    async fn create_install_token(&self, token: &InstallToken) -> Result<()> {
        let now = Self::now_secs();
        sqlx::query(
            "INSERT INTO install_token (token_hash, salt, expires_at, created_at) VALUES (?1, ?2, ?3, ?4)"
        )
        .bind(&token.token_hash)
        .bind(&token.salt)
        .bind(token.expires_at.map(|e| e as i64))
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn validate_install_token(&self, plaintext: &str) -> Result<bool> {
        let now = Self::now_secs();
        let tokens = sqlx::query_as::<_, (String, String)>(
            "SELECT token_hash, salt FROM install_token WHERE expires_at IS NULL OR expires_at > ?1"
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await?;

        for (token_hash, salt) in tokens {
            if CryptoService::verify(plaintext, &salt, &token_hash) {
                return Ok(true);
            }
        }

        Ok(false)
    }
}
