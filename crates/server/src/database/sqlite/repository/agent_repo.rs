use async_trait::async_trait;
use sqlx::SqlitePool;
use crate::domain::repository::AgentRepository;
use crate::{domain::model::Agent, error::Result, InstallToken};
use crate::domain::security::CryptoService;
use std::time::{Duration, SystemTime};

pub(crate) struct SqliteAgentRepository {
    pool: SqlitePool,
}

impl SqliteAgentRepository {
    pub(crate) fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    fn now_secs() -> i64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    fn map_row(
        agent_id: String,
        hostname: String,
        agent_version: String,
        ipv4: Option<String>,
        last_seen: i64,
        created_at: i64,
        revoked: i64,
        deactivated_at: Option<i64>,
    ) -> Agent {
        Agent {
            agent_id,
            hostname,
            agent_version,
            ipv4,
            last_seen_secs: last_seen as u64,
            created_at: SystemTime::UNIX_EPOCH + Duration::from_secs(created_at as u64),
            revoked: revoked != 0,
            deactivated_at: deactivated_at.map(|d| SystemTime::UNIX_EPOCH + Duration::from_secs(d as u64)),
        }
    }
}

#[async_trait]
impl AgentRepository for SqliteAgentRepository {
    async fn create_agent(&self, agent: &Agent) -> Result<()> {
        let now: i64 = Self::now_secs();
        sqlx::query(
            "INSERT OR REPLACE INTO agent (agent_id, hostname, agent_version, ipv4, last_seen, created_at, revoked)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
        )
        .bind(&agent.agent_id)
        .bind(&agent.hostname)
        .bind(&agent.agent_version)
        .bind(&agent.ipv4)
        .bind(now)
        .bind(now)
        .bind(agent.revoked as i64)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_agent(&self, agent_id: &str) -> Result<Option<Agent>> {
        let row = sqlx::query_as::<_, (String, String, String, Option<String>, i64, i64, i64, Option<i64>)>(
            "SELECT agent_id, hostname, agent_version, ipv4, last_seen, created_at, revoked, deactivated_at FROM agent WHERE agent_id = ?"
        )
        .bind(agent_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(a, b, c, d, e, f, g, h)| Self::map_row(a, b, c, d, e, f, g, h)))
    }

    async fn get_agent_by_hostname(&self, hostname: &str) -> Result<Option<Agent>> {
        let row = sqlx::query_as::<_, (String, String, String, Option<String>, i64, i64, i64, Option<i64>)>(
            "SELECT agent_id, hostname, agent_version, ipv4, last_seen, created_at, revoked, deactivated_at FROM agent WHERE hostname = ?"
        )
        .bind(hostname)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(a, b, c, d, e, f, g, h)| Self::map_row(a, b, c, d, e, f, g, h)))
    }

    async fn update_agent_last_seen(&self, agent_id: &str) -> Result<()> {
        let now = Self::now_secs();
        sqlx::query("UPDATE agent SET last_seen = ?1, deactivated_at = NULL WHERE agent_id = ?2")
            .bind(now)
            .bind(agent_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_agent_ip(&self, agent_id: &str, ipv4: &str) -> Result<()> {
        sqlx::query("UPDATE agent SET ipv4 = ?1 WHERE agent_id = ?2")
            .bind(ipv4)
            .bind(agent_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_agent_version(&self, agent_id: &str, version: &str) -> Result<()> {
        sqlx::query("UPDATE agent SET agent_version = ?1 WHERE agent_id = ?2")
            .bind(version)
            .bind(agent_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    #[allow(dead_code)]
    async fn revoke_agent(&self, agent_id: &str) -> Result<()> {
        sqlx::query("UPDATE agent SET revoked = 1 WHERE agent_id = ?1")
            .bind(agent_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    #[allow(dead_code)]
    async fn delete_agent(&self, agent_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM agent WHERE agent_id = ?1")
            .bind(agent_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn set_agent_deactivated(&self, agent_id: &str) -> Result<()> {
        let now = Self::now_secs();
        sqlx::query(
            "UPDATE agent SET deactivated_at = ?1 WHERE agent_id = ?2 AND deactivated_at IS NULL",
        )
        .bind(now)
        .bind(agent_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list_agents(&self) -> Result<Vec<Agent>> {
        let rows = sqlx::query_as::<_, (String, String, String, Option<String>, i64, i64, i64, Option<i64>)>(
            "SELECT agent_id, hostname, agent_version, ipv4, last_seen, created_at, revoked, deactivated_at FROM agent"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(a, b, c, d, e, f, g, h)| Self::map_row(a, b, c, d, e, f, g, h)).collect())
    }

    async fn list_agents_with_last_seen(&self) -> Result<Vec<(String, u64)>> {
        let rows = sqlx::query_as::<_, (String, i64)>(
            "SELECT agent_id, last_seen FROM agent WHERE revoked = 0 and deactivated_at IS NULL"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(id, ts)| (id, ts as u64)).collect())
    }

    async fn create_install_token(&self, token: &InstallToken) -> Result<()> {
        let now = Self::now_secs();
        sqlx::query(
            "INSERT OR IGNORE INTO install_token (token_hash, salt, expires_at, created_at) VALUES (?1, ?2, ?3, ?4)"
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
