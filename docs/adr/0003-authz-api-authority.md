# ADR-0003: Authorization Only Through AuthzApi

| Field | Value |
| --- | --- |
| Status | Accepted |
| Date | 2026-08-03 |
| Deciders | Lead Software Engineering, Security |

## Context

Every compliance module needs a single fail-closed authorization authority.

## Decision

- `AuthzApi` in `proven-core` is the **only** permission decision API.
- No module may implement a parallel ACL or read Core grant tables.
- Redis may cache decisions with TTL; Postgres remains authority.
- Project Membership lives in Core as an access binding; Project lifecycle stays in `projects`.

## Consequences

- All command handlers call `authorize` before mutating.
- Downstream modules register permission codes in Core’s catalog via review; they do not invent side-channel AuthZ.
