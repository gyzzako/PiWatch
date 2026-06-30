use sqlx::migrate::Migrator;
use crate::error::Result;
use core_watch::logging::{info, warn};

static MIGRATOR: Migrator = sqlx::migrate!("resources/migrations/sqlite");

pub(crate) async fn run_migrations(pool: &sqlx::SqlitePool) -> Result<()> {
    let mut conn = pool.acquire().await?;

    MIGRATOR.run(&mut conn).await.map_err(|e| {
        warn!("Migration error: {}", e);
        crate::error::Error::Internal(format!("Migration failed: {}", e))
    })?;

    info!("Applied migrations");
    Ok(())
}

pub(crate) async fn verify_migrations(pool: &sqlx::SqlitePool) -> Result<()> {
    use sqlx::Row;

    let latest_version = MIGRATOR.iter().map(|m| m.version).max().unwrap_or(0) as i64;
    let current: Option<i64> = sqlx::query("SELECT MAX(version) FROM _sqlx_migrations")
        .fetch_one(pool)
        .await?
        .get(0);

    if current != Some(latest_version) {
        return Err(crate::error::Error::Internal(format!(
            "Schema migration verification failed: expected version {}, got {:?}",
            latest_version, current
        )));
    }

    for embedded in MIGRATOR.iter() {
        let applied: Option<Vec<u8>> = sqlx::query(
            "SELECT checksum FROM _sqlx_migrations WHERE version = ?1 AND description = ?2"
        )
        .bind(embedded.version as i64)
        .bind(&embedded.description)
        .fetch_one(pool)
        .await?
        .get(0);

        match applied {
            Some(stored_checksum) if stored_checksum == embedded.checksum.as_ref() => continue,
            Some(_) => {
                return Err(crate::error::Error::Internal(format!(
                    "Migration checksum mismatch for v{}: {}",
                    embedded.version, embedded.description
                )));
            }
            None => {
                return Err(crate::error::Error::Internal(format!(
                    "Migration record missing in _sqlx_migrations for v{}: {}",
                    embedded.version, embedded.description
                )));
            }
        }
    }

    info!("Migration checksum verification passed");
    Ok(())
}
