use std::collections::HashMap;
use std::env;
use std::path::Path;

use crate::env::Environment;
use crate::error::ConfigError;
use crate::secret::SecretString;
use crate::validate::{validate_secrets, validate_startup};
use crate::{
    Config, DatabaseConfig, InfraConfig, NatsConfig, ObservabilityConfig, RedisConfig,
    SecretsConfig, ServerConfig, TemporalConfig,
};

const DEFAULT_DEV_SESSION_SECRET: &str = "dev-only-session-secret-change-me-32b";

/// Options for loading configuration.
#[derive(Debug, Clone)]
pub struct LoadOptions {
    /// When true (default), load `.env` / `.env.<environment>` in development/testing.
    pub load_dotenv: bool,
    /// Optional explicit environment override (otherwise `PROVEN_ENV`).
    pub environment: Option<Environment>,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            load_dotenv: true,
            environment: None,
        }
    }
}

/// Load and validate configuration from the process environment.
pub fn load() -> Result<Config, ConfigError> {
    load_with_options(LoadOptions::default())
}

pub fn load_with_options(options: LoadOptions) -> Result<Config, ConfigError> {
    let preliminary_env = options
        .environment
        .or_else(|| env::var("PROVEN_ENV").ok().and_then(|v| v.parse().ok()))
        .unwrap_or(Environment::Development);

    if options.load_dotenv && preliminary_env.allows_dotenv() {
        load_dotenv_files(preliminary_env);
    }

    let map: HashMap<String, String> = env::vars().collect();
    load_from_map(&map, options.environment)
}

/// Load from an arbitrary key/value map (tests / alternate sources).
pub fn load_from_iter<I, K, V>(iter: I) -> Result<Config, ConfigError>
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
{
    let map: HashMap<String, String> = iter
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect();
    load_from_map(&map, None)
}

