//! Process-wide dependency injection root (`AppState`).

use std::path::Path;
use std::sync::Arc;

use anyhow::Context;
use proven_companies::CompaniesModule;
use proven_config::Config;
use proven_core::CoreModule;
use proven_db::{check_health, migrate};
use proven_projects::ProjectsModule;
use proven_users::UsersModule;
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
    core: CoreModule,
    companies: CompaniesModule,
    users: UsersModule,
    projects: ProjectsModule,
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
                    for dir in migration_dirs(&config) {
                        let path = Path::new(&dir);
                        match migrate(&pool, path).await {
                            Ok(status) => info!(
                                applied = status.applied,
                                directory = %status.directory,
                                "database migrations applied on start"
                            ),
                            Err(err) if optional => {
                                warn!(
                                    error = %err,
                                    directory = %dir,
                                    "migrate on start failed (infra optional)"
                                )
                            }
                            Err(err) => {
                                return Err(err).context(format!(
                                    "database migrate on start failed ({dir})"
                                ));
                            }
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

        let core = CoreModule::in_memory();
        let companies = CompaniesModule::with_core(core.services.clone());
        let users = UsersModule::with_core(core.services.clone());
        let projects = ProjectsModule::with_core(core.services.clone());
        info!("proven-core module ready (in-memory ports; AuthzApi authoritative)");
        info!("proven-companies module ready (profile SoR; CompanyId from Core)");
        info!("proven-users module ready (account profile SoR; UserId from Core)");
        info!("proven-projects module ready (Place SoR; membership via Core MembershipApi)");

        Ok(Self {
            inner: Arc::new(AppStateInner {
                config,
                db,
                redis,
                nats,
                temporal,
                metrics,
                core,
                companies,
                users,
                projects,
            }),
        })
    }

    /// Build state without opening network connections (unit tests / OpenAPI).
    pub fn for_tests(config: Config) -> Self {
        let core = CoreModule::in_memory();
        let companies = CompaniesModule::with_core(core.services.clone());
        let users = UsersModule::with_core(core.services.clone());
        let projects = ProjectsModule::with_core(core.services.clone());
        Self {
            inner: Arc::new(AppStateInner {
                config,
                db: None,
                redis: None,
                nats: None,
                temporal: None,
                metrics: None,
                core,
                companies,
                users,
                projects,
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

    /// NATS event publisher when the client is connected.
    pub fn event_publisher(&self) -> Option<proven_events::NatsEventPublisher> {
        self.nats().map(crate::infra::event_publisher)
    }

    /// NATS event subscriber when the client is connected.
    pub fn event_subscriber(&self) -> Option<proven_events::NatsEventSubscriber> {
        self.nats().map(crate::infra::event_subscriber)
    }

    pub fn temporal(&self) -> Option<&TemporalHandle> {
        self.inner.temporal.as_ref()
    }

    pub fn metrics(&self) -> Option<&PrometheusHandle> {
        self.inner.metrics.as_ref()
    }

    pub fn core(&self) -> &CoreModule {
        &self.inner.core
    }

    pub fn companies(&self) -> &CompaniesModule {
        &self.inner.companies
    }

    pub fn users(&self) -> &UsersModule {
        &self.inner.users
    }

    pub fn projects(&self) -> &ProjectsModule {
        &self.inner.projects
    }

    pub fn infra_ready(&self) -> bool {
        self.inner.db.is_some()
            && self.inner.redis.is_some()
            && self.inner.nats.is_some()
            && self.inner.temporal.is_some()
    }

    pub async fn postgres_healthy(&self) -> bool {
        match self.db() {
            Some(pool) => check_health(pool).await.ok,
            None => false,
        }
    }
}

fn migration_dirs(config: &Config) -> Vec<String> {
    if !config.database.migrations_dir.is_empty()
        && config.database.migrations_dir != "db/migrations/platform"
    {
        return vec![config.database.migrations_dir.clone()];
    }
    vec![
        "db/migrations/platform".to_string(),
        "db/migrations/core".to_string(),
        "db/migrations/companies".to_string(),
        "db/migrations/users".to_string(),
        "db/migrations/projects".to_string(),
    ]
}
