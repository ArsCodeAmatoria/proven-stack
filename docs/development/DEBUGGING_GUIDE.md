# Debugging Guide

- API logs: `RUST_LOG` / structured JSON in prod-like envs.
- Correlation: `x-request-id` + `x-correlation-id` on API responses.
- Metrics: `GET /metrics` (Prometheus text).
- VS Code launch configs: `.vscode/launch.json` (API, web, worker).
- Temporal UI (compose): `http://localhost:8088`.
- Auth sessions are in-memory for foundation Better Auth — restart clears users.