fn load_from_map(
    map: &HashMap<String, String>,
    env_override: Option<Environment>,
) -> Result<Config, ConfigError> {
    let environment = match env_override {
        Some(e) => e,
        None => match map.get("PROVEN_ENV") {
            Some(v) => v.parse()?,
            None => Environment::Development,
        },
    };

    let mut missing: Vec<String> = Vec::new();

    let host = get_or_default(map, "PROVEN_API_HOST", "0.0.0.0");
    let port = parse_port(map, "PROVEN_API_PORT", 8080)?;

    let database_url = require_or_default(
        map,
        "DATABASE_URL",
        environment,
        "postgres://proven:proven@127.0.0.1:5432/proven",
        &mut missing,
    );
    let redis_url = require_or_default(
        map,
        "REDIS_URL",
        environment,
        "redis://127.0.0.1:6379",
        &mut missing,
    );
    let nats_url = require_or_default(
        map,
        "NATS_URL",
        environment,
        "nats://127.0.0.1:4222",
        &mut missing,
    );
    let temporal_address = require_or_default(
        map,
        "TEMPORAL_ADDRESS",
        environment,
        "127.0.0.1:7233",
        &mut missing,
    );
    let temporal_namespace = get_or_default(map, "TEMPORAL_NAMESPACE", "default");

    let session_secret = match map.get("PROVEN_SESSION_SECRET") {
        Some(v) if !v.is_empty() => v.clone(),
        _ if environment == Environment::Development => DEFAULT_DEV_SESSION_SECRET.to_string(),
        _ => {
            missing.push("PROVEN_SESSION_SECRET".into());
            String::new()
        }
    };

    let rust_log = get_or_default(map, "RUST_LOG", default_rust_log(environment));
    let service_name = get_or_default(map, "PROVEN_SERVICE_NAME", "proven-api");
    let service_version = first_non_empty(&[
        map.get("PROVEN_SERVICE_VERSION").cloned().unwrap_or_default(),
        map.get("GIT_SHA").cloned().unwrap_or_default(),
        "0.1.0".into(),
    ]);
    let otel_endpoint = first_non_empty(&[
        map.get("OTEL_EXPORTER_OTLP_ENDPOINT")
            .cloned()
            .unwrap_or_default(),
        map.get("PROVEN_OTEL_ENDPOINT").cloned().unwrap_or_default(),
    ]);
    let otel_enabled = parse_bool(
        map,
        "PROVEN_OTEL_ENABLED",
        !otel_endpoint.is_empty(),
    )?;
    let otel_sample_ratio = parse_f64(map, "PROVEN_OTEL_SAMPLE_RATIO", 1.0)?.clamp(0.0, 1.0);
    let metrics_enabled = parse_bool(map, "PROVEN_METRICS_ENABLED", true)?;
    let log_json = parse_bool(
        map,
        "PROVEN_LOG_JSON",
        matches!(environment, Environment::Production | Environment::Testing),
    )?;
    let infra_optional = parse_bool(
        map,
        "PROVEN_INFRA_OPTIONAL",
        matches!(environment, Environment::Development),
    )?;
    let db_max_connections = parse_u32(map, "PROVEN_DB_MAX_CONNECTIONS", 10)?;
    let db_min_connections = parse_u32(map, "PROVEN_DB_MIN_CONNECTIONS", 1)?;
    let db_acquire_timeout_secs = parse_u32(map, "PROVEN_DB_ACQUIRE_TIMEOUT_SECS", 5)? as u64;
    let db_idle_timeout_secs = parse_u32(map, "PROVEN_DB_IDLE_TIMEOUT_SECS", 600)? as u64;
    let db_max_lifetime_secs = parse_u32(map, "PROVEN_DB_MAX_LIFETIME_SECS", 1800)? as u64;
    let migrate_on_start = parse_bool(
        map,
        "PROVEN_MIGRATE_ON_START",
        matches!(environment, Environment::Development),
    )?;
    let migrations_dir = get_or_default(map, "PROVEN_MIGRATIONS_DIR", "db/migrations/platform");

    if !missing.is_empty() {
        return Err(ConfigError::missing(missing));
    }

    let config = Config {
        environment,
        server: ServerConfig { host, port },
        database: DatabaseConfig {
            url: SecretString::new(database_url),
            max_connections: db_max_connections.max(1),
            min_connections: db_min_connections.min(db_max_connections.max(1)),
            acquire_timeout_secs: db_acquire_timeout_secs.max(1),
            idle_timeout_secs: db_idle_timeout_secs,
            max_lifetime_secs: db_max_lifetime_secs,
            migrate_on_start,
            migrations_dir,
        },
        redis: RedisConfig {
            url: SecretString::new(redis_url),
        },
        nats: NatsConfig { url: nats_url },
        temporal: TemporalConfig {
            address: temporal_address,
            namespace: temporal_namespace,
        },
        observability: ObservabilityConfig {
            rust_log,
            service_name,
            service_version,
            otel_enabled,
            otel_endpoint,
            otel_sample_ratio,
            metrics_enabled,
            log_json,
        },
        secrets: SecretsConfig {
            session_secret: SecretString::new(session_secret),
        },
        infra: InfraConfig {
            optional: infra_optional,
            db_max_connections,
        },
    };

    validate_secrets(&config)?;
    validate_startup(&config)?;
    Ok(config)
}

fn load_dotenv_files(environment: Environment) {
    // Layered: `.env` then `.env.<environment>` (later wins via dotenvy if already unset).
    // dotenvy does not override existing process env by default.
    let _ = dotenvy::dotenv();
    let specific = format!(".env.{}", environment.as_str());
    if Path::new(&specific).exists() {
        let _ = dotenvy::from_filename(&specific);
    }
    // Optional examples-style path used in docs.
    let example_path = format!("config/examples/{}.env", environment.as_str());
    if Path::new(&example_path).exists() && env::var_os("PROVEN_LOAD_EXAMPLE_ENV").is_some() {
        let _ = dotenvy::from_filename(&example_path);
    }
}

fn get_or_default(map: &HashMap<String, String>, key: &str, default: &str) -> String {
    map.get(key)
        .filter(|v| !v.is_empty())
        .cloned()
        .unwrap_or_else(|| default.to_string())
}

fn require_or_default(
    map: &HashMap<String, String>,
    key: &str,
    environment: Environment,
    development_default: &str,
    missing: &mut Vec<String>,
) -> String {
    if let Some(v) = map.get(key).filter(|v| !v.is_empty()) {
        return v.clone();
    }
    match environment {
        Environment::Development => development_default.to_string(),
        Environment::Testing | Environment::Production => {
            missing.push(key.to_string());
            String::new()
        }
    }
}

fn parse_port(map: &HashMap<String, String>, key: &str, default: u16) -> Result<u16, ConfigError> {
    match map.get(key) {
        None => Ok(default),
        Some(v) if v.is_empty() => Ok(default),
        Some(v) => v.parse::<u16>().map_err(|_| ConfigError::Invalid {
            key: key.into(),
            reason: format!("expected u16 port, got '{v}'"),
        }),
    }
}

