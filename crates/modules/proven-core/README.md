# proven-core

Platform foundation module for Proven. **Every other module must consume Core** via public interfaces — never by reading `core.*` tables.

## Owns

Tenancy · companies/orgs · users/sessions · RBAC AuthZ · project membership · teams · file object metadata · audit · settings · feature flags · licensing.

## Public surfaces

| Surface | Location |
| --- | --- |
| In-process traits | `TenancyApi`, `IdentityApi`, `AuthzApi`, `MembershipApi`, `FileApi`, `AuditApi`, `SettingsApi`, `FlagsApi`, `LicenseApi` |
| HTTP | `/api/v1/core/*` |
| Events | `proven.core.v1.*` (`events::CoreEvent`) |
| Schema | `db/migrations/core/` → PostgreSQL schema `core` |

## Design decisions

See [ADR-0001..0004](../../../docs/adr/README.md) and [CORE_DOMAIN.md](../../../docs/architecture/CORE_DOMAIN.md).

## Usage

```rust
use proven_core::{AuthzApi, CoreModule};

let core = CoreModule::in_memory();
// merge `core.clone().router()` into the platform Axum router
```

Persistence: unit tests and current host wiring use the seeded **in-memory** store. SQLx adapters exist under `infrastructure/postgres.rs` for gradual cutover; schema is applied by `proven-migrate`.

## Non-goals

No Projects, Safety, Workforce, Documents, Signatures, or other business modules.
