//! OpenTelemetry tracing hooks (OTLP HTTP when enabled).

use anyhow::Context;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::resource::Resource;
use opentelemetry_sdk::trace::{RandomIdGenerator, Sampler, SdkTracerProvider};
use opentelemetry_semantic_conventions::resource::{SERVICE_NAME, SERVICE_VERSION};
use proven_config::ObservabilityConfig;
use tracing::Subscriber;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::registry::LookupSpan;

/// Holds the SDK tracer provider so Drop can flush exporters.
pub struct OtelGuard {
    provider: SdkTracerProvider,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        if let Err(err) = self.provider.shutdown() {
            eprintln!("opentelemetry shutdown error: {err}");
        }
    }
}

pub fn init_otel_tracer(
    config: &ObservabilityConfig,
) -> anyhow::Result<(Option<OtelGuard>, Option<opentelemetry_sdk::trace::Tracer>)> {
    if !config.otel_enabled || config.otel_endpoint.trim().is_empty() {
        return Ok((None, None));
    }

    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    let endpoint = normalize_otlp_http_endpoint(&config.otel_endpoint);
    let resource = Resource::builder()
        .with_attributes([
            KeyValue::new(SERVICE_NAME, config.service_name.clone()),
            KeyValue::new(SERVICE_VERSION, config.service_version.clone()),
        ])
        .build();

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_endpoint(endpoint)
        .build()
        .context("build OTLP HTTP span exporter")?;

    let sampler = if config.otel_sample_ratio >= 1.0 {
        Sampler::AlwaysOn
    } else if config.otel_sample_ratio <= 0.0 {
        Sampler::AlwaysOff
    } else {
        Sampler::TraceIdRatioBased(config.otel_sample_ratio)
    };

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_sampler(sampler)
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(resource)
        .build();

    let tracer = provider.tracer(config.service_name.clone());
    opentelemetry::global::set_tracer_provider(provider.clone());

    Ok((Some(OtelGuard { provider }), Some(tracer)))
}

pub fn otel_layer<S>(
    tracer: opentelemetry_sdk::trace::Tracer,
) -> OpenTelemetryLayer<S, opentelemetry_sdk::trace::Tracer>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    OpenTelemetryLayer::new(tracer)
}

fn normalize_otlp_http_endpoint(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.ends_with("/v1/traces") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/v1/traces")
    }
}
