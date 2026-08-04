# Database Guidelines

- Migrations: `db/migrations/platform/`, `core/`, `companies/`, `users/`, and `projects/` via sqlx / `proven-migrate`.
- Expand/contract; no cross-schema FKs ([migration strategy](../architecture/DATABASE_MIGRATION_STRATEGY.md)).
- Core owns `core`; Companies owns `companies`; Users owns `users`; Projects owns `projects` (Place keyed by minted `ProjectId`).
- Commands: `just db-migrate`, `just db-seed`, `just db-reset`, `just db-migrate-create <name>`.


