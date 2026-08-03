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

OpenAPI is generated from utoipa (`proven-platform`). Keep `/api/v1` versioned; auth examples will expand with Core AuthN adapter.
