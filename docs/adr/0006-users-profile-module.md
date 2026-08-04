# ADR-0006: Users Profile Module

| Field | Value |
| --- | --- |
| Status | Accepted |
| Date | 2026-08-03 |
| Deciders | Lead Software Engineering |

## Context

Core owns **User** login identity, credentials/SSO links, sessions, and AuthZ grants ([CORE_DOMAIN.md](../architecture/CORE_DOMAIN.md)). People owns operational **Person** workforce profiles ([PEOPLE_DOMAIN.md](../architecture/PEOPLE_DOMAIN.md)). Product also needs an account-facing **Users** surface: classifications (worker through guest), profile/avatar/locale/accessibility, notification preferences, signature prefs, emergency contacts, settings, and audit history views—without project assignments yet.

## Decision

1. Add `crates/modules/proven-users` with schema `users`.
2. Core remains SoR for authentication identity (`UserId`, invite/activate/lock, credentials, sessions).
3. Users module is SoR for **account profile & preferences** keyed by `UserId` + `TenantId` (no cross-schema FK).
4. **UserKind** classifications (Worker, Supervisor, Manager, SafetyCoordinator, Administrator, External, Guest) are profile tags for UX/directory—not Core RBAC permissions and not People workforce roles. AuthZ remains `AuthzApi` only.
5. **Guest** users are profile shells for guest-capable principals; guest signing tokens stay in Signatures (out of scope).
6. **Authentication** feature = read models / preference hooks over Core identity (MFA preference flags, last methods)—does not store password hashes.
7. **Digital Signature Profile** = user signing preferences/assurance hints—not signature packages.
8. **Audit History** = append-only profile change log in this module + optional query orchestration of Core audit for the actor (no second security SoR).
9. **Do not** implement project assignments (Core `ProjectMembership` / People assignment views later).

## Consequences

- Invite/activate still goes through Core; then `EnsureUserProfile` in Users.
- People module (when built) links `PersonId` ↔ `UserId`; Users does not become workforce SoR.
- Permissions: `users.*` in Core catalog; decisions via `AuthzApi`.
- Arch: `proven-platform` → `proven-users`; `proven-users` → `proven-core` (traits) + infra.
