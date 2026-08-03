use std::time::{Duration, Instant};

use serde::Serialize;
use sqlx::PgPool;
use tokio::time::timeout;

#[derive(Debug, Clone, Serialize)]
pub struct DbHealth {
    pub ok: bool,
    pub latency_ms: u128,
    pub detail: String,
}

/// Live health probe (`SELECT 1`) against an existing pool.
pub async fn check_health(pool: &PgPool) -> DbHealth {
    let started = Instant::now();
    match timeout(Duration::from_secs(2), sqlx::query("SELECT 1").execute(pool)).await {
        Ok(Ok(_)) => DbHealth {
            ok: true,
            latency_ms: started.elapsed().as_millis(),
            detail: "ok".into(),
        },
        Ok(Err(err)) => DbHealth {
            ok: false,
            latency_ms: started.elapsed().as_millis(),
            detail: err.to_string(),
        },
        Err(_) => DbHealth {
            ok: false,
            latency_ms: started.elapsed().as_millis(),
            detail: "timed out".into(),
        },
    }
}
