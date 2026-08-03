//! Process-wide dependency injection root (`AppState`).

use std::sync::Arc;

use std::path::Path;

use anyhow::Context;
use proven_config::Config;
use proven_db::{check_health, migrate};
use tracing::{info, warn};

use crate::infra::{
    connect_nats, connect_postgres, connect_redis, connect_temporal, NatsHandle, PostgresPool,
    RedisHandle, TemporalHandle,
};
use proven_observability::PrometheusHandle;

/// Shared application state injected into Axum handlers via `State<AppState>`.
#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    config: Config,
    db: Option<PostgresPool>,
    redis: Option<RedisHandle>,
    nats: Option<NatsHandle>,
    temporal: Option<TemporalHandle>,
    metrics: Option<PrometheusHandle>,
}

impl AppState {
    /// Connect infrastructure clients and build the DI container.
    pub async fn connect(
        config: Config,
        metrics: Option<PrometheusHandle>,
    ) -> anyhow::Result<Self> {
        let optional = config.infra.optional;

        let db = match connect_postgres(&config).await {
            Ok(pool) => {
                if config.database.migrate_on_start {
                    let dir = Path::new(&config.database.migrations_dir);
                    match migrate(&pool, dir).await {
                        Ok(status) => info!(
                            applied = status.applied,
                            directory = %status.directory,
                            "database migrations applied on start"
                        ),
                        Err(err) if optional => {
                            warn!(error = %err, "migrate on start failed (infra optional)")
                        }
                        Err(err) => {
                            return Err(err).context("database migrate on start failed");
                        }
                    }
                }
                info!("postgres pool ready");
                Some(pool)
            }
            Err(err) if optional => {
                warn!(error = %err, "postgres unavailable (infra optional)");
                None
            }
            Err(err) => return Err(err).context("postgres connection failed"),
        };

        let redis = match connect_redis(&config).await {
            Ok(client) => {
                info!("redis connection manager ready");
                Some(client)
            }
            Err(err) if optional => {
                warn!(error = %err, "redis unavailable (infra optional)");
                None
            }
            Err(err) => return Err(err).context("redis connection failed"),
        };

        let nats = match connect_nats(&config).await {
            Ok(client) => {
                info!("nats client ready");
                Some(client)
            }
            Err(err) if optional => {
                warn!(error = %err, "nats unavailable (infra optional)");
                None
            }
            Err(err) => return Err(err).context("nats connection failed"),
        };

        let temporal = match connect_temporal(&config).await {
            Ok(client) => {
                info!(
                    address = %client.address(),
                    namespace = %client.namespace(),
                    "temporal client configured"
                );
                Some(client)
            }
            Err(err) if optional => {
                warn!(error = %err, "temporal unavailable (infra optional)");
                None
            }
            Err(err) => return Err(err).context("temporal client failed"),
        };

        Ok(Self {
            inner: Arc::new(AppStateInner {
                config,
                db,
                redis,
                nats,
                temporal,
                metrics,
            }),
        })
    }

    /// Build state without opening network connections (unit tests / OpenAPI).
    pub fn for_tests(config: Config) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                config,
                db: None,
                redis: None,
                nats: None,
                temporal: None,
                metrics: None,
            }),
        }
    }

    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    pub fn db(&self) -> Option<&PostgresPool> {
        self.inner.db.as_ref()
    }

    pub fn redis(&self) -> Option<&RedisHandle> {
        self.inner.redis.as_ref()
    }

    pub fn nats(&self) -> Option<&NatsHandle> {
        self.inner.nats.as_ref()
    }

    pub fn temporal(&self) -> Option<&TemporalHandle> {
        self.inner.temporal.as_ref()
    }

    pub fn metrics(&self) -> Option<&PrometheusHandle> {
        self.inner.metrics.as_ref()
    }

    pub fn infra_ready(&self) -> bool {
        self.inner.db.is_some()
            && self.inner.redis.is_some()
            && self.inner.nats.is_some()
            && self.inner.temporal.is_some()
    }

    /// Live Postgres probe when a pool exists.
    pub async fn postgres_healthy(&self) -> bool {
        match self.db() {
            Some(pool) => check_health(pool).await.ok,
            None => false,
        }
    }
}
