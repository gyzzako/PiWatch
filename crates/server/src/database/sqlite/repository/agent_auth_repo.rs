use async_trait::async_trait;
use sqlx::SqlitePool;
use crate::{domain::repository::AgentAuthRepository, domain::model::AgentAuth, error::Result};

pub(crate) struct SqliteAgentAuthRepository {
    pool: SqlitePool,
}

impl SqliteAgentAuthRepository {
    pub(crate) fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AgentAuthRepository for SqliteAgentAuthRepository {
    async fn create_agent_auth(&self, auth: &AgentAuth) -> Result<()> {
        sqlx::query(
            "INSERT INTO agent_auth (agent_auth_id, fk_agent_id, secret_hash, salt) VALUES (?1, ?2, ?3, ?4)"
        )
        .bind(&auth.agent_auth_id)
        .bind(&auth.agent_id)
        .bind(&auth.secret_hash)
        .bind(&auth.salt)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_agent_auth(&self, agent_id: &str) -> Result<Option<AgentAuth>> {
        let row = sqlx::query_as::<_, (String, String, String, String)>(
            "SELECT agent_auth_id, fk_agent_id, secret_hash, salt FROM agent_auth WHERE fk_agent_id = ?"
        )
        .bind(agent_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(id, agent_id, hash, salt)| AgentAuth {
            agent_auth_id: id,
            agent_id,
            secret_hash: hash,
            salt,
        }))
    }
}
