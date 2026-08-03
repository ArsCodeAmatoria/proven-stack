use std::path::{Path, PathBuf};

use serde::Serialize;
use sqlx::migrate::{MigrateError, Migrator};
use sqlx::PgPool;

use crate::DbError;

#[derive(Debug, Clone, Serialize)]
pub struct MigrationStatus {
    /// Rows present in `_sqlx_migrations` after the run.
    pub applied: usize,
    pub directory: String,
}

/// Resolve migrations directory (env `PROVEN_MIGRATIONS_DIR` or default).
pub fn migrations_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("PROVEN_MIGRATIONS_DIR") {
        if !dir.trim().is_empty() {
            return PathBuf::from(dir);
        }
    }
    PathBuf::from("db/migrations/platform")
}

/// Run pending sqlx migrations from `dir` (creates `_sqlx_migrations` metadata).
pub async fn migrate(pool: &PgPool, dir: impl AsRef<Path>) -> Result<MigrationStatus, DbError> {
    let dir = dir.as_ref();
    if !dir.exists() {
        return Err(DbError::Migrate(format!(
            "migrations directory not found: {}",
            dir.display()
        )));
    }

    let migrator = Migrator::new(dir.to_path_buf())
        .await
        .map_err(map_migrate_err)?;

    let known = migrator.iter().count();

    migrator.run(pool).await.map_err(map_migrate_err)?;

    let applied: (i64,) = sqlx::query_as("SELECT COUNT(*)::bigint FROM _sqlx_migrations")
        .fetch_one(pool)
        .await
        .map_err(|e| DbError::Migrate(e.to_string()))?;

    tracing::info!(
        directory = %dir.display(),
        known,
        applied = applied.0,
        "migrations applied"
    );

    Ok(MigrationStatus {
        applied: applied.0 as usize,
        directory: dir.display().to_string(),
    })
}

fn map_migrate_err(err: MigrateError) -> DbError {
    DbError::Migrate(err.to_string())
}
