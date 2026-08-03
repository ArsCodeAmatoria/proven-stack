# Go Guidelines

- Workers are **I/O-only** — no AuthZ or domain authority.
- Shared runtime under `go/internal/platform/`.
- `gofmt` + `go vet` required.
- Task queues are empty until workflows are registered intentionally.

See [`docs/architecture/GO_WORKERS_ARCHITECTURE.md`](../architecture/GO_WORKERS_ARCHITECTURE.md).
