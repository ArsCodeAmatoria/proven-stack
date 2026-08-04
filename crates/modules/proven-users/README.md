# proven-users

Account **profile & preferences** module for Proven (ADR-0006). Every other module must consume
it via public interfaces — never by reading this module's schema directly.

## Ownership boundary vs. Core vs. People

| | Owns | System of Record for |
| --- | --- | --- |
| `proven-core` | Login identity | `UserId`, invite/activate/lock lifecycle, credentials, SSO links, sessions, AuthZ grants. Mints `UserId`. |
| `proven-users` (this crate) | Account profile & preferences, keyed by `UserId` + `TenantId` | `UserKind` classification tags, avatar, locale, accessibility, notification preferences, authentication preference mirror flags, digital signature preferences, emergency contacts, a settings bag, and an append-only profile audit log. |
| `proven-people` (future) | Workforce identity | Operational `Person` profiles — certifications, crew assignments, trade, employment status. Links `PersonId` ↔ `UserId`; never implemented here. |

Onboarding a user end to end: Core's `IdentityApi::invite_user` (then `activate_user`), then
Users' `UsersApi::ensure_profile` to provision the profile shell + default preference rows. See
[`domain::ownership`](src/domain/ownership.rs) for the full, code-level statement of the boundary.

## Supported `UserKind` classifications

`Worker`, `Supervisor`, `Manager`, `SafetyCoordinator`, `Administrator`, `External`, `Guest`. These
are **profile tags for UX/directory filtering only** — never consulted by an AuthZ decision (that
stays exclusively in `proven_core::AuthzApi`, ADR-0003) and never a People workforce role. At most
one kind per user may be marked `is_primary`.

## Non-goals

This crate **never** implements or depends on:

- **Project assignments** — Core's `ProjectMembership` (and any future People-side assignment
  view) remains the only place a user is tied to a project.
- **People / workforce SoR** — Users only stores an optional, unenforced `PersonId` pointer for
  future cross-linking; it never becomes the workforce System of Record.
- **Password storage** — `AuthenticationProfile` holds *preference* mirror flags only (MFA
  preference, last auth method, OAuth link booleans). It never stores password hashes,
  credentials, or session material; Core remains the sole authentication SoR.
- **Guest signing tokens** — a `UserKind::Guest` profile is a shell for a guest-capable
  principal's preferences. The short-lived tokens that let an unauthenticated guest sign a
  document belong to the (future) Signatures module.
- Any other business module (Projects, Equipment, Documents, Training, Safety, Signatures,
  COR audit, etc.) — see
  [Domain Modules Overview](../../../docs/architecture/DOMAIN_MODULES_OVERVIEW.md) and
  [AGENTS.md](../../../AGENTS.md). If a change here would require reaching into another module's
  schema, that's a signal to use that module's public API/trait or an event instead.

## Public surfaces

| Surface | Location |
| --- | --- |
| In-process trait | `UsersApi` |
| HTTP | `/api/v1/users/*` |
| Events | `proven.users.v1.*` (`events::UsersEvent`) |
| Schema | `db/migrations/users/` → PostgreSQL schema `users` (follow-up; in-memory store is authoritative today) |

## Permissions

Published into Core's catalog (`domain::permissions`), but every AuthZ decision still flows
through `proven_core::AuthzApi` (ADR-0003) — this crate makes zero decisions of its own:

`users.profile.read` · `users.profile.manage` · `users.kind.manage` · `users.avatar.manage` ·
`users.preferences.manage` · `users.auth_profile.manage` · `users.signature_profile.manage` ·
`users.emergency_contact.manage` · `users.settings.manage` · `users.audit.read`

Every preference-style mutation (avatar, locale, accessibility, notification, authentication
profile, signature profile, emergency contacts, settings, audit reads) allows **either** an
administrator holding the relevant tenant-wide permission **or** the acting principal managing
their own record via a `GrantScopeType::SelfScope` grant — see
`application::services::authz::authorize_self_or_permission`. Profile lifecycle actions
(`ensure_profile`, `archive_profile`) and `UserKind` assignment remain administrator/system-only.

## Events

`UserProfileEnsured/Updated/Archived`, `UserKindAssigned/Removed`, `AvatarUpdated`,
`LocaleUpdated`, `AccessibilityUpdated`, `NotificationPreferencesUpdated`,
`AuthenticationProfileUpdated`, `DigitalSignatureProfileUpdated`,
`EmergencyContactAdded/Updated/Removed`, `UserSettingUpserted`, `ProfileAuditAppended` — each
published on `proven.users.v1.<EventName>`.

## Usage

```rust
use std::sync::Arc;
use proven_users::UsersModule;
use proven_core::CoreModule;

// Unit tests / no-dependency local dev: stub Allow-all AuthZ, no Core wired.
let module = UsersModule::in_memory();
let _router = module.router();

// Real wiring: reuse Core's services for both AuthzApi and IdentityApi.
let core = CoreModule::in_memory();
let module = UsersModule::with_core(core.services);
```

## Design decisions

See [ADR-0006](../../../docs/adr/0006-users-profile-module.md) and
[ADR-0001..0005](../../../docs/adr/README.md) for the Core AuthZ/tenancy model this module builds
on.
