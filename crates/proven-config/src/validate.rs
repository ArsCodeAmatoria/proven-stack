use crate::env::Environment;
use crate::error::ConfigError;
use crate::Config;

/// Validate secret material for the active environment.
pub fn validate_secrets(config: &Config) -> Result<(), ConfigError> {
    let mut reasons: Vec<String> = Vec::new();

    if config.secrets.session_secret.is_empty() {
        reasons.push("PROVEN_SESSION_SECRET is empty".into());
    }

    match config.environment {
        Environment::Development => {
            // Weak local secrets are allowed; still reject completely empty.
        }
        Environment::Testing => {
            if config.secrets.session_secret.len() < 16 {
                reasons
                    .push("PROVEN_SESSION_SECRET must be at least 16 characters in testing".into());
            }
            reject_placeholder_db(config.database.url.expose(), &mut reasons);
        }
        Environment::Production => {
            if config.secrets.session_secret.len() < 32 {
                reasons.push(
                    "PROVEN_SESSION_SECRET must be at least 32 characters in production".into(),
                );
            }
            if is_weak_session_secret(config.secrets.session_secret.expose()) {
                reasons.push("PROVEN_SESSION_SECRET looks like a development placeholder".into());
            }
            reject_placeholder_db(config.database.url.expose(), &mut reasons);
            reject_local_infra_urls(config, &mut reasons);
        }
    }

    if reasons.is_empty() {
        Ok(())
    } else {
        Err(ConfigError::secrets(reasons))
    }
}

/// Final startup checks (bind settings, required connectivity endpoints).
pub fn validate_startup(config: &Config) -> Result<(), ConfigError> {
    let mut reasons: Vec<String> = Vec::new();

    if config.server.host.trim().is_empty() {
        reasons.push("PROVEN_API_HOST is empty".into());
    }
    if config.server.port == 0 {
        reasons.push("PROVEN_API_PORT must be non-zero".into());
    }
    if config.nats.url.trim().is_empty() {
        reasons.push("NATS_URL is empty".into());
    }
    if config.temporal.address.trim().is_empty() {
        reasons.push("TEMPORAL_ADDRESS is empty".into());
    }
    if config.temporal.namespace.trim().is_empty() {
        reasons.push("TEMPORAL_NAMESPACE is empty".into());
    }

    if config.environment.is_production()
        && config
            .observability
            .rust_log
            .to_ascii_lowercase()
            .contains("trace")
    {
        reasons.push("RUST_LOG must not enable trace in production".into());
    }

    if reasons.is_empty() {
        Ok(())
    } else {
        Err(ConfigError::startup(reasons))
    }
}

fn reject_placeholder_db(url: &str, reasons: &mut Vec<String>) {
    let lower = url.to_ascii_lowercase();
    if lower.contains("proven:proven@")
        || lower.contains(":changeme@")
        || lower.contains(":password@")
        || lower.contains(":secret@")
    {
        reasons
            .push("DATABASE_URL must not use placeholder credentials outside development".into());
    }
}

fn reject_local_infra_urls(config: &Config, reasons: &mut Vec<String>) {
    for (key, value) in [
        ("DATABASE_URL", config.database.url.expose()),
        ("REDIS_URL", config.redis.url.expose()),
        ("NATS_URL", config.nats.url.as_str()),
        ("TEMPORAL_ADDRESS", config.temporal.address.as_str()),
    ] {
        if is_loopback_endpoint(value) {
            reasons.push(format!(
                "{key} must not point at localhost/loopback in production"
            ));
        }
    }
}

fn is_loopback_endpoint(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("127.0.0.1")
        || lower.contains("localhost")
        || lower.contains("[::1]")
        || lower.contains("0.0.0.0")
}

fn is_weak_session_secret(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("dev-only")
        || lower.contains("change-me")
        || lower.contains("changeme")
        || lower == "secret"
        || lower == "password"
}
