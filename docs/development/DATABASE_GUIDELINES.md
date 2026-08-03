# Database Guidelines

- Migrations: `db/migrations/platform/` via sqlx / `proven-migrate`.
- Expand/contract; no cross-schema FKs ([migration strategy](../architecture/DATABASE_MIGRATION_STRATEGY.md)).
- DX-1: **no business schema** — only `platform` + `_sqlx_migrations`.
- Commands: `just db-migrate`, `just db-seed`, `just db-reset`, `just db-migrate-create <name>`.
