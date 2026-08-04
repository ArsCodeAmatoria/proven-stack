# Users Module

Canonical design: [ADR-0006](../adr/0006-users-profile-module.md).

## Boundary

| Concern | Owner |
| --- | --- |
| Login identity, credentials, sessions, AuthZ grants | **Core** |
| Account profile, kinds, avatar, locale, accessibility, prefs | **Users** (`proven-users`) |
| Workforce Person (trades, employment, assignments views) | **People** (not implemented) |
| Project assignments | **Core ProjectMembership** (not in this module) |
| Guest signing tokens / packages | **Signatures** (not implemented) |

## Supported kinds (profile tags, not RBAC)

Worker · Supervisor · Manager · Safety Coordinator · Administrator · External · Guest

## HTTP

`/api/v1/users/{user_id}/…` — profile, kinds, avatar, locale, accessibility, notification preferences, authentication prefs, signature profile, emergency contacts, settings, audit history.

Flow: invite/activate via Core, then `POST …/profile/ensure`.

## Migrations

`db/migrations/users/` after platform → core → companies.
