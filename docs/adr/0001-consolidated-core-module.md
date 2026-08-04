# ADR-0001: Consolidated Core Module Boundary

| Field | Value |
| --- | --- |
| Status | Accepted |
| Date | 2026-08-03 |
| Deciders | Lead Software Engineering |

## Context

Earlier maps split foundation into `tenancy`, `identity`, and `audit` crates. Core Domain Architecture consolidates these into one Open Host Service so AuthZ, tenancy, and audit cannot drift.

## Decision

Implement a single Rust crate `crates/modules/proven-core` owning schema `core`. Do **not** create separate `proven-tenancy` / `proven-identity` / `proven-audit` crates.

Public consumption is only via published traits (`AuthzApi`, `TenancyApi`, …), HTTP under `/api/v1/core/*`, and `proven.core.v1.*` events.

## Consequences

- One permission catalog and one AuthZ decision path.
- Migrations live under `db/migrations/core/`.
- Older docs that list split crates are superseded by [CORE_DOMAIN.md](../architecture/CORE_DOMAIN.md).
