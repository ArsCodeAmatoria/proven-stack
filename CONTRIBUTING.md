# Contributing to Proven

Thanks for contributing to **Proven** — the Construction Compliance Operating System.

This guide covers how to propose changes. For local setup and day-to-day commands, start with the [Developer Handbook](./docs/development/README.md) (`just setup`). Legacy notes also live in [Development Guide](./docs/engineering/DEVELOPMENT.md). For repository layout, labels, and release process, see [GitHub Repository Design](./docs/engineering/GITHUB_REPOSITORY.md) and [Repository Plan](./docs/architecture/REPOSITORY_PLAN.md).

---

## Before you start

1. Read [AGENTS.md](./AGENTS.md) — hard constraints (modular monolith, domain ownership, no business rules in React or Go workers).  
2. Skim architecture docs for the area you touch (`docs/architecture/`).  
3. Prefer an issue (bug/feature) before large PRs; use `status:needs-design` when an ADR is required.

---

## Development model

- **Trunk-based:** short-lived branches → PR → `main`.  
- **No direct commits to `main`.**  
- Branch names: `feat/…`, `fix/…`, `chore/…`, `docs/…`, `hotfix/…`.  
- Commits: [Conventional Commits](./docs/development/COMMIT_CONVENTIONS.md) with monorepo scopes (`feat(api): …`, `chore(dx): …`).  
- Prefer **squash merge**.

---

## Pull requests

Use the PR template. In every PR:

- Explain **why**, not only what.  
- List **modules/apps** touched.  
- Call out **OpenAPI / events / migrations / Temporal** impact.  
- Include a **test plan**.  
- Note **offline/field** impact if any.  
- Confirm **no secrets** committed.  
- Confirm business rules stay in **Rust domain modules**.  
- Confirm cross-module work uses **public interfaces, events, or Temporal**—not another module’s internals.  
- Consider **audit logging** for compliance-significant actions.

CI must be green: required check **`PR Validation`** ([details](./docs/engineering/CI_AND_BRANCH_PROTECTION.md)). Run locally with `just ci` or `./scripts/ci/check.sh`. See also [PR Process](./docs/development/PULL_REQUEST_PROCESS.md).

### Review expectations

| Change | Extra scrutiny |
| --- | --- |
| AuthN/AuthZ, tenancy, files | Security-sensitive |
| Migrations | Expand/contract; no cross-module FKs |
| Contracts | Additive evolution; consumer impact |
| Worker behavior | Idempotency only; no domain authority |

CODEOWNERS may require specific team approvals by path.

---

## Issues

Use GitHub issue templates:

- **Bug** — steps, expected/actual, correlation ids (redact PII).  
- **Feature** — problem, personas, acceptance criteria.  
- **Chore** — debt with risk if deferred.  
- **Incident** — follow-up from production events.  

**Security vulnerabilities:** do **not** file public issues with exploit detail. Follow [SECURITY.md](./SECURITY.md).

---

## Documentation PRs

Docs-only changes are welcome on `main` via short-lived `docs/…` branches or PRs when review is needed. Keep architecture docs consistent with `AGENTS.md`.

---

## Code of conduct

Be respectful and professional. Harassment or unsafe behavior is not tolerated. (Formal CoC document may be added later; until then, standard open-source civility applies.)

---

## License

Contributions are subject to the repository license (`LICENSE` — TBD). If a CLA is introduced, it will be documented here.
