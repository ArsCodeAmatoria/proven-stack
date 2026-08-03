# proven-observability

Process observability infrastructure: structured logs, Prometheus metrics, correlation IDs, and optional OpenTelemetry OTLP export.

**No dashboards** — exporters and hooks only.

## Usage

```rust
let handle = proven_observability::init_observability(&config)?;
// keep `handle` alive for the process; drop flushes OTel
```

## Env

| Variable | Purpose |
| --- | --- |
| `RUST_LOG` | tracing filter |
| `PROVEN_LOG_JSON` | JSON logs |
| `PROVEN_METRICS_ENABLED` | install Prometheus recorder |
| `PROVEN_OTEL_ENABLED` | enable OTLP when endpoint set |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | Collector base URL |
| `PROVEN_OTEL_SAMPLE_RATIO` | head sample ratio |
| `PROVEN_SERVICE_NAME` / `PROVEN_SERVICE_VERSION` | resource attributes |
