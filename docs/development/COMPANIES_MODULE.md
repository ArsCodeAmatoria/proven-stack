# Companies Module

Canonical design: [ADR-0005](../adr/0005-companies-profile-module.md).

## Boundary

| Concern | Owner |
| --- | --- |
| Legal company identity (`CompanyId`, type, legal name, activate/deactivate) | **Core** |
| Profile, business units, addresses, contacts, branding, settings | **Companies** (`proven-companies`) |
| Projects / workers / equipment / documents / training / safety resources | Future modules (logically owned by `CompanyId`) — **not implemented here** |

## HTTP

`/api/v1/companies/{company_id}/…` — profile, business-units, addresses, contacts, branding, safety/regional settings, default template pointers, notification defaults, storage.

Register the legal company first via Core (`POST /api/v1/core/companies`), then `POST …/profile/ensure`.

## Permissions

`companies.*` codes live in Core’s catalog; decisions go through `AuthzApi`.

## Migrations

`db/migrations/companies/` applied after platform + core (`just db-migrate`).
