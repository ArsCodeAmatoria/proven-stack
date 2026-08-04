# REST API Conventions

Canonical design: [ADR-0013](../adr/0013-rest-api-conventions.md) and
[REST_API.md](../architecture/REST_API.md).

**Every new HTTP endpoint must follow these rules.** Prefer types from `proven-shared` and host
middleware from `proven-platform`.

## Versioning

| Item | Value |
| --- | --- |
| URI prefix | `/api/v1/...` (`API_V1_PREFIX`) |
| Response header | `X-Api-Version: v1` |
| Breaking changes | New major (`/api/v2`) |

## Success envelopes

```json
{ "data": { /* resource */ } }
```

```json
{
  "data": [ /* items */ ],
  "pagination": { "next_cursor": "...", "has_more": true }
}
```

Types: `DataEnvelope<T>`, `ListEnvelope<T>`, `PaginationMeta`.

## Pagination

- **Default for lists:** cursor (`?limit=&cursor=`). Default limit **25**, max **100**.
- Types: `CursorPageRequest`, `CursorPage`, `ListEnvelope::from_cursor_page`.
- Offset (`Page` / `PageRequest`) is for admin/export/search SoR only — not hot field lists.

## Filtering / sorting / search

| Concern | Rule |
| --- | --- |
| Filters | Strict whitelist via `require_known_filters` |
| Multi-value | CSV via `parse_multi_value` |
| Sort | `?sort=field:asc,other:desc` + whitelist (`ListQuery::parse`) |
| Search | `?q=` (max 200 chars); global `GET /api/v1/search` later |

## Errors

Nested envelope (not flat RFC7807 title/status):

```json
{
  "error": {
    "code": "validation_failed",
    "message": "...",
    "details": [{ "field": "name", "code": "required", "message": "..." }],
    "correlation_id": "...",
    "doc_url": "https://docs.proven.example/errors/validation_failed"
  }
}
```

| Status | Code (examples) |
| --- | --- |
| 400 | `bad_request` |
| 401 | `unauthorized` |
| 403 | `forbidden` |
| 404 | `not_found` |
| 409 | `conflict` |
| 412 | `precondition_failed` |
| 422 | `validation_failed` |
| 429 | `rate_limited` (+ `Retry-After`) |
| 503 | `unavailable` |

Platform: `ApiError` / `ErrorResponse`. Modules map domain errors → `AppError` (validation → 422).

## Validation

Use `ValidationReport`, `require_non_empty`, `require_uuid`, or `AppError::Validation { details }`.
Domain invariants stay in modules; wire checks happen at the edge.

## Authentication

| Mode | Transport |
| --- | --- |
| Primary (future) | `Authorization: Bearer <JWT>` |
| Interim DX | `X-Proven-Tenant-Id` + `X-Proven-User-Id` |
| Integrations (future) | `X-Api-Key` |

Public: health, readiness, metrics, OpenAPI/docs. Config: `PROVEN_ENFORCE_AUTHN` (default on in production).

## Authorization

Call Core via `require_permission` / `AuthzApi`. Never trust client permission claims.

## Rate limits

Headers: `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`.

| Env | Default |
| --- | --- |
| `PROVEN_RATE_LIMIT_PER_MINUTE` | `600` |
| `PROVEN_RATE_LIMIT_ENABLED` | `true` |

## Health / metrics / docs

| Surface | Path |
| --- | --- |
| Liveness | `/health`, `/healthz` |
| Readiness | `/readyz` |
| API health | `/api/v1/health`, `/api/v1/health/db`, `/api/v1/health/temporal` |
| Metrics | `/metrics` |
| Swagger | `/docs` |
| Redoc | `/redoc` |
| OpenAPI | `/api-docs/openapi.json`, `/api/v1/openapi.json` |

## Handler checklist (new endpoints)

1. Mount under `/api/v1/<module>/...`
2. Return `DataEnvelope` / `ListEnvelope` (not bare JSON)
3. Parse lists with `ListQuery` + filter whitelist
4. Map failures through `AppError` → nested `{ error }`
5. Annotate with utoipa; security schemes already on the host doc
6. Rely on host AuthN / rate-limit / version / correlation layers — do not reimplement

## Tests

```bash
cargo test -p proven-shared
cargo test -p proven-platform
```
