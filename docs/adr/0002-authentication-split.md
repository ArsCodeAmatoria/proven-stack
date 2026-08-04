# ADR-0002: Authentication Split (Better Auth + Core)

| Field | Value |
| --- | --- |
| Status | Accepted |
| Date | 2026-08-03 |
| Deciders | Lead Software Engineering, Security |

## Context

Human AuthN UX (password, OAuth, MFA plugins, cookies) is provided by Better Auth on Next.js. Core must remain system of record for users, tenant binding, and session revocation.

## Decision

1. **Better Auth** owns login protocols and cookie issuance for the web shell.
2. **Core** owns `User`, `Session` ledger (`sid`), revoke-all, and all AuthZ.
3. JWT claims carry `sub`, `tid`, `sid` — **never** roles or permissions.
4. Guest signing tokens are **out of scope** for Core (Signatures module).

Interim (until the Better Auth ↔ Core adapter lands): non-production HTTP may accept `X-Proven-Tenant-Id` / `X-Proven-User-Id` for Core admin/API smoke tests. Production must use validated JWT/session.

## Consequences

- Dual-identity drift is a P0 risk until the adapter syncs users through Core APIs.
- React must never treat UI permission hints as authoritative.
