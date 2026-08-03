# Observability infrastructure

Foundation exporters and hooks only — **no Grafana dashboards** in this milestone.

## Layout

```text
deploy/observability/
├── otel-collector.yaml          # OTLP receive → logging exporter
├── README.md
└── (dashboards deferred)
```

## Local collector (optional)

```bash
docker compose \
  -f docker/compose/docker-compose.yml \
  -f docker/compose/docker-compose.observability.yml \
  --project-directory . \
  up -d otel-collector
```

Point apps at the collector:

```bash
export PROVEN_OTEL_ENABLED=true
export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4318
```

## Signals

| Signal | API | Workers |
| --- | --- | --- |
| Logs | structured `tracing` JSON/compact | structured `slog` |
| Traces | OTLP HTTP via `proven-observability` | OTLP HTTP via `platform/tracing` |
| Metrics | `GET /metrics` Prometheus | `GET /metrics` Prometheus |
| Health | `/healthz`, `/readyz` | `/healthz`, `/readyz` |
| IDs | `x-request-id`, `x-correlation-id` | same headers |
