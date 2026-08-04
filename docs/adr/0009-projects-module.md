# ADR-0009: Projects Module Skeleton

| Field | Value |
| --- | --- |
| Status | Accepted |
| Date | 2026-08-03 |
| Deciders | Lead Software Engineering |

## Context

Proven needs a **Place** module: the construction undertaking that scopes compliance work
([PROJECTS_DOMAIN.md](../architecture/PROJECTS_DOMAIN.md)). `ProjectId` already exists in
`proven-shared` as a reference-only id used by Core membership, AuthZ project scope, and audit.
Core owns **project membership ACL** (`MembershipApi`); it does **not** own the Project Place
aggregate.

Product requires a first slice that can create, update, and archive projects, and assign workers
via Core membership — without Safety, inspections, forms, equipment assignment, or document
binaries.

## Decision

1. Add crate `crates/modules/proven-projects` with PostgreSQL schema `projects`.
2. Projects is SoR for the **Project Place** aggregate: identity (`ProjectId` minted here), code,
   name, lifecycle status, primary location, prime contractor, and client (company participants).
3. **Worker access** remains Core `ProjectMembership`. Projects **orchestrates**
   `MembershipApi::grant_project_membership` after validating project invariants; it never stores a
   competing ACL table.
4. Public APIs: `ProjectsApi` + HTTP `/api/v1/projects/*`.
5. Permission codes reuse Core-seeded `projects.project.{read,create,manage}`; AuthZ via
   `AuthzApi` (ADR-0003).
6. Events: `proven.projects.v1.*`.
7. **Skeleton scope only:** create, update, archive, membership orchestration. Document
   responsibilities for Workers (via Core), Equipment, Safety, Documents, and Settings as
   deferred — no safety features, inspections, or forms in this crate.

## Consequences

- Creating a project mints `ProjectId` in Projects (not Core).
- Assigning a worker: Projects validates the project → Core `GrantProjectMembership`.
- Arch gates: `proven-platform` may depend on `proven-projects`; `proven-projects` may depend on
  `proven-core` (traits only) + infra — never on Companies/Users schemas.
- Full lifecycle (activate/hold/close), areas, required controls, form bindings, document links,
  equipment requirements, templates, and dashboard projections remain future work.
