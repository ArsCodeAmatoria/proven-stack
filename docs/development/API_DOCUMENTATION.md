# API Documentation

| Surface | URL |
| --- | --- |
| Swagger UI | `http://127.0.0.1:8080/docs` |
| OpenAPI JSON | `http://127.0.0.1:8080/api-docs/openapi.json` |
| Redoc | `http://127.0.0.1:8080/redoc` |
| Contract snapshot | [`contracts/openapi/openapi.json`](../../contracts/openapi/openapi.json) |

Regenerate contract file:

```bash
just docs
# or: ./scripts/codegen/export-openapi.sh
```

OpenAPI is generated from utoipa (`proven-platform`). Keep `/api/v1` versioned.

Conventions for every endpoint: [REST_API_CONVENTIONS.md](./REST_API_CONVENTIONS.md) (ADR-0013).
Versioned OpenAPI is also at `GET /api/v1/openapi.json`. Security schemes: `bearerAuth`,
`apiKeyAuth`, interim `X-Proven-*` headers.
