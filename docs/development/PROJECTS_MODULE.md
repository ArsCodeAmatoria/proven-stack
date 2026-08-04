# Projects Module

Canonical design: [ADR-0009](../adr/0009-projects-module.md) and
[PROJECTS_DOMAIN.md](../architecture/PROJECTS_DOMAIN.md).

## Boundary

| Concern | Owner |
| --- | --- |
| Project Place (code, name, status, location, prime, client) | **Projects** (`proven-projects`) |
| Worker access / membership ACL | **Core** (`MembershipApi`) — Projects orchestrates |
| Safety, inspections, forms | Not in this skeleton |
| Equipment / Documents SoR | Future modules |

## Skeleton capabilities

| Capability | Status |
| --- | --- |
| Create project | Implemented |
| Update project | Implemented |
| Archive project | Implemented |
| Assign membership | Implemented (via Core) |
| Location (primary) | Implemented (embedded) |
| Prime / Client | Implemented (participants) |
| Settings / Equipment / Safety / Documents APIs | Deferred |

## HTTP

`/api/v1/projects/*`

| Method | Path | Purpose |
| --- | --- | --- |
| `POST` | `/api/v1/projects` | Create |
| `GET` | `/api/v1/projects` | List (`?include_archived=`) |
| `GET` | `/api/v1/projects/mine` | Projects for acting principal (Core membership) |
| `GET` | `/api/v1/projects/{id}` | Get |
| `PATCH` | `/api/v1/projects/{id}` | Update |
| `POST` | `/api/v1/projects/{id}/archive` | Archive |
| `GET` | `/api/v1/projects/{id}/participants` | Company participants |
| `POST` | `/api/v1/projects/{id}/memberships` | Assign worker via Core |

## Permissions

`projects.project.{read,create,manage}` — AuthZ via `AuthzApi`.

## Migrations

`db/migrations/projects/` applied after platform → core → companies → users (`just db-migrate`).

## Hard rules

1. Mint `ProjectId` here; Core only references it.
2. Never store a competing membership ACL — call `MembershipApi`.
3. No safety features, inspections, or forms in this crate.
