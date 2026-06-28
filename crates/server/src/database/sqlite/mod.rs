use std::sync::Arc;
use tokio::sync::Mutex;
use rusqlite::Connection;
use crate::error::Result;

pub(crate) struct SqliteDatabase {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteDatabase {
    pub async fn new(database_path: &str) -> Result<Self> {
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
            );",
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub(crate) fn conn(&self) -> &Arc<Mutex<Connection>> {
        &self.conn
    }

    pub fn agent_repository(&self) -> SqliteAgentRepository {
        SqliteAgentRepository::new(self.conn().clone())
    }
}

mod agent_repo;

pub(crate) use agent_repo::{SqliteAgentRepository};
