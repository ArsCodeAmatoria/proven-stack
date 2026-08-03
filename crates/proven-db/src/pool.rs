use std::time::Duration;

use anyhow::Context;
use proven_config::Config;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tokio::time::timeout;

use crate::DbError;

pub type PostgresPool = PgPool;

/// Build a configured connection pool and verify with `SELECT 1`.
pub async fn connect_pool(config: &Config) -> Result<PostgresPool, DbError> {
    let db = &config.database;
    let connect = PgPoolOptions::new()
        .max_connections(db.max_connections.max(1))
        .min_connections(db.min_connections.min(db.max_connections))
        .acquire_timeout(Duration::from_secs(db.acquire_timeout_secs.max(1)))
        .idle_timeout(Duration::from_secs(db.idle_timeout_secs))
        .max_lifetime(Duration::from_secs(db.max_lifetime_secs))
        .test_before_acquire(true)
        .connect(db.url.expose());

    let pool = timeout(Duration::from_secs(db.acquire_timeout_secs.max(1) + 2), connect)
        .await
        .map_err(|_| DbError::Connect("postgres connect timed out".into()))?
        .map_err(|e| DbError::Connect(e.to_string()))?;

    timeout(
        Duration::from_secs(3),
        sqlx::query("SELECT 1").execute(&pool),
    )
    .await
    .map_err(|_| DbError::Connect("postgres ping timed out".into()))?
    .context("postgres ping")
    .map_err(|e| DbError::Connect(e.to_string()))?;

    tracing::info!(
        max = db.max_connections,
        min = db.min_connections,
        "postgres pool ready"
    );

    Ok(pool)
}
