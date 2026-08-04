# proven-projects

**Project Place** module skeleton for Proven (ADR-0009). Owns the construction undertaking
that scopes compliance work. Other modules must consume it via `ProjectsApi` — never by reading
this module's schema.

## Ownership boundary

| Concern | Owner |
| --- | --- |
| Project Place (`ProjectId`, code, name, status, location, prime, client) | **Projects** (this crate) |
| Project membership ACL (who may access) | **Core** `MembershipApi` — Projects orchestrates only |
| Company legal identity | **Core** |
| Safety / inspections / forms | **Not in this skeleton** |
| Equipment assignment authority | Future Equipment module |
| Document binaries / versions | Future Documents module |
| Full project settings API | Deferred (settings table is a placeholder) |

## Skeleton scope

Implemented:

- Project **create** (starts in `Planning`)
- Project **update** (name, description, location, client, dates)
- Project **archive**
- Project **membership** assignment via Core `GrantProjectMembership`

Not implemented: activate/hold/close, areas, subcontractors CRUD beyond create-time prime/client,
required controls, form bindings, document links, equipment requirements, templates, dashboard,
safety features, inspections, forms.

## Public surfaces

| Surface | Location |
| --- | --- |
| In-process trait | `ProjectsApi` |
| HTTP | `/api/v1/projects/*` |
| Events | `proven.projects.v1.*` |
| Schema | `db/migrations/projects/` → PostgreSQL schema `projects` |

## Permissions

`projects.project.read` · `projects.project.create` · `projects.project.manage` — catalogued in
Core; decisions via `AuthzApi`.

## Usage

```rust
use proven_core::CoreModule;
use proven_projects::ProjectsModule;

let core = CoreModule::in_memory();
let projects = ProjectsModule::with_core(core.services);
```

## Design

See [ADR-0009](../../../docs/adr/0009-projects-module.md) and
[PROJECTS_DOMAIN.md](../../../docs/architecture/PROJECTS_DOMAIN.md).
