//! Process-wide observability bootstrap.

use proven_config::Config;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use crate::metrics::{install_metrics, PrometheusHandle};
use crate::otel::{init_otel_tracer, otel_layer, OtelGuard};

/// Owns metrics + OTel resources for the process lifetime.
pub struct ObservabilityHandle {
    pub metrics: Option<PrometheusHandle>,
    _otel: Option<OtelGuard>,
}

/// Initialize structured logging, optional OTLP tracing, and Prometheus metrics.
pub fn init_observability(config: &Config) -> anyhow::Result<ObservabilityHandle> {
    let obs = &config.observability;

    let filter = EnvFilter::try_new(&obs.rust_log)
        .or_else(|_| EnvFilter::try_from_default_env())
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let (otel_guard, tracer) = init_otel_tracer(obs)?;

    let metrics = if obs.metrics_enabled {
        Some(install_metrics()?)
    } else {
        None
    };

    match (obs.log_json, tracer) {
        (true, Some(tracer)) => {
            tracing_subscriber::registry()
                .with(filter)
                .with(tracing_subscriber::fmt::layer().json().with_target(true))
                .with(otel_layer(tracer))
                .init();
        }
        (true, None) => {
            tracing_subscriber::registry()
                .with(filter)
                .with(tracing_subscriber::fmt::layer().json().with_target(true))
                .init();
        }
        (false, Some(tracer)) => {
            tracing_subscriber::registry()
                .with(filter)
                .with(tracing_subscriber::fmt::layer().compact().with_target(true))
                .with(otel_layer(tracer))
                .init();
        }
        (false, None) => {
            tracing_subscriber::registry()
                .with(filter)
                .with(tracing_subscriber::fmt::layer().compact().with_target(true))
                .init();
        }
    }

    if obs.otel_enabled && !obs.otel_endpoint.trim().is_empty() {
        tracing::info!(
            service = %obs.service_name,
            version = %obs.service_version,
            endpoint = %obs.otel_endpoint,
            sample_ratio = obs.otel_sample_ratio,
            "opentelemetry tracing enabled (OTLP)"
        );
    } else {
        tracing::info!(
            service = %obs.service_name,
            version = %obs.service_version,
            "opentelemetry tracing disabled (logs/metrics only)"
        );
    }

    Ok(ObservabilityHandle {
        metrics,
        _otel: otel_guard,
    })
}
