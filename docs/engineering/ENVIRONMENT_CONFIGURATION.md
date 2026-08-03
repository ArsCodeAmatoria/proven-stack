# Proven — Environment Configuration

| Field | Value |
| --- | --- |
| **Document type** | Engineering guide |
| **Status** | Active |
| **Last updated** | 2026-08-03 |
| **Companion** | [Development](./DEVELOPMENT.md), [Docker Local Development](./DOCKER_LOCAL_DEVELOPMENT.md), [Security Architecture](../architecture/SECURITY_ARCHITECTURE.md) |

---

## 1. Purpose

Typed, fail-fast configuration for **development**, **testing**, and **production**.

| Concern | Behavior |
| --- | --- |
| Configuration loader | Env vars (+ optional `.env` in non-production) |
| Typed configuration | Rust `proven-config`, Go `internal/config`, TS `@proven/config` |
| Secrets validation | Rejects weak / placeholder secrets outside development |
| Startup validation | Bind settings, required endpoints, log policy |
| Missing detection | Lists missing keys and exits non-zero |

No business domain logic lives in these loaders.

---

## 2. Environments

| `PROVEN_ENV` | Aliases | Defaults | Secrets |
| --- | --- | --- | --- |
| `development` | `dev`, `local` | Localhost infra URLs allowed | Weak/dev session secret allowed |
| `testing` | `test` | **No** infra defaults — keys required | Session ≥ 16 chars; no `proven:proven` DB |
| `production` | `prod` | **No** infra defaults — keys required | Session ≥ 32 chars; no localhost; no placeholder DB creds |

Unknown `PROVEN_ENV` values fail at load time.

---

## 3. Files

| Path | Role |
| --- | --- |
| [`.env.example`](../../.env.example) | Master template (committed) |
| [`config/examples/development.env`](../../config/examples/development.env) | Local defaults |
| [`config/examples/testing.env`](../../config/examples/testing.env) | CI / test template |
| [`config/examples/production.env`](../../config/examples/production.env) | Placeholder-only prod shape |
| `.env` / `.env.<env>` | Local overlays (**gitignored**) |

```bash
cp .env.example .env
# or
cp config/examples/development.env .env
```

---

## 4. Loaders

### 4.1 Rust API — `crates/proven-config`

```rust
let config = proven_config::load()?; // fails fast
```

- Loads `.env` then `.env.<environment>` when `PROVEN_ENV` is development/testing.
- `SecretString` redacts values in `Debug` / `Display`.
- Wired from `apps/api` at process start.

### 4.2 Go workers — `go/internal/config`

```go
cfg := config.MustLoad("notify-worker", "8091")
log.Print(cfg.Redacted()) // never prints secrets
```

### 4.3 Web — `@proven/config`

```ts
import { loadWebConfig } from "@proven/config";
const config = loadWebConfig(); // uses process.env
```

Validates `NEXT_PUBLIC_PROVEN_API_URL` / `PROVEN_API_URL`.

---

## 5. Required variables

| Variable | Dev default | Test/Prod |
| --- | --- | --- |
| `PROVEN_ENV` | `development` | required semantic |
| `PROVEN_API_HOST` / `PROVEN_API_PORT` | `0.0.0.0` / `8080` | same defaults OK |
| `DATABASE_URL` | local postgres | **required** |
| `REDIS_URL` | local redis | **required** |
| `NATS_URL` | local nats | **required** |
| `TEMPORAL_ADDRESS` | local temporal | **required** |
| `TEMPORAL_NAMESPACE` | `default` | optional |
| `PROVEN_SESSION_SECRET` | injected weak default | **required** |
| `NEXT_PUBLIC_PROVEN_API_URL` | localhost API | **required** (web) |
| `PROVEN_API_URL` | localhost API | **required** (web; may fall back to public URL) |
| `PROVEN_DB_MAX_CONNECTIONS` | `10` | pool size |
| `PROVEN_DB_MIN_CONNECTIONS` | `1` | pool floor |
| `PROVEN_DB_ACQUIRE_TIMEOUT_SECS` | `5` | pool acquire timeout |
| `PROVEN_DB_IDLE_TIMEOUT_SECS` | `600` | idle connection timeout |
| `PROVEN_DB_MAX_LIFETIME_SECS` | `1800` | max connection lifetime |
| `PROVEN_MIGRATE_ON_START` | `true` (dev) | API applies pending migrations on boot |
| `PROVEN_MIGRATIONS_DIR` | `db/migrations/platform` | sqlx migrations path |
| `BETTER_AUTH_URL` | `http://localhost:3000` | Better Auth base URL |
| `NEXT_PUBLIC_APP_URL` | `http://localhost:3000` | Web app origin (auth fallback) |
| `BETTER_AUTH_SECRET` | falls back to session secret / dev default | AuthN signing (≥ 32 chars; required when `PROVEN_ENV=production`) |
| `PROVEN_SERVICE_NAME` | `proven-api` | OTel/log service name |
| `PROVEN_SERVICE_VERSION` / `GIT_SHA` | `0.1.0` | service version attribute |
| `PROVEN_LOG_JSON` | true in test/prod | structured JSON logs |
| `PROVEN_METRICS_ENABLED` | `true` | expose `/metrics` |
| `PROVEN_OTEL_ENABLED` | true when endpoint set | export OTLP traces |
| `OTEL_EXPORTER_OTLP_ENDPOINT` / `PROVEN_OTEL_ENDPOINT` | empty | Collector base URL (e.g. `http://127.0.0.1:4318`) |
| `PROVEN_OTEL_SAMPLE_RATIO` | `1.0` | head sample ratio |

Worker ports: `PROVEN_WORKER_*_PORT` or `PROVEN_WORKER_PORT`.

---

## 6. Validation rules (summary)

**Missing configuration** — empty/absent required keys →  
`missing required configuration: DATABASE_URL, …`

**Secrets** (non-development) — placeholder DB users (`proven:proven`, `:changeme@`, …), short session secrets, production loopback URLs.

**Startup** — empty host/port/namespace, invalid ports, production `RUST_LOG` containing `trace`.

Secrets are **never** written to logs.

---

## 7. Docker Compose

Compose sets `PROVEN_ENV=development` and injects shared infra URLs via Docker DNS (`postgres`, `redis`, `nats`, `temporal`). See `docker/compose/docker-compose.yml`.

---

## 8. Hard rules

- No secrets in git.  
- Production values only from platform secret stores.  
- Fail closed: misconfiguration prevents process start.
