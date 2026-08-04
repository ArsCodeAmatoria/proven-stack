# ADR-0013: REST API Conventions

| Field | Value |
| --- | --- |
| Status | Accepted |
| Date | 2026-08-03 |
| Deciders | Lead Software Engineering |

## Context

[REST_API.md](../architecture/REST_API.md) defines versioning, envelopes, pagination, filtering,
sorting, search, errors, AuthN/AuthZ, rate limits, OpenAPI, health, and metrics. Handlers had
diverged: offset paging, flat problem bodies, no shared list-query helpers, and no host-wide rate
limit or versioned OpenAPI path.

## Decision

1. **`proven-shared` owns wire conventions** (no Axum): `{ data }` / `{ data, pagination }`, nested
   `{ error }`, cursor + offset paging, `ListQuery` (sort / `q`), filter whitelist, validation
   helpers, `/api/v1` constants.
2. **`proven-platform` applies transport to every route**: `X-Api-Version`, AuthN credential gate
   (Bearer or interim headers), in-process rate limits + headers, correlation, metrics, tracing,
   nested `ApiError`, Swagger `/docs`, Redoc `/redoc`, OpenAPI at `/api-docs/openapi.json` and
   `/api/v1/openapi.json` with security schemes.
3. **Errors**: domain validation → HTTP **422** (`validation_failed`); malformed query → **400**;
   rate limit → **429** + `Retry-After`.
4. **Lists (future endpoints)**: cursor pagination (default 25, max 100); strict unknown filters;
   sort whitelist; optional `q`.
5. **AuthZ** remains Core `AuthzApi` via `require_permission` (ADR-0003 / ADR-0007).
6. Existing resource handlers may still return bare JSON until migrated; **new endpoints must use
   shared envelopes and list helpers**.

## Consequences

- Modules depend on `proven-shared` types only for wire shapes; no per-module error schema.
- Better Auth JWT validation replaces interim headers when adapter lands; OpenAPI already documents
  `bearerAuth`.
- Redis-backed rate limits can replace the in-memory window without changing response headers.
