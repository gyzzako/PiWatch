use sqlx::SqlitePool;
use crate::error::Result;

pub(crate) struct SqliteDatabase {
    pool: SqlitePool,
}

impl SqliteDatabase {
    pub async fn new(database_path: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_path).await?;

        migration::run_migrations(&pool).await?;
        migration::verify_migrations(&pool).await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub fn agent_repository(&self) -> SqliteAgentRepository {
        SqliteAgentRepository::new(self.pool().clone())
    }
}

mod repository;
mod migration;

pub(crate) use repository::SqliteAgentRepository;
