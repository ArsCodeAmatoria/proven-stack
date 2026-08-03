//! Logging + tracing bootstrap (delegates to `proven-observability`).

use proven_config::Config;
use proven_observability::{init_observability, ObservabilityHandle};

/// Initialize structured logging, metrics, and optional OpenTelemetry.
pub fn init_tracing(config: &Config) -> anyhow::Result<ObservabilityHandle> {
    init_observability(config)
}
