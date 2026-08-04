# proven-companies

Company **profile & configuration** module for Proven (ADR-0005). Every other module must
consume it via public interfaces — never by reading this module's schema directly.

## Ownership boundary vs. Core

| | Owns | System of Record for |
| --- | --- | --- |
| `proven-core` | Legal Company identity | `legal_name`, `display_name`, `company_type`, lifecycle `status` (active/deactivated), tenancy/AuthZ scoping. Mints `CompanyId`. |
| `proven-companies` (this crate) | Company profile & configuration, keyed by `CompanyId` + `TenantId` | Business units, addresses, contacts, branding, safety settings, regional settings, default template pointers, notification defaults, storage configuration. |

Creating a company end to end: call Core's `TenancyApi::register_company`, then Companies'
`CompaniesApi::ensure_profile` to provision the profile shell + default config rows. See
[`domain::ownership`](src/domain/ownership.rs) for the full, code-level statement of the
boundary.

## Non-goals

This crate **never** implements or depends on Projects, Workers/People, Equipment, Documents,
Training, Safety incident/inspection resources, or any other business module. Those modules key
off `CompanyId` but live elsewhere — see
[Domain Modules Overview](../../../docs/architecture/DOMAIN_MODULES_OVERVIEW.md) and
[AGENTS.md](../../../AGENTS.md). If a change here would require reaching into another module's
schema, that's a signal to use that module's public API/trait or an event instead.

## Public surfaces

| Surface | Location |
| --- | --- |
| In-process trait | `CompaniesApi` |
| HTTP | `/api/v1/companies/*` |
| Events | `proven.companies.v1.*` (`events::CompaniesEvent`) |
| Schema | `db/migrations/companies/` → PostgreSQL schema `companies` (follow-up; in-memory store is authoritative today) |

## Permissions

Published into Core's catalog (`domain::permissions`), but every AuthZ decision still flows
through `proven_core::AuthzApi` (ADR-0003) — this crate makes zero decisions of its own:

`companies.profile.read` · `companies.profile.manage` · `companies.unit.manage` ·
`companies.address.manage` · `companies.contact.manage` · `companies.branding.manage` ·
`companies.safety_settings.manage` · `companies.regional_settings.manage` ·
`companies.templates.manage` · `companies.notification_defaults.manage` ·
`companies.storage.manage`

## Events

`CompanyProfileEnsured/Updated/Archived`, `BusinessUnitCreated/Updated/Archived`,
`AddressAdded/Updated/Removed`, `ContactAdded/Updated/Removed`, `BrandingUpdated`,
`SafetySettingsUpdated`, `RegionalSettingsUpdated`, `DefaultTemplateUpserted`,
`NotificationDefaultsUpdated`, `StorageConfigurationUpdated` — each published on
`proven.companies.v1.<EventName>`.

## Usage

```rust
use std::sync::Arc;
use proven_companies::CompaniesModule;
use proven_core::CoreModule;

// Unit tests / no-dependency local dev: stub Allow-all AuthZ, no Core wired.
let module = CompaniesModule::in_memory();
let _router = module.router();

// Real wiring: reuse Core's services for both AuthzApi and TenancyApi.
let core = CoreModule::in_memory();
let module = CompaniesModule::with_core(core.services);
```

## Design decisions

See [ADR-0005](../../../docs/adr/0005-companies-profile-module.md) and
[ADR-0001..0004](../../../docs/adr/README.md) for the Core AuthZ/tenancy model this module
builds on.
