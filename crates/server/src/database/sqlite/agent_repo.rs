use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::Mutex;
use rusqlite::{params, Connection};
use crate::domain::security::CryptoService;
use crate::{domain::repository::AgentRepository, error::Result, InstallToken};
use crate::domain::model::Agent;
use std::time::{Duration, SystemTime};

pub(crate) struct SqliteAgentRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteAgentRepository {
    pub(crate) fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    fn now_secs() -> i64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }
}

#[async_trait]
impl AgentRepository for SqliteAgentRepository {
    async fn create_agent(&self, agent: &Agent) -> Result<()> {
        let now = Self::now_secs();
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT OR REPLACE INTO agents (agent_id, secret_hash, salt, hostname, agent_version, ipv4, last_seen, created_at, revoked)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![agent.agent_id, agent.secret_hash, agent.salt, agent.hostname, agent.agent_version, agent.ipv4, now, now, agent.revoked],
        )?;
        Ok(())
    }

    async fn get_agent(&self, agent_id: &str) -> Result<Option<Agent>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT agent_id, secret_hash, salt, hostname, agent_version, ipv4, last_seen, created_at, revoked, deactivated_at FROM agents WHERE agent_id = ?1",
        )?;

        let result = stmt.query_row(params![agent_id], |row| {
            Ok(Agent {
                agent_id: row.get(0)?,
                secret_hash: row.get(1)?,
                salt: row.get(2)?,
                hostname: row.get(3)?,
                agent_version: row.get(4)?,
                ipv4: row.get(5)?,
                last_seen_secs: row.get(6)?,
                created_at: SystemTime::UNIX_EPOCH + Duration::from_secs(row.get(7)?),
                revoked: row.get(8)?,
                deactivated_at: row.get::<_, Option<u64>>(9)?
                    .map(|secs| SystemTime::UNIX_EPOCH + Duration::from_secs(secs)),
            })
        });

        Ok(result.ok())
    }

    async fn get_agent_by_hostname(&self, hostname: &str) -> Result<Option<Agent>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT agent_id, secret_hash, salt, hostname, agent_version, ipv4, last_seen, created_at, revoked, deactivated_at FROM agents WHERE hostname = ?1",
        )?;

        let result = stmt.query_row(params![hostname], |row| {
            Ok(Agent {
                agent_id: row.get(0)?,
                secret_hash: row.get(1)?,
                salt: row.get(2)?,
                hostname: row.get(3)?,
                agent_version: row.get(4)?,
                ipv4: row.get(5)?,
                last_seen_secs: row.get(6)?,
                created_at: SystemTime::UNIX_EPOCH + Duration::from_secs(row.get(7)?),
                revoked: row.get(8)?,
                deactivated_at: row.get::<_, Option<u64>>(9)?
                    .map(|secs| SystemTime::UNIX_EPOCH + Duration::from_secs(secs)),
            })
        });

        Ok(result.ok())
    }

    async fn update_agent_last_seen(&self, agent_id: &str) -> Result<()> {
        let now = Self::now_secs();
        let conn = self.conn.lock().await;
        conn.execute("UPDATE agents SET last_seen = ?1, deactivated_at = NULL WHERE agent_id = ?2", params![now, agent_id])?;
        Ok(())
    }

    async fn update_agent_ip(&self, agent_id: &str, ipv4: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute("UPDATE agents SET ipv4 = ?1 WHERE agent_id = ?2", params![ipv4, agent_id])?;
        Ok(())
    }

    #[allow(dead_code)]
    async fn revoke_agent(&self, agent_id: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute("UPDATE agents SET revoked = 1 WHERE agent_id = ?1", params![agent_id])?;
        Ok(())
    }

    #[allow(dead_code)]
    async fn delete_agent(&self, agent_id: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute("DELETE FROM agents WHERE agent_id = ?1", params![agent_id])?;
        Ok(())
    }

    async fn set_agent_deactivated(&self, agent_id: &str) -> Result<()> {
        let now = Self::now_secs();
        let conn = self.conn.lock().await;
        conn.execute(
            "UPDATE agents SET deactivated_at = ?1 WHERE agent_id = ?2 AND deactivated_at IS NULL",
            params![now, agent_id],
        )?;
        Ok(())
    }

    async fn list_agents(&self) -> Result<Vec<Agent>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT agent_id, secret_hash, salt, hostname, agent_version, ipv4, last_seen, created_at, revoked, deactivated_at FROM agents",
        )?;

        let agents = stmt
            .query_map([], |row| {
                Ok(Agent {
                    agent_id: row.get(0)?,
                    secret_hash: row.get(1)?,
                    salt: row.get(2)?,
                    hostname: row.get(3)?,
                    agent_version: row.get(4)?,
                    ipv4: row.get(5)?,
                    last_seen_secs: row.get(6)?,
                    created_at: SystemTime::UNIX_EPOCH + Duration::from_secs(row.get(7)?),
                    revoked: row.get(8)?,
                    deactivated_at: row.get::<_, Option<u64>>(9)?
                        .map(|secs| SystemTime::UNIX_EPOCH + Duration::from_secs(secs)),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(agents)
    }

    async fn list_agents_with_last_seen(&self) -> Result<Vec<(String, u64)>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare("SELECT agent_id, last_seen FROM agents WHERE revoked = 0 and deactivated_at IS NULL")?;

        let agents = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(agents)
    }

    async fn create_install_token(&self, token: &InstallToken) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT OR IGNORE INTO install_tokens (token_hash, salt) VALUES (?1, ?2)",
            params![token.token_hash, token.salt],
        )?;
        Ok(())
    }

    async fn validate_install_token(&self, plaintext: &str) -> Result<bool> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT token_hash, salt FROM install_tokens",
        )?;
        let tokens = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<std::result::Result<Vec<(String, String)>, _>>()?;

        for (token_hash, salt) in &tokens {
            if CryptoService::verify(plaintext, salt, token_hash) {
                return Ok(true);
            }
        }

        Ok(false)
    }
}