fn parse_u32(map: &HashMap<String, String>, key: &str, default: u32) -> Result<u32, ConfigError> {
    match map.get(key) {
        None => Ok(default),
        Some(v) if v.is_empty() => Ok(default),
        Some(v) => v.parse::<u32>().map_err(|_| ConfigError::Invalid {
            key: key.into(),
            reason: format!("expected u32, got '{v}'"),
        }),
    }
}

fn parse_bool(
    map: &HashMap<String, String>,
    key: &str,
    default: bool,
) -> Result<bool, ConfigError> {
    match map.get(key) {
        None => Ok(default),
        Some(v) if v.is_empty() => Ok(default),
        Some(v) => match v.to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Ok(true),
            "0" | "false" | "no" | "off" => Ok(false),
            other => Err(ConfigError::Invalid {
                key: key.into(),
                reason: format!("expected boolean, got '{other}'"),
            }),
        },
    }
}

fn parse_f64(map: &HashMap<String, String>, key: &str, default: f64) -> Result<f64, ConfigError> {
    match map.get(key) {
        None => Ok(default),
        Some(v) if v.is_empty() => Ok(default),
        Some(v) => v.parse::<f64>().map_err(|_| ConfigError::Invalid {
            key: key.into(),
            reason: format!("expected f64, got '{v}'"),
        }),
    }
}

fn first_non_empty(values: &[String]) -> String {
    values
        .iter()
        .map(|s| s.trim())
        .find(|s| !s.is_empty())
        .unwrap_or("")
        .to_string()
}

fn default_rust_log(environment: Environment) -> &'static str {
    match environment {
        Environment::Development => "info,tower_http=info",
        Environment::Testing => "info",
        Environment::Production => "info,tower_http=warn",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn development_loads_with_defaults() {
        let cfg = load_from_iter([("PROVEN_ENV", "development")]).unwrap();
        assert_eq!(cfg.environment, Environment::Development);
        assert_eq!(cfg.server.port, 8080);
        assert!(!cfg.database.url.expose().is_empty());
    }

    #[test]
    fn production_detects_missing_keys() {
        let err = load_from_iter([("PROVEN_ENV", "production")]).unwrap_err();
        match err {
            ConfigError::Missing { keys } => {
                assert!(keys.contains("DATABASE_URL"));
                assert!(keys.contains("REDIS_URL"));
                assert!(keys.contains("NATS_URL"));
                assert!(keys.contains("TEMPORAL_ADDRESS"));
                assert!(keys.contains("PROVEN_SESSION_SECRET"));
            }
            other => panic!("expected Missing, got {other:?}"),
        }
    }

    #[test]
    fn production_rejects_weak_secrets() {
        let err = load_from_iter([
            ("PROVEN_ENV", "production"),
            (
                "DATABASE_URL",
                "postgres://proven:proven@db.example.com:5432/proven",
            ),
            ("REDIS_URL", "redis://redis.example.com:6379"),
            ("NATS_URL", "nats://nats.example.com:4222"),
            ("TEMPORAL_ADDRESS", "temporal.example.com:7233"),
            ("PROVEN_SESSION_SECRET", "short"),
        ])
        .unwrap_err();
        assert!(matches!(err, ConfigError::Secrets { .. }));
    }

    #[test]
    fn production_accepts_strong_config() {
        let cfg = load_from_iter([
            ("PROVEN_ENV", "production"),
            (
                "DATABASE_URL",
                "postgres://app:s3cure-P@ssw0rd-long@db.internal:5432/proven",
            ),
            ("REDIS_URL", "rediss://:token@redis.internal:6379"),
            ("NATS_URL", "tls://nats.internal:4222"),
            ("TEMPORAL_ADDRESS", "temporal.internal:7233"),
            (
                "PROVEN_SESSION_SECRET",
                "production-session-secret-value-32chars-min",
            ),
        ])
        .unwrap();
        assert!(cfg.environment.is_production());
    }

    #[test]
    fn secrets_are_redacted_in_debug() {
        let cfg = load_from_iter([("PROVEN_ENV", "development")]).unwrap();
        let debug = format!("{cfg:?}");
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains(cfg.database.url.expose()));
    }

    #[test]
    #[serial]
    fn invalid_env_rejected() {
        let err = load_from_iter([("PROVEN_ENV", "staging")]).unwrap_err();
        assert!(matches!(err, ConfigError::Invalid { .. }));
    }
}
