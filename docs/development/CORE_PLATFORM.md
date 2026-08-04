# Core Platform (developer notes)

Canonical domain design: [`docs/architecture/CORE_DOMAIN.md`](../architecture/CORE_DOMAIN.md).

Decisions: [ADR-0001..0004, ADR-0007](../adr/README.md).

Enterprise RBAC engines (roles, permission overrides, ABAC-ready policies): [ENTERPRISE_RBAC.md](./ENTERPRISE_RBAC.md).

Audit Engine (record, search, export, retention — ADR-0008): [AUDIT_ENGINE.md](./AUDIT_ENGINE.md).

File Management (R2-backed classes, versions, scan hook, links — ADR-0010): [FILE_MANAGEMENT.md](./FILE_MANAGEMENT.md).

## Crate

`crates/modules/proven-core` — public traits + HTTP `/api/v1/core/*`.

Nothing may bypass Core for AuthZ, tenancy, membership, audit append, file object identity, settings, flags, or licensing.

## Interim AuthN

Non-production Core HTTP accepts `X-Proven-Tenant-Id` and `X-Proven-User-Id` (ADR-0002). Production uses JWT/`sid` validation via the Better Auth ↔ Core adapter (next milestone).

## Migrations

```bash
just db-migrate
# applies db/migrations/platform then db/migrations/core
```
