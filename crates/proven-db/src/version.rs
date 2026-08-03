use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

use crate::DbError;

#[derive(Debug, Clone, Serialize)]
pub struct DatabaseVersion {
    pub postgres_version: String,
    pub migration_version: Option<i64>,
    pub migration_description: Option<String>,
    pub migration_installed_on: Option<DateTime<Utc>>,
    pub migration_success: Option<bool>,
    pub applied_migrations: i64,
}

/// Report Postgres server version and latest sqlx migration metadata.
pub async fn database_version(pool: &PgPool) -> Result<DatabaseVersion, DbError> {
    let (postgres_version,): (String,) = sqlx::query_as("SELECT version()")
        .fetch_one(pool)
        .await
        .map_err(|e| DbError::Version(e.to_string()))?;

    // `_sqlx_migrations` exists only after the first migrate run.
    let table_exists: (bool,) = sqlx::query_as(
        r#"
        SELECT EXISTS (
          SELECT 1
          FROM information_schema.tables
          WHERE table_schema = 'public'
            AND table_name = '_sqlx_migrations'
        )
        "#,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| DbError::Version(e.to_string()))?;

    if !table_exists.0 {
        return Ok(DatabaseVersion {
            postgres_version,
            migration_version: None,
            migration_description: None,
            migration_installed_on: None,
            migration_success: None,
            applied_migrations: 0,
        });
    }

    let applied: (i64,) = sqlx::query_as("SELECT COUNT(*)::bigint FROM _sqlx_migrations")
        .fetch_one(pool)
        .await
        .map_err(|e| DbError::Version(e.to_string()))?;

    let latest: Option<(i64, String, DateTime<Utc>, bool)> = sqlx::query_as(
        r#"
        SELECT version, description, installed_on, success
        FROM _sqlx_migrations
        ORDER BY version DESC
        LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| DbError::Version(e.to_string()))?;

    let (migration_version, migration_description, migration_installed_on, migration_success) =
        match latest {
            Some((v, d, t, s)) => (Some(v), Some(d), Some(t), Some(s)),
            None => (None, None, None, None),
        };

    Ok(DatabaseVersion {
        postgres_version,
        migration_version,
        migration_description,
        migration_installed_on,
        migration_success,
        applied_migrations: applied.0,
    })
}
