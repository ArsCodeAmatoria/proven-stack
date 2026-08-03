# Database

PostgreSQL foundation for Proven: migrations, seeds, and pool configuration.

## Layout

```text
db/
├── migrations/
│   └── platform/          # sqlx migrations (metadata + platform schema only)
├── seeds/
│   ├── local/             # developer seed SQL (empty foundation)
│   └── ci/                # CI seed SQL (empty foundation)
└── README.md
```

## Commands

```bash
# Apply migrations (creates public._sqlx_migrations + platform schema)
cargo run -p proven-migrate -- migrate
# or
./scripts/db/migrate.sh

# Run seed profile (no-op until real seed SQL exists)
cargo run -p proven-migrate -- seed local
./scripts/db/seed.sh local
```

## API endpoints

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/api/v1/health/db` | Live pool probe (`SELECT 1`) |
| `GET` | `/api/v1/db/version` | Postgres `version()` + latest `_sqlx_migrations` row |
| `GET` | `/readyz` | Readiness includes live Postgres health |

## Rules

- No business schemas/tables in foundation migrations.
- Only `platform` schema + sqlx migration ledger (`_sqlx_migrations`).
- Expand/contract for future module migrations under `db/migrations/<module>/`.
