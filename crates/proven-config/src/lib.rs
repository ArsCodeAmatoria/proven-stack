//! Typed configuration for Proven processes.
//!
//! Loads environment variables, validates required settings and secrets by
//! environment (development / testing / production), and fails fast at startup.
//! No business domain logic.

mod env;
mod error;
mod load;
mod secret;
mod validate;

pub use env::Environment;
pub use error::ConfigError;
pub use load::{load, load_from_iter, LoadOptions};
pub use secret::SecretString;
pub use validate::validate_startup;

use serde::Serialize;

/// Root configuration for the Proven API (and shared infra endpoints).
#[derive(Clone, Serialize)]
pub struct Config {
    pub environment: Environment,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub nats: NatsConfig,
    pub temporal: TemporalConfig,
    pub observability: ObservabilityConfig,
    pub secrets: SecretsConfig,
    pub infra: InfraConfig,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("environment", &self.environment)
            .field("server", &self.server)
            .field("database", &self.database)
            .field("redis", &self.redis)
            .field("nats", &self.nats)
            .field("temporal", &self.temporal)
            .field("observability", &self.observability)
            .field("secrets", &self.secrets)
            .field("infra", &self.infra)
            .finish()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl ServerConfig {
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Clone, Serialize)]
pub struct DatabaseConfig {
    pub url: SecretString,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub max_lifetime_secs: u64,
    /// When true, API runs pending migrations during boot.
    pub migrate_on_start: bool,
    pub migrations_dir: String,
}

impl std::fmt::Debug for DatabaseConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatabaseConfig")
            .field("url", &self.url)
            .field("max_connections", &self.max_connections)
            .field("min_connections", &self.min_connections)
            .field("acquire_timeout_secs", &self.acquire_timeout_secs)
            .field("idle_timeout_secs", &self.idle_timeout_secs)
            .field("max_lifetime_secs", &self.max_lifetime_secs)
            .field("migrate_on_start", &self.migrate_on_start)
            .field("migrations_dir", &self.migrations_dir)
            .finish()
    }
}

#[derive(Clone, Serialize)]
pub struct RedisConfig {
    pub url: SecretString,
}

impl std::fmt::Debug for RedisConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisConfig")
            .field("url", &self.url)
            .finish()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct NatsConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TemporalConfig {
    pub address: String,
    pub namespace: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObservabilityConfig {
    pub rust_log: String,
    pub service_name: String,
    pub service_version: String,
    /// When true and an OTLP endpoint is configured, export traces via OpenTelemetry.
    pub otel_enabled: bool,
    /// OTLP HTTP endpoint base (e.g. `http://127.0.0.1:4318`). Empty disables export.
    pub otel_endpoint: String,
    /// Head sample ratio in `0.0..=1.0`.
    pub otel_sample_ratio: f64,
    /// Expose Prometheus scrape endpoint at `/metrics`.
    pub metrics_enabled: bool,
    /// Force JSON logs. When unset in config loaders, production defaults to JSON.
    pub log_json: bool,
}

/// Infrastructure connection behavior (pools / soft-fail).
#[derive(Debug, Clone, Serialize)]
pub struct InfraConfig {
    /// When true, failed DB/Redis/NATS/Temporal connects warn instead of aborting boot.
    pub optional: bool,
    pub db_max_connections: u32,
}

/// Application secrets that must never be logged in plaintext.
#[derive(Clone, Serialize)]
pub struct SecretsConfig {
    /// Future session / cookie signing material (foundation placeholder).
    pub session_secret: SecretString,
}

impl std::fmt::Debug for SecretsConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretsConfig")
            .field("session_secret", &self.session_secret)
            .finish()
    }
}
