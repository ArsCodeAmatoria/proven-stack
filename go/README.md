# Proven Go Workers

I/O-only worker processes. **No domain authority** and **no workflows registered** in foundation.

## Binaries

| Command | Port | Default Temporal task queue |
| --- | --- | --- |
| `cmd/notify-worker` | 8091 | `proven-notify` |
| `cmd/temporal-io-worker` | 8092 | `proven-io` |
| `cmd/media-worker` | 8093 | `proven-media` |
| `cmd/analytics-worker` | 8094 | `proven-analytics` |

```bash
cd go
PROVEN_ENV=development PROVEN_INFRA_OPTIONAL=true go run ./cmd/notify-worker
curl http://127.0.0.1:8091/health
curl http://127.0.0.1:8091/readyz
```

## Folder structure

```text
go/
├── cmd/*/main.go
├── internal/
│   ├── app/                 # DI, run loop, graceful shutdown
│   ├── config/              # typed env + validation
│   └── platform/
│       ├── health/          # /health /healthz /readyz
│       ├── logging/         # slog (text/JSON)
│       ├── natsx/           # NATS connect helpers
│       ├── temporalx/       # Temporal client + empty worker
│       └── retry/           # exponential backoff + jitter
└── README.md
```

## Boot sequence

1. Load/validate config  
2. Structured logging  
3. Connect NATS (retry policy)  
4. Dial Temporal + register empty worker on task queue  
5. Serve health endpoints  
6. Block until SIGINT/SIGTERM → drain HTTP, stop Temporal worker, drain NATS  

## Configuration (selected)

| Variable | Notes |
| --- | --- |
| `PROVEN_ENV` | `development` / `testing` / `production` |
| `PROVEN_INFRA_OPTIONAL` | default `true` in development |
| `NATS_URL` | NATS server |
| `TEMPORAL_ADDRESS` | host:port |
| `TEMPORAL_NAMESPACE` | default `default` |
| `TEMPORAL_TASK_QUEUE` | overrides per-binary default |
| `PROVEN_RETRY_MAX_ATTEMPTS` | shared connect/retry policy |
| `PROVEN_SHUTDOWN_TIMEOUT_SEC` | graceful shutdown budget |

See root `.env.example`.
