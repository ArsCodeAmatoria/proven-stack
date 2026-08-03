# Proven — Monorepo Repository Plan

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Repository Structure & Engineering Plan |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Staff Engineering |
| **Audience** | Engineering, DevEx, Security, Product |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [PRD](../PRD.md), [Domain Model](./DOMAIN_MODEL.md), [System Architecture](./SYSTEM_ARCHITECTURE.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document defines the **complete GitHub monorepo plan** for Proven.

It specifies folder layout, applications, Rust crates, shared libraries, documentation, CI/CD, testing, scripts, assets, configuration, Docker, branch strategy, versioning, labels, issue/PR templates, release strategy, and coding standards.

**This is a plan only.** It does not generate implementation source code, workflows, or template file bodies beyond structural specification.

---

## 2. Monorepo Goals

1. **One product, one repo** — web, API, workers, workflows contracts, docs, and infra-as-config live together.
2. **Module-aligned ownership** — directory boundaries mirror bounded contexts from the Domain Model.
3. **Independent evolution** — path filters in CI allow targeted builds/tests without breaking encapsulation.
4. **Clear deployables** — few runtime applications; many libraries/modules.
5. **API-first contracts** — shared OpenAPI/event schemas versioned in-repo.
6. **Security by default** — secrets never committed; templates and standards enforce review hygiene.
7. **Docs as product** — PRD, domain, and system architecture remain first-class.

---

## 3. Top-Level Folder Structure

```text
proven-stack/
├── .github/
│   ├── workflows/
│   ├── ISSUE_TEMPLATE/
│   ├── PULL_REQUEST_TEMPLATE.md
│   ├── CODEOWNERS
│   ├── dependabot.yml
│   ├── labeler.yml
│   └── CODEOWNERS
├── apps/
│   ├── web/                 # Next.js PWA + admin UI
│   ├── api/                 # Rust Axum binary (modular monolith host)
│   └── workers/             # Go worker binaries
├── crates/                  # Rust workspace crates
│   ├── proven-api/          # binary crate (or thin wrapper; may live under apps/api)
│   ├── proven-platform/     # host wiring, middleware, shared runtime
│   ├── proven-shared/       # minimal shared kernel (IDs, time, errors)
│   ├── modules/             # one crate (or crate family) per bounded context
│   └── ...
├── go/                      # Go module(s) for workers
│   ├── cmd/
│   ├── internal/
│   └── pkg/                 # only if truly shareable across cmds
├── packages/                # JS/TS shared packages (pnpm workspace)
│   ├── ui/                  # design system / shadcn wrappers
│   ├── api-client/          # generated or hand-maintained API client
│   ├── eslint-config/
│   ├── typescript-config/
│   └── pwa-sync/            # offline queue utilities (no business rules)
├── contracts/               # API, events, workflow contract artifacts
│   ├── openapi/
│   ├── events/
│   └── temporal/
├── db/                      # database migrations & seeds (by module schema)
│   ├── migrations/
│   └── seeds/
├── deploy/                  # deployment manifests & environment templates
│   ├── fly/
│   ├── vercel/
│   └── cloudflare/
├── docker/
│   ├── Dockerfile.api
│   ├── Dockerfile.workers
│   ├── Dockerfile.web.dev   # optional
│   └── compose/
├── docs/
│   ├── PRD.md
│   ├── architecture/
│   ├── adr/
│   ├── runbooks/
│   └── engineering/
├── scripts/
├── assets/
├── config/
│   ├── default/
│   ├── local/
│   └── examples/
├── tools/                   # repo tooling (codegen, lint helpers)
├── tests/                   # cross-cutting / e2e / contract tests
│   ├── e2e/
│   ├── contract/
│   └── load/
├── AGENTS.md
├── README.md
├── LICENSE
├── Cargo.toml               # Rust workspace root
├── go.work                  # optional Go workspace
├── pnpm-workspace.yaml
├── package.json             # workspace root scripts
├── rust-toolchain.toml
├── .editorconfig
├── .gitignore
├── .dockerignore
├── .env.example
└── Makefile                 # or just/task — thin task runner entrypoints
```

### 3.1 Placement Rules

| Concern | Location |
| --- | --- |
| Deployable UI | `apps/web` |
| Deployable API | `apps/api` (binary) + `crates/*` (logic) |
| Deployable workers | `apps/workers` or `go/cmd/*` |
| Domain modules | `crates/modules/<context>` |
| Shared kernel (minimal) | `crates/proven-shared` |
| TS shared code | `packages/*` |
| Cross-language contracts | `contracts/*` |
| Product/engineering docs | `docs/*` |
| CI/templates/owners | `.github/*` |

---

## 4. Applications

### 4.1 `apps/web` — Next.js

**Role:** Worker mobile-first PWA and desktop-first supervisor/admin experience.

**Suggested layout:**

```text
apps/web/
├── app/                     # App Router routes
├── components/              # app-specific UI (not design system)
├── features/                # UI features mapped loosely to domains
├── lib/                     # web helpers (auth client, query client)
├── public/
├── styles/
├── tests/
├── next.config.ts
├── package.json
├── tsconfig.json
└── README.md
```

**Rules:**

- No business invariants (eligibility, closure rules, signature validity).
- Zod used for form/input shaping only.
- Calls Proven API via `packages/api-client`.
- Offline queue lives in `packages/pwa-sync` or `apps/web` feature layer.

### 4.2 `apps/api` — Rust Axum Host

**Role:** HTTP entrypoint for the modular monolith.

**Suggested layout:**

```text
apps/api/
├── src/
│   └── main.rs             # bootstrap only
├── Cargo.toml              # depends on proven-platform + modules
└── README.md
```

Business logic does **not** accumulate in `main.rs`. Host wiring belongs in `crates/proven-platform`.

### 4.3 `apps/workers` / `go/cmd` — Go Workers

**Role:** Notification delivery, analytics ETL, object post-processing, maintenance jobs.

**Suggested layout:**

```text
go/
├── go.mod
├── cmd/
│   ├── notify-worker/
│   ├── analytics-worker/
│   ├── object-worker/
│   └── maintenance-worker/
├── internal/
│   ├── notify/
│   ├── analytics/
│   ├── objects/
│   ├── natsx/
│   └── config/
└── README.md
```

**Rules:**

- Workers execute I/O and transforms; they do not own domain decisions.
- Configuration via env + `config` examples.
- Each `cmd` is a separately deployable Fly process as needed.

---

## 5. Rust Crates

### 5.1 Workspace Shape

Root `Cargo.toml` defines a workspace members list.

Recommended crates:

| Crate | Purpose |
| --- | --- |
| `proven-shared` | Minimal shared kernel: IDs, timestamps, error envelopes, correlation IDs |
| `proven-platform` | Axum middleware, auth gates, outbox publisher, telemetry, module registration |
| `proven-contracts` | Optional Rust types generated/mirrored from `contracts/` |
| `modules-tenancy` | Tenancy & organization |
| `modules-identity` | Identity & access |
| `modules-projects` | Projects |
| `modules-workforce` | People / crews |
| `modules-safety` | Safety operations |
| `modules-equipment` | Equipment compliance |
| `modules-documents` | Document control |
| `modules-signatures` | Digital evidence |
| `modules-training` | Training & competency |
| `modules-cor-audit` | COR audit readiness |
| `modules-notifications` | Notification records/rules |
| `modules-workflows` | Workflow definition/instance metadata + Temporal client helpers |
| `modules-analytics` | Analytics config / projection hooks (OLTP side) |
| `modules-audit` | Platform audit append API |
| `proven-test-support` | Test fixtures, DB helpers for integration tests |

Naming may use `proven-<context>` instead of `modules-*`; pick one convention and keep it consistent.

### 5.2 Module Crate Internal Layout (Plan)

Each domain crate:

```text
crates/modules/<context>/
├── src/
│   ├── lib.rs
│   ├── domain/
│   ├── application/
│   ├── infrastructure/
│   └── api/                 # route registration for host
├── tests/
├── Cargo.toml
└── README.md                # context ownership + public interface summary
```

### 5.3 Dependency Rules (Enforced by Review + CI Lint Where Possible)

1. `modules-*` may depend on `proven-shared`.
2. `modules-*` must **not** depend on another module’s `domain` or `infrastructure`.
3. Cross-module calls go through public application interfaces re-exported for the host.
4. `proven-platform` depends on modules for composition only at the edges.
5. `apps/api` depends on `proven-platform` (and transitively modules).
6. No crate may import Go or TS packages.

---

## 6. Shared Libraries

### 6.1 TypeScript Packages (`packages/`)

| Package | Purpose |
| --- | --- |
| `packages/ui` | Shared UI primitives (Tailwind + shadcn-based), accessible components |
| `packages/api-client` | Typed client for Axum APIs |
| `packages/typescript-config` | Shared `tsconfig` bases |
| `packages/eslint-config` | Shared ESLint rules |
| `packages/pwa-sync` | Offline mutation queue primitives (transport + idempotency helpers only) |
| `packages/analytics-ui` (optional later) | Chart widgets for dashboards |

### 6.2 Contracts (`contracts/`)

| Path | Purpose |
| --- | --- |
| `contracts/openapi/` | Versioned OpenAPI documents (per surface or monolithic with tags) |
| `contracts/events/` | NATS event JSON schemas / AsyncAPI |
| `contracts/temporal/` | Workflow/activity name catalogs and payload schemas |

Contracts are the **cross-language source of truth**. Code generation may target Rust, Go, and TS in later implementation phases.

### 6.3 What Must Not Be Shared

- Mutable domain models across modules
- SQL schemas across module crates
- “Utility” bags that become dumping grounds for business rules
- Redis-backed permanent stores disguised as shared libs

---

## 7. Documentation

```text
docs/
├── PRD.md
├── architecture/
│   ├── DOMAIN_MODEL.md
│   ├── SYSTEM_ARCHITECTURE.md
│   └── REPOSITORY_PLAN.md          # this document
├── adr/                            # Architecture Decision Records
│   └── NNNN-title.md
├── engineering/
│   ├── coding-standards.md         # summary linking to this plan §17
│   ├── testing-strategy.md
│   ├── local-development.md
│   └── module-playbook.md          # how to add a new bounded context crate
├── runbooks/
│   ├── deploy.md
│   ├── rollback.md
│   ├── outbox-replay.md
│   ├── db-restore.md
│   └── incident-response.md
└── product/                        # optional future product specs
```

Root docs:

- `README.md` — overview, quickstart pointers, repo map
- `AGENTS.md` — AI/engineering agent constitution (already present)

**ADR rule:** Any decision that changes module boundaries, storage authority, auth model, or deploy topology requires an ADR.

---

## 8. CI/CD

### 8.1 Workflow Inventory (Planned)

| Workflow | Trigger | Purpose |
| --- | --- | --- |
| `ci-web.yml` | PRs touching `apps/web/**`, `packages/**` | Lint, typecheck, unit tests, build |
| `ci-api.yml` | PRs touching `crates/**`, `apps/api/**`, `Cargo.*` | `fmt`, `clippy`, tests, build |
| `ci-workers.yml` | PRs touching `go/**` | `fmt`, `vet`, `staticcheck`, tests, build |
| `ci-contracts.yml` | PRs touching `contracts/**` | Schema validation, breaking-change checks |
| `ci-db.yml` | PRs touching `db/**` | Migration lint, dry-run migrate on ephemeral Postgres |
| `ci-docs.yml` | PRs touching `docs/**` | Link check / markdown lint (lightweight) |
| `security.yml` | PR + schedule | Dependency scanners, secret scan, container scan |
| `e2e.yml` | PR (labeled/nightly) or main | Playwright (or equivalent) against compose stack |
| `deploy-staging.yml` | Push to `main` | Deploy web (Vercel), API/workers (Fly), migrate |
| `deploy-prod.yml` | Tag / release | Production deploy with approvals |
| `release.yml` | Manual or tag | Changelog + GitHub Release |

### 8.2 Path Filtering

Use path filters so unrelated changes do not block on full monorepo builds. Always run a **minimal required** set on every PR:

- Secret scan
- CODEOWNERS-covered critical path checks
- Contract break detection if contracts changed

### 8.3 Required Status Checks (Branch Protection)

For `main`:

- API lint/tests (if Rust changed) **or** aggregate `ci-required` job that no-ops cleanly when unaffected
- Web lint/tests (if web/packages changed)
- Workers tests (if go changed)
- Security secret scan
- Migration dry-run (if db changed)

Prefer a single **merge gate workflow** that computes affected projects and fails closed on tool errors.

### 8.4 Deploy Pipeline Shape

```text
PR → CI
main merge → staging migrate + deploy (Vercel + Fly)
vX.Y.Z tag / release → production approvals → migrate → deploy → smoke
```

### 8.5 Tooling Integrations

- **Dependabot** or equivalent for Cargo, npm, Go modules, Actions
- **CODEOWNERS** for modules, security-sensitive paths, contracts, deploy
- Artifact retention for SBOM and test reports

---

## 9. Testing

### 9.1 Test Layers

| Layer | Location | Scope |
| --- | --- | --- |
| Unit | beside code (`crates/**/src`, `packages/**`, `go/internal/**`) | Pure domain/application logic |
| Module integration | `crates/modules/*/tests` | Postgres-backed aggregate tests |
| API integration | `tests/contract`, `apps/api` tests | HTTP + authz + idempotency |
| Worker integration | `go/**` testcontainers/NATS fakes | Delivery & ETL pipelines |
| Contract | `tests/contract` + `contracts/` | OpenAPI/event schema conformance |
| E2E | `tests/e2e` | Critical user journeys (online + offline sync happy path) |
| Load (later) | `tests/load` | API write paths, sync drain |

### 9.2 Testing Principles

1. Domain tests do not require Next.js.
2. Business rules tested in Rust modules, not in React.
3. Workers tested for retry/idempotency/DLQ behavior, not compliance outcomes.
4. Use deterministic fixtures; no reliance on production data.
5. Mark expensive suites (`e2e`, `load`) for selective CI.
6. Every module must have a documented “test pyramid” section in its README over time.

### 9.3 Local Test Stack

Docker Compose profile provides Postgres, Redis, NATS, Temporal (dev), and MinIO/R2-compatible stubs where needed for integration tests.

---

## 10. Scripts

```text
scripts/
├── dev/
│   ├── bootstrap.md                 # documented steps (or shell entry)
│   ├── up-dependencies              # start compose deps
│   └── seed-local
├── db/
│   ├── migrate
│   ├── rollback                     # carefully constrained
│   └── reset-local
├── codegen/
│   ├── openapi-clients
│   └── event-types
├── ci/
│   ├── affected                     # compute changed projects
│   └── check-module-boundaries      # dependency rule checks
└── release/
    ├── changelog
    └── version-bump
```

**Rules:**

- Scripts are thin wrappers over standard tools (`cargo`, `pnpm`, `go`, `fly`, `sqlx`/`goose`, etc.).
- No hidden production mutations from developer laptops without explicit env confirmation.
- Prefer documented Task/Make targets at repo root that call into `scripts/`.

---

## 11. Assets

```text
assets/
├── brand/
│   ├── logo.svg
│   └── wordmark.svg
├── illustrations/              # marketing / empty states (if needed)
├── icons/
└── samples/                    # non-sensitive sample PDFs for local/docs demos
```

**Rules:**

- No customer data, PII, or real audit evidence in-repo.
- Large binaries prefer Git LFS only if absolutely necessary; prefer external design storage for bulky media.
- Web runtime public assets primarily live under `apps/web/public`.

---

## 12. Configuration

```text
config/
├── examples/
│   ├── api.example.toml
│   ├── workers.example.toml
│   └── web.example.env
├── default/                    # non-secret defaults safe to commit
└── local/                      # gitignored overlays (documented, not committed)
```

Also:

- `.env.example` at repo root listing all required variables (no values that are secrets).
- Runtime config for Fly/Vercel in `deploy/` templates.
- Feature flags / module entitlements configured per environment, not hard-coded in clients.

**Secrets:** GitHub Actions secrets, Fly secrets, Vercel env — never committed.

---

## 13. Docker

```text
docker/
├── Dockerfile.api
├── Dockerfile.workers
├── compose/
│   ├── docker-compose.yml           # core dependencies
│   ├── docker-compose.dev.yml       # hot-reload overlays
│   └── docker-compose.ci.yml        # CI service stack
└── README.md
```

### 13.1 Image Strategy

| Image | Base expectations | Deploy target |
| --- | --- | --- |
| API | Multi-stage Rust build, minimal runtime | Fly.io |
| Workers | Multi-stage Go build, minimal runtime | Fly.io |
| Web | Not required for production (Vercel); optional for local parity | Local/dev |

### 13.2 Compose Services (Local)

- PostgreSQL  
- Redis  
- NATS  
- Temporal (dev) + UI (optional)  
- ClickHouse (optional profile)  
- R2-compatible object store stub (optional profile)  
- Mail catcher for notification testing  

API/web may run on host for faster iteration while dependencies run in Compose.

---

## 14. Branch Strategy

### 14.1 Model

**Trunk-based development** with short-lived branches.

| Branch | Purpose | Protection |
| --- | --- | --- |
| `main` | Always releasable trunk | Required reviews + CI |
| `feat/<ticket>-<slug>` | Features | PR into `main` |
| `fix/<ticket>-<slug>` | Bug fixes | PR into `main` |
| `chore/<slug>` | Tooling/docs | PR into `main` |
| `hotfix/<slug>` | Urgent production fix | PR into `main` + accelerated review |
| `release/<x.y>` (optional) | Stabilization if needed | Rare; prefer tags from `main` |

### 14.2 Rules

1. No long-lived feature branches (> a few days without merge/rebase).
2. No direct commits to `main`.
3. Rebase or merge commits allowed; prefer squash merge for feature PRs to keep trunk history readable.
4. Delete branches after merge.
5. Environment promotion is via deploy pipelines, not long-lived `develop`/`staging` git branches (staging deploys from `main`).

### 14.3 Commit Messages

Conventional Commits recommended:

```text
feat(safety): add corrective action due date invariant
fix(identity): revoke sessions on role removal
docs(architecture): update repository plan
chore(ci): speed up clippy cache
```

Scopes should map to modules/apps (`safety`, `web`, `workers`, `contracts`, `db`, …).

---

## 15. Versioning

### 15.1 Product Versioning

- **SemVer** for platform releases: `MAJOR.MINOR.PATCH`
- Git tags: `vX.Y.Z`
- GitHub Releases correspond 1:1 with production-deployable tags

### 15.2 What Increments Mean

| Change | Version impact |
| --- | --- |
| Breaking API/event contract | `MAJOR` (or coordinated deprecation window + `MINOR` then later removal) |
| New module capability / backward-compatible API | `MINOR` |
| Bug fix / security patch | `PATCH` |
| Docs/chore only | Usually no tag; included in next release notes |

### 15.3 Component Versions

- Prefer **single product version** for API + web + workers in early life (monorepo release train).
- Contracts may carry explicit schema `version` fields independent of product tag.
- npm packages inside monorepo may remain private/unversioned externally until published.
- Rust crates remain workspace-internal unless published (default: unpublished).

### 15.4 Compatibility Policy

- HTTP APIs support additive changes without major bumps.
- Event schemas are additive; consumers ignore unknown fields.
- Migrations follow expand/contract; API and DB compatibility maintained across rolling deploys.

---

## 16. GitHub Labels

### 16.1 Type

| Label | Color intent | Use |
| --- | --- | --- |
| `type:feature` | green | New capability |
| `type:bug` | red | Defect |
| `type:chore` | gray | Maintenance |
| `type:docs` | blue | Documentation |
| `type:security` | dark red | Security issue |
| `type:refactor` | purple | Restructuring without behavior change |
| `type:perf` | orange | Performance |

### 16.2 Area / Module

| Label | Use |
| --- | --- |
| `area:web` | Next.js |
| `area:api` | Axum host/platform |
| `area:workers` | Go workers |
| `area:contracts` | OpenAPI/events/temporal contracts |
| `area:db` | Migrations |
| `area:ci` | Pipelines |
| `area:docs` | Docs |
| `area:tenancy` | Module |
| `area:identity` | Module |
| `area:projects` | Module |
| `area:workforce` | Module |
| `area:safety` | Module |
| `area:equipment` | Module |
| `area:documents` | Module |
| `area:signatures` | Module |
| `area:training` | Module |
| `area:cor-audit` | Module |
| `area:notifications` | Module |
| `area:workflows` | Module |
| `area:analytics` | Module |
| `area:audit` | Module |
| `area:infra` | Deploy/Docker/Cloud |

### 16.3 Priority & Status

| Label | Use |
| --- | --- |
| `priority:p0` | Immediate |
| `priority:p1` | High |
| `priority:p2` | Normal |
| `priority:p3` | Low |
| `status:blocked` | External dependency |
| `status:needs-design` | Requires ADR/spec |
| `status:ready` | Ready for engineering |
| `status:in-progress` | Active |
| `status:needs-review` | Awaiting review |

### 16.4 Risk / Process

| Label | Use |
| --- | --- |
| `risk:data-migration` | DB changes needing care |
| `risk:breaking-change` | Contract/API break |
| `risk:security-sensitive` | Authz/secrets/PII |
| `needs:product` | Product decision required |
| `good first issue` | Onboarding-friendly |
| `incident` | Production incident follow-up |

Automated labeling via `.github/labeler.yml` on path changes is recommended.

---

## 17. Issue Templates

Planned templates under `.github/ISSUE_TEMPLATE/`:

### 17.1 Bug Report

Sections:

- Summary  
- Environment (web/api/workers versions, tenant if applicable, offline?)  
- Steps to reproduce  
- Expected vs actual  
- Impact (field blocked? audit risk?)  
- Logs/correlation IDs (redact PII)  
- Severity proposal  

### 17.2 Feature Request

Sections:

- Problem statement  
- Proposed outcome  
- Affected personas/modules  
- Acceptance criteria  
- Out of scope  
- Design links / ADR need  

### 17.3 Security Report

Sections:

- **Do not file public security details if policy requires private reporting** — template should point to security contact / private channel  
- Impact summary  
- Affected components  
- Reproduction (private)  

### 17.4 Incident Follow-up

Sections:

- Incident timeline link  
- Customer impact  
- Root cause  
- Action items  
- Runbook updates required  

### 17.5 Chore / Tech Debt

Sections:

- Motivation  
- Area  
- Risk if deferred  
- Proposed approach  

Config file `config.yml` should set default labels and choose template prompts.

---

## 18. Pull Request Templates

Single default `.github/PULL_REQUEST_TEMPLATE.md` with sections:

1. **Summary** — why this change exists  
2. **Type** — feature / fix / chore / docs / security  
3. **Modules touched** — checklist of bounded contexts/apps  
4. **Contract impact** — OpenAPI/events/temporal changes? migration?  
5. **Test plan** — commands run + manual checks  
6. **Offline / field impact** — yes/no and notes  
7. **Security / authz considerations**  
8. **Rollback plan**  
9. **Screenshots** (UI only)  
10. **Checklist**
   - [ ] Tests added/updated  
   - [ ] Docs/ADR updated if required  
   - [ ] No secrets committed  
   - [ ] Business rules not placed in React or Go workers  
   - [ ] Cross-module access uses public interfaces/events/workflows only  
   - [ ] Audit logging considered for compliance-significant actions  

Optional: path-specific templates later (`PULL_REQUEST_TEMPLATE/web.md`, etc.).

---

## 19. Release Strategy

### 19.1 Release Train

1. Continuous integration on `main` → **staging auto-deploy**  
2. Staging validation (smoke + critical e2e)  
3. Cut release candidate via tag `vX.Y.Z` (or `vX.Y.Z-rc.N` if needed)  
4. Production deploy workflow with required reviewers  
5. GitHub Release notes generated from Conventional Commits + manual highlights  
6. Post-release smoke + monitor error budgets  

### 19.2 Hotfix Process

1. Branch `hotfix/*` from the production tag (or `main` if identical)  
2. Accelerated review (security/SRE as needed)  
3. Patch version bump  
4. Deploy production + back-merge to `main` immediately  

### 19.3 Database Releases

- Migrations ship **before** or **with** compatible API (expand first).  
- Breaking contract removals only after consumers migrated.  
- Production migrate gated in deploy workflow; never manually improvised without runbook.

### 19.4 Feature Exposure

- Module entitlements / feature flags for incomplete capabilities.  
- Do not rely on unmerged branches for production hiding.

### 19.5 Artifacts Per Release

- Container images (API, workers) tagged with git SHA and semver  
- SBOM  
- OpenAPI snapshot  
- Changelog  

---

## 20. Coding Standards

### 20.1 Cross-Language Principles

1. Follow `AGENTS.md` and Domain Model boundaries.  
2. Business rules live in owning Rust domain modules.  
3. React validates UX only; Go workers deliver/transform only.  
4. Prefer clarity over cleverness; small composable modules.  
5. Strong typing at boundaries; version contracts.  
6. Every compliance-significant write considers audit + idempotency.  
7. No `unwrap()`/silent error swallow in production paths without justification.  
8. Never commit secrets; use examples only.  
9. Accessibility and mobile-first for worker-facing UI.  
10. Public interfaces documented in module README.

### 20.2 Rust Standards

- `rustfmt` + `clippy` (deny warnings in CI for workspace).  
- Error types explicit; map to stable API error codes at HTTP edge.  
- Module code organized as domain / application / infrastructure.  
- Async where I/O bound; avoid holding DB transactions across external calls.  
- Outbox pattern for event publication.  
- Tests for aggregate invariants mandatory for core domains.

### 20.3 TypeScript / Next.js Standards

- TypeScript strict mode.  
- Shared ESLint config; no `any` without justification.  
- App Router conventions; server/client component boundaries intentional.  
- TanStack Query for server state; no ad-hoc global mutable caches for authz truth.  
- Tailwind + shared UI package; avoid one-off inaccessible components.  
- Zod at form boundaries only.

### 20.4 Go Standards

- `gofmt`, `go vet`, static analysis in CI.  
- `internal/` by default; minimal exported `pkg`.  
- Context propagation on all I/O.  
- Retries with backoff; dead-letter visibility.  
- No domain ownership of Safety/Training/Equipment decisions.

### 20.5 SQL / Migrations Standards

- One migration chain ownership per module schema.  
- Expand/contract; avoid destructive one-step changes.  
- Never create cross-module foreign keys as a coupling mechanism.  
- Seeds only for local/dev; deterministic and non-PII.

### 20.6 Events & APIs

- Past-tense event names; envelopes include tenant, actor, correlation IDs.  
- Additive schema evolution.  
- HTTP resources aligned to module ownership.  
- Idempotency keys on client-originated writes (especially offline sync).

### 20.7 Review Standards

Reviewers check:

- Boundary violations  
- Business logic leakage to web/workers  
- Missing audit/authz  
- Contract compatibility  
- Test adequacy  
- Operational risk (migrations, flags, rollback)

---

## 21. CODEOWNERS Plan

Representative ownership (final GitHub handles TBD):

| Path | Owners |
| --- | --- |
| `/crates/modules/safety/**` | safety module team |
| `/crates/modules/identity/**` | identity/security team |
| `/contracts/**` | platform + API stewards |
| `/db/migrations/**` | platform + module owners |
| `/.github/workflows/**` | DevEx / platform |
| `/deploy/**` | SRE / platform |
| `/docs/architecture/**` | architecture group |
| `/apps/web/**` | frontend team |

Security-sensitive paths (`identity`, `audit`, `deploy`, workflows) require at least one designated reviewer.

---

## 22. Repository Bootstrap Sequence (Plan)

When implementation begins, recommended order:

1. Root workspace files (`Cargo.toml`, `pnpm-workspace.yaml`, `go.mod`, `.gitignore`, `.editorconfig`)  
2. `.github` templates, labels doc, CODEOWNERS stubs  
3. `docs/` (already started) + `README.md`  
4. Docker compose dependencies  
5. `crates/proven-shared` + `proven-platform` skeletons  
6. `apps/api` hello/health  
7. `apps/web` shell + PWA baseline  
8. `go/cmd/*` worker skeletons  
9. `contracts/` initial health/OpenAPI  
10. CI workflows with path filters  
11. First domain module (`tenancy`/`identity`) vertical slice  

This plan document remains the structural authority until ADRs supersede specific sections.

---

## 23. Success Criteria

The monorepo plan succeeds when:

1. A new engineer can locate any bounded context in under one minute.  
2. CI runs primarily affected scopes without false confidence.  
3. Modules can evolve without cross-schema entanglement.  
4. Releases are boring: tag → migrate → deploy → smoke.  
5. Templates steer contributors away from architecture violations.  
6. Docs, contracts, and code stay in one discoverable system.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Staff Engineering | Initial complete monorepo repository plan |

---

*End of Repository Plan*
