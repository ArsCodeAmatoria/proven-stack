# ADR-0004: Core Persistence and Migration Layout

| Field | Value |
| --- | --- |
| Status | Accepted |
| Date | 2026-08-03 |
| Deciders | Lead Software Engineering |

## Context

Foundation already uses sqlx migrations under `db/migrations/platform/`. Core needs its own schema without cross-schema FKs.

## Decision

1. Core DDL and permission seed live in `db/migrations/core/` with unique timestamps.
2. Migrator runs **platform then core** (shared `_sqlx_migrations` ledger; versions must not collide).
3. No PostgreSQL FKs from other schemas into `core.*`; UUID references only.
4. Repository ports in application layer; SQLx adapters in infrastructure. Unit tests use an in-memory repository; integration tests use Postgres when available.
5. File **bytes** are not stored in Postgres — Core stores `file_objects` metadata and storage keys only.

## Consequences

- `proven-migrate` and `scripts/db/migrate.sh` apply both directories.
- Outbox for Core events uses `platform.outbox_messages` (platform-owned transport).
