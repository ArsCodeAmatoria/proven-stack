# Domain modules

`proven-core` (platform foundation — tenancy, identity, AuthZ, membership, files, audit,
settings, flags, licensing) is implemented here. See
[Core Domain Architecture](../../docs/architecture/CORE_DOMAIN.md) and
[ADR-0001..0004](../../docs/adr/README.md).

`proven-companies` (company **profile & configuration**) — see
[proven-companies/README.md](proven-companies/README.md) and
[ADR-0005](../../docs/adr/0005-companies-profile-module.md).

`proven-users` (account **profile & preferences**) — see
[proven-users/README.md](proven-users/README.md) and
[ADR-0006](../../docs/adr/0006-users-profile-module.md).

`proven-projects` (**Project Place** skeleton — create, update, archive, membership
orchestration via Core) — see [proven-projects/README.md](proven-projects/README.md) and
[ADR-0009](../../docs/adr/0009-projects-module.md). Worker ACL remains in Core; no safety,
inspections, or forms in this crate.

**All other business modules remain forbidden** (`safety`, `workforce`, `people`,
`equipment`, `documents`, `signatures`, `training`, `cor_audit`, etc.). Do **not** implement
them here. Modules integrate via public APIs / events — never another module's SQL/schema. See
[Domain Modules Overview](../../docs/architecture/DOMAIN_MODULES_OVERVIEW.md) and
[AGENTS.md](../../AGENTS.md).
