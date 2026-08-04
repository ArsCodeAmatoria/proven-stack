# ADR-0005: Companies Profile Module

| Field | Value |
| --- | --- |
| Status | Accepted |
| Date | 2026-08-03 |
| Deciders | Lead Software Engineering |

## Context

Core owns the legal/operating **Company** aggregate (`core.companies`) used for tenancy, AuthZ company scope, and cross-module `CompanyId` references ([CORE_DOMAIN.md](../architecture/CORE_DOMAIN.md), [REST_API.md](../architecture/REST_API.md)).

Product needs richer company configuration: business units, addresses, contacts, branding, safety/regional defaults, template pointers, notification defaults, and storage configuration. Downstream modules (Projects, People, Equipment, Documents, Training, Safety) treat a Core `CompanyId` as the **owner** of their resources — but those modules are not implemented here.

## Decision

1. Add crate `crates/modules/proven-companies` with PostgreSQL schema `companies`.
2. Core remains SoR for company **identity** (register, type, legal name, activate/deactivate).
3. Companies module is SoR for company **profile & configuration** keyed by `CompanyId` + `TenantId` (UUID refs, **no** cross-schema FK).
4. Public APIs: `CompaniesApi` traits + HTTP `/api/v1/companies/*`.
5. Permission codes: `companies.*` published into Core’s catalog; AuthZ still via `AuthzApi`.
6. Events: `proven.companies.v1.*`.
7. **Do not** implement Projects (or other business modules). Ownership rules are documented and enforced only as invariants inside this module (e.g. profile requires an existing Core company).

## Consequences

- Creating a company: call Core `RegisterCompany`, then Companies `EnsureProfile` / provision profile shell.
- Admin branding in Administration domain may compose tenant branding; **company** branding lives here.
- Business Units are company-scoped (distinct from Core tenant `OrgUnit` tree; optional `org_unit_id` ref).
- Arch gates: `proven-platform` may depend on `proven-companies`; `proven-companies` may depend on `proven-core` (traits only) + infra.
