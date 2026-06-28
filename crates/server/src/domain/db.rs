use crate::domain::state::{Agent, InstallToken};
use rusqlite::{params, Result, Connection};
use std::time::{Duration, SystemTime};
use std::sync::Arc;
use tokio::sync::Mutex;

pub(crate) struct DbStore {
    conn: Arc<Mutex<Connection>>,
}

impl DbStore {
    pub(crate) async fn new(database_path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(database_path)?;
        
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS agents (
                agent_id TEXT PRIMARY KEY NOT NULL,
                secret_hash TEXT NOT NULL,
                salt TEXT NOT NULL,
                hostname TEXT,
                agent_version TEXT,
                ipv4 TEXT,
                last_seen INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                revoked INTEGER NOT NULL DEFAULT 0,
                deactivated_at INTEGER
            );
            CREATE TABLE IF NOT EXISTS install_tokens (
                token_hash TEXT PRIMARY KEY NOT NULL,
                salt TEXT NOT NULL DEFAULT ''
            );"
        )?;

        Ok(Self { 
            conn: Arc::new(Mutex::new(conn)) 
        })
    }
    
    fn now_secs() -> i64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }
    
    pub(crate) async fn create_agent(&self, agent: &Agent) -> Result<(), rusqlite::Error> {
        let now = Self::now_secs();
        
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT OR REPLACE INTO agents (agent_id, secret_hash, salt, hostname, agent_version, ipv4, last_seen, created_at, revoked)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![agent.agent_id, agent.secret_hash, agent.salt, agent.hostname, agent.agent_version, agent.ipv4, now, now, agent.revoked]
        )?;
        
        Ok(())
    }
    
    pub(crate) async fn get_agent(&self, agent_id: &str) -> Result<Option<Agent>, rusqlite::Error> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT agent_id, secret_hash, salt, hostname, agent_version, ipv4, last_seen, created_at, revoked, deactivated_at FROM agents WHERE agent_id = ?1"
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

    pub(crate) async fn get_agent_by_hostname(&self, hostname: &str) -> Result<Option<Agent>, rusqlite::Error> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT agent_id, secret_hash, salt, hostname, agent_version, ipv4, last_seen, created_at, revoked, deactivated_at FROM agents WHERE hostname = ?1"
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
    
    pub(crate) async fn update_agent_last_seen(&self, agent_id: &str) -> Result<(), rusqlite::Error> {
        let now = Self::now_secs();
        let conn = self.conn.lock().await;
        conn.execute("UPDATE agents SET last_seen = ?1, deactivated_at = NULL WHERE agent_id = ?2", params![now, agent_id])?;
        Ok(())
    }
    
    pub(crate) async fn update_agent_ip(&self, agent_id: &str, ipv4: &str) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().await;
        conn.execute("UPDATE agents SET ipv4 = ?1 WHERE agent_id = ?2", params![ipv4, agent_id])?;
        Ok(())
    }
    
    pub(crate) async fn revoke_agent(&self, agent_id: &str) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().await;
        conn.execute("UPDATE agents SET revoked = 1 WHERE agent_id = ?1", params![agent_id])?;
        Ok(())
    }
    
    pub(crate) async fn delete_agent(&self, agent_id: &str) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().await;
        conn.execute("DELETE FROM agents WHERE agent_id = ?1", params![agent_id])?;
        Ok(())
    }
    
    pub(crate) async fn create_install_token(&self, token: &InstallToken) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT OR IGNORE INTO install_tokens (token_hash, salt) VALUES (?1, ?2)",
            params![token.token_hash, token.salt]
        )?;
        Ok(())
    }
    
    pub(crate) async fn validate_install_token(&self, plaintext: &str) -> Result<bool, rusqlite::Error> {
        let tokens: Vec<(String, String)> = {
            let conn = self.conn.lock().await;
            let mut stmt = conn.prepare(
                "SELECT token_hash, salt FROM install_tokens"
            )?;
            stmt.query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?.collect::<Result<Vec<_>, _>>()?
        };

        for (token_hash, salt) in &tokens {
            if crate::domain::security::verify(plaintext, salt, token_hash) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub(crate) async fn set_agent_deactivated(&self, agent_id: &str) -> Result<(), rusqlite::Error> {
        let now = Self::now_secs();
        let conn = self.conn.lock().await;
        conn.execute(
            "UPDATE agents SET deactivated_at = ?1 WHERE agent_id = ?2 AND deactivated_at IS NULL",
            params![now, agent_id]
        )?;
        Ok(())
    }

    pub(crate) async fn list_agents(&self) -> Result<Vec<Agent>, rusqlite::Error> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT agent_id, secret_hash, salt, hostname, agent_version, ipv4, last_seen, created_at, revoked, deactivated_at FROM agents"
        )?;
        
        let agents = stmt.query_map([], |row| {
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
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(agents)
    }
    
    pub(crate) async fn list_agents_with_last_seen(&self) -> Result<Vec<(String, u64)>, rusqlite::Error> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare("SELECT agent_id, last_seen FROM agents WHERE revoked = 0 and deactivated_at IS NULL")?;
        
        let agents = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(agents)
    }
}