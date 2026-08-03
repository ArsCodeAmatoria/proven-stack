# Proven — Complete GitHub Repository Design

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | GitHub Monorepo & Engineering Repository Design |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Staff Engineering / DevEx |
| **Audience** | Engineering, Security, SRE, Product |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Repository Plan](../architecture/REPOSITORY_PLAN.md), [System Architecture](../architecture/SYSTEM_ARCHITECTURE.md), [AGENTS.md](../../AGENTS.md), [Contributing](../../CONTRIBUTING.md), [Development](./DEVELOPMENT.md), [Security Policy](../../SECURITY.md) |

---

## 1. Purpose

This document is the **complete GitHub repository design** for Proven: monorepo layout, naming, versioning, release process, GitHub Actions, Docker, future Terraform, issue/PR templates, labels, milestones, CODEOWNERS, Dependabot, security policies, and pointers to contributor docs.

**No application implementation** is specified here beyond structural and process contracts. Detailed folder rationale also lives in [REPOSITORY_PLAN.md](../architecture/REPOSITORY_PLAN.md); this document is the **operational GitHub + EngDocs source of truth**.

---

## 2. Repository Identity

| Item | Value |
| --- | --- |
| **Name** | `proven-stack` |
| **Visibility** | Private (enterprise) until public policy decided |
| **Default branch** | `main` |
| **License** | TBD (document in `LICENSE` when chosen) |
| **Primary language mix** | TypeScript (Next.js), Rust (API), Go (workers), Markdown (docs) |

---

## 3. Monorepo Principles

1. One product, one repo: web, API, workers, contracts, docs, deploy config.  
2. Directory boundaries mirror bounded contexts.  
3. Few deployables; many libraries/modules.  
4. Contracts (`contracts/`) versioned in-repo.  
5. Path-filtered CI; no cross-module internal imports.  
6. Docs are first-class product artifacts.

---

## 4. Complete Directory Layout

```text
proven-stack/
├── .github/
│   ├── workflows/                    # CI/CD (see §10) — design only until implemented
│   ├── ISSUE_TEMPLATE/
│   │   ├── config.yml
│   │   ├── bug_report.md
│   │   ├── feature_request.md
│   │   ├── chore.md
│   │   └── incident.md
│   ├── PULL_REQUEST_TEMPLATE.md
│   ├── CODEOWNERS
│   ├── dependabot.yml                # design §14
│   ├── labeler.yml                   # path → labels
│   └── SECURITY.md.symlink note      # use root SECURITY.md
├── apps/
│   ├── web/                          # Next.js App Router PWA + admin
│   ├── api/                          # Rust Axum host binary entry (thin)
│   └── workers/                      # Go worker process entrypoints (or go/cmd)
├── crates/                           # Rust workspace
│   ├── proven-platform/              # host wiring, middleware
│   ├── proven-shared/                # shared kernel (IDs, errors, time)
│   └── modules/
│       ├── proven-core/
│       ├── proven-projects/
│       ├── proven-people/
│       ├── proven-safety/
│       ├── proven-equipment/
│       ├── proven-documents/
│       ├── proven-signatures/
│       ├── proven-training/
│       ├── proven-cor/
│       ├── proven-notifications/
│       ├── proven-workflows/
│       ├── proven-analytics/
│       └── proven-admin/
├── go/                               # Go module root
│   ├── cmd/
│   │   ├── temporal-io-worker/
│   │   ├── notify-worker/
│   │   ├── analytics-worker/
│   │   └── …
│   ├── internal/
│   └── pkg/                          # rare shared libs only
├── packages/                         # pnpm workspace
│   ├── ui/
│   ├── api-client/
│   ├── pwa-sync/
│   ├── eslint-config/
│   └── typescript-config/
├── contracts/
│   ├── openapi/
│   ├── events/
│   └── temporal/
├── db/
│   ├── migrations/                   # per-schema ownership
│   └── seeds/
├── deploy/
│   ├── fly/
│   ├── vercel/
│   └── cloudflare/
├── docker/
│   ├── Dockerfile.api
│   ├── Dockerfile.workers
│   └── compose/
├── infra/                            # Future Terraform (§11)
│   └── terraform/
│       ├── modules/
│       └── envs/
├── docs/
│   ├── PRD.md
│   ├── architecture/
│   ├── design/
│   ├── ux/
│   ├── adr/
│   ├── runbooks/
│   └── engineering/                  # this guide + DEVELOPMENT.md
├── scripts/
│   ├── dev/
│   ├── db/
│   ├── codegen/
│   ├── ci/
│   └── release/
├── assets/
│   └── brand/
├── config/
│   ├── default/
│   ├── examples/
│   └── local/                        # gitignored
├── tools/
├── tests/
│   ├── e2e/
│   ├── contract/
│   └── load/
├── AGENTS.md
├── README.md
├── CONTRIBUTING.md
├── SECURITY.md
├── LICENSE
├── Cargo.toml
├── rust-toolchain.toml
├── go.work                           # optional
├── pnpm-workspace.yaml
├── package.json
├── Makefile                          # or justfile
├── .editorconfig
├── .gitignore
├── .dockerignore
└── .env.example
```

### 4.1 Placement Rules

| Concern | Location |
| --- | --- |
| User-facing web | `apps/web` |
| HTTP API binary | `apps/api` + `crates/*` |
| Background I/O | `go/cmd/*` |
| Domain logic | `crates/modules/proven-*` only |
| Shared TS UI | `packages/ui` |
| Offline sync primitives | `packages/pwa-sync` |
| OpenAPI / events | `contracts/` |
| SQL migrations | `db/migrations` by schema |
| Human docs | `docs/` + root guides |
| IaC (future) | `infra/terraform` |

---

## 5. Naming Standards

### 5.1 Branches

| Pattern | Example |
| --- | --- |
| `feat/<ticket>-<slug>` | `feat/SAFE-123-flha-draft-sync` |
| `fix/<ticket>-<slug>` | `fix/EQP-45-readiness-race` |
| `chore/<slug>` | `chore/ci-cache-clippy` |
| `docs/<slug>` | `docs/warehouse-architecture` |
| `hotfix/<slug>` | `hotfix/auth-refresh-loop` |

Trunk-based; short-lived; no direct commits to `main`.

### 5.2 Commits

[Conventional Commits](https://www.conventionalcommits.org/):

```text
feat(safety): enforce hazard required before submit
fix(core): revoke sessions on grant removal
docs(engineering): add development guide
chore(deps): bump axum
```

Scopes: `core`, `projects`, `people`, `safety`, `equipment`, `documents`, `signatures`, `training`, `cor`, `notifications`, `workflows`, `analytics`, `admin`, `web`, `api`, `workers`, `contracts`, `db`, `ci`, `infra`.

### 5.3 Packages & Crates

| Kind | Pattern |
| --- | --- |
| Rust module crate | `proven-<context>` |
| TS package | `@proven/<name>` (private) |
| Go module | `github.com/<org>/proven-stack/go` |
| Docker image | `proven-api`, `proven-workers` |
| Workflow id prefix | See Temporal workflows doc |
| Event subject | `proven.<module>.v1.<EventName>` |

### 5.4 Files & Paths

- kebab-case for dirs in `apps/web` routes where conventional; Rust `snake_case` modules.  
- Migrations: `{utc_timestamp}_{slug}.sql` with schema prefix ownership.  
- ADRs: `docs/adr/NNNN-title.md`.

---

## 6. Versioning

| Artifact | Scheme |
| --- | --- |
| **Product release** | SemVer `MAJOR.MINOR.PATCH`; git tag `vX.Y.Z` |
| **OpenAPI** | URI `/api/v1`; additive fields; breaking → `v2` |
| **Events** | Envelope `event_version`; additive payloads |
| **Mobile/PWA** | Same product version in release notes; SW update separate |
| **Internal crates/packages** | Workspace-private; not independently published initially |

| Change | Bump |
| --- | --- |
| Breaking API/event without deprecation window | MAJOR |
| Compatible capability | MINOR |
| Fix / security patch | PATCH |
| Docs-only | No required tag |

Single release train for API + web + workers early on.

---

## 7. Release Process

```text
PR → main (CI green + review)
  → auto deploy staging
  → smoke / critical e2e
  → tag vX.Y.Z + GitHub Release
  → production deploy (manual approval)
  → post-release monitors + SBOM attach
```

### 7.1 Artifacts

- Container images tagged `:sha` and `:vX.Y.Z`  
- OpenAPI snapshot in Release assets  
- Changelog from Conventional Commits + curated highlights  
- SBOM (e.g. Syft) for API/workers images  

### 7.2 Hotfix

1. `hotfix/*` from production tag or `main`  
2. Accelerated review (security if needed)  
3. PATCH tag + prod deploy  
4. Merge back to `main` immediately  

### 7.3 Database

- Expand/contract migrations; migrate in deploy pipeline  
- Never ad-hoc prod SQL outside runbook  

### 7.4 Feature exposure

- Flags / license entitlements — not long-lived branches  

---

## 8. Issue Templates

Located under `.github/ISSUE_TEMPLATE/` (bodies checked into repo as process docs).

| Template | Use |
| --- | --- |
| **Bug report** | Defects; severity; correlation ids (redact PII) |
| **Feature request** | Problem, personas, acceptance criteria, modules |
| **Chore / tech debt** | Motivation, risk if deferred |
| **Incident follow-up** | Timeline, RCA, actions, runbooks |
| **Security** | Public template points to [SECURITY.md](../../SECURITY.md) — no exploit details in public issues |

`config.yml` enables blank issues carefully or disables them; routes security privately.

---

## 9. Pull Request Template

Default `.github/PULL_REQUEST_TEMPLATE.md` requires:

- Summary (why)  
- Type  
- Modules touched  
- Contract / migration impact  
- Test plan  
- Offline/field impact  
- Security/authz notes  
- Rollback  
- Checklist aligned with `AGENTS.md` (no domain rules in React/Go; no secrets; audit considered)

Prefer **squash merge** to `main`.

---

## 10. GitHub Actions (Design)

| Workflow | Trigger | Purpose |
| --- | --- | --- |
| `ci.yml` | PR + `main` | Lint/test affected paths |
| `ci-rust.yml` | `crates/**`, `apps/api/**`, `Cargo.*` | fmt, clippy, test, SQL if needed |
| `ci-web.yml` | `apps/web/**`, `packages/**` | pnpm lint, typecheck, unit, build |
| `ci-go.yml` | `go/**` | vet, test, staticcheck |
| `ci-contracts.yml` | `contracts/**` | OpenAPI/event validate |
| `ci-docs.yml` | `docs/**` | Link check / markdown lint (optional) |
| `e2e.yml` | `main` / nightly / labeled PR | Playwright critical paths |
| `container.yml` | tag / `main` | Build/push API & workers images |
| `deploy-staging.yml` | `main` | Fly + Vercel staging |
| `deploy-prod.yml` | `v*` tags + approval | Production |
| `codeql.yml` | schedule + PR | SAST |
| `secret-scan.yml` | PR | Gitleaks or equivalent |
| `labeler.yml` | PR | Auto area labels |

### 10.1 CI Rules

- Path filters to minimize cost.  
- Required status checks on `main`.  
- No production secrets on fork PRs.  
- Caches: `pnpm`, `cargo`, Go modules.  

### 10.2 Branch Protection (`main`)

- Require PR  
- Require approvals (CODEOWNERS)  
- Require CI  
- No force push  
- Linear history optional (squash)  

---

## 11. Docker

| Artifact | Role |
| --- | --- |
| `Dockerfile.api` | Multi-stage Rust → minimal runtime → Fly |
| `Dockerfile.workers` | Multi-stage Go → Fly |
| `compose/*.yml` | Local Postgres, Redis, NATS, Temporal, optional ClickHouse/R2 stub |

Web production on **Vercel** (no required prod web image). Compose is for dependencies; API/web may run on host.

Design details: [REPOSITORY_PLAN §13](../architecture/REPOSITORY_PLAN.md).

---

## 12. Terraform (Future)

```text
infra/terraform/
├── modules/          # VPC-less SaaS wrappers, DNS, R2 buckets, alerts
├── envs/
│   ├── staging/
│   └── prod/
└── README.md
```

| Scope (future) | Examples |
| --- | --- |
| Cloudflare | Zones, WAF packs, R2 buckets, Access apps |
| Observability | Dashboards/alerts as code |
| IAM-ish | OIDC roles for GitHub Actions → cloud |

**Not** day-one blocker: start with `deploy/fly`, `deploy/vercel`, `deploy/cloudflare` declarative configs; migrate to Terraform when env drift demands it.

Rules: state in remote backend; no secrets in TF files; `plan` in CI, `apply` gated.

---

## 13. Labels

### Type

`type:feature` · `type:bug` · `type:chore` · `type:docs` · `type:security` · `type:refactor` · `type:perf`

### Area

`area:web` · `area:api` · `area:workers` · `area:contracts` · `area:db` · `area:ci` · `area:docs` · `area:infra` · `area:core` · `area:projects` · `area:people` · `area:safety` · `area:equipment` · `area:documents` · `area:signatures` · `area:training` · `area:cor` · `area:notifications` · `area:workflows` · `area:analytics` · `area:admin`

### Priority / status

`priority:p0`–`p3` · `status:blocked` · `status:needs-design` · `status:ready` · `status:in-progress` · `status:needs-review`

### Risk / process

`risk:data-migration` · `risk:breaking-change` · `risk:security-sensitive` · `needs:product` · `good first issue` · `incident`

Path-based automation via `labeler.yml`.

---

## 14. Milestones

| Pattern | Use |
| --- | --- |
| `v0.1 Foundation` | Docs, scaffolding, CI skeleton |
| `v0.x Platform` | Core auth, projects, files |
| `v1.0 Field MVP` | FLHA, inspections, offline sync P0 |
| Quarterly OKR milestones | Optional product planning |
| Incident milestones | Rare; prefer issues + labels |

Releases use **tags**, not milestones, as deploy truth. Milestones group issues for planning.

---

## 15. CODEOWNERS (Design)

File: `.github/CODEOWNERS`

| Path | Owners (logical teams) |
| --- | --- |
| `*` | `@org/proven-staff` (fallback) |
| `/apps/web/` `/packages/` | `@org/proven-frontend` |
| `/crates/` `/apps/api/` | `@org/proven-backend` |
| `/crates/modules/proven-safety/` | `@org/proven-safety` |
| `/crates/modules/proven-equipment/` | `@org/proven-equipment` |
| `/go/` | `@org/proven-workers` |
| `/contracts/` | `@org/proven-backend` + `@org/proven-frontend` |
| `/db/` | `@org/proven-backend` |
| `/docs/` | `@org/proven-architecture` |
| `/.github/workflows/` | `@org/proven-devex` |
| `/deploy/` `/docker/` `/infra/` | `@org/proven-sre` |
| `/SECURITY.md` `**/SECURITY*` | `@org/proven-security` |

Exact GitHub team slugs are org-specific; update when teams exist. CODEOWNERS reviews required on protected paths.

---

## 16. Dependabot (Design)

`.github/dependabot.yml` (when implemented):

| Ecosystem | Directory | Cadence |
| --- | --- | --- |
| `cargo` | `/` | Weekly |
| `npm` | `/` (pnpm) | Weekly |
| `gomod` | `/go` | Weekly |
| `github-actions` | `/` | Weekly |
| `docker` | `/docker` | Weekly |
| `terraform` | `/infra/terraform` | Weekly (when present) |

Rules: group minor/patch where safe; major bumps manual; security updates prioritized; CI must pass; CODEOWNERS review for `risk:security-sensitive` paths.

---

## 17. Security Policies

Authoritative public policy: root [SECURITY.md](../../SECURITY.md).

Summary:

- Private vulnerability reporting  
- No public exploit PoCs in issues  
- Supported versions = latest release train + current `main` staging  
- Secrets scanning in CI; no secrets in git  
- Align with [Security Architecture](../architecture/SECURITY_ARCHITECTURE.md)  

---

## 18. Contributor & Developer Docs

| Doc | Role |
| --- | --- |
| [README.md](../../README.md) | Product + doc index + quick start pointer |
| [CONTRIBUTING.md](../../CONTRIBUTING.md) | How to contribute, PR rules, DCO/CLA if any |
| [DEVELOPMENT.md](./DEVELOPMENT.md) | Local setup, commands, compose, debugging |
| [AGENTS.md](../../AGENTS.md) | Hard engineering constraints |
| [REPOSITORY_PLAN.md](../architecture/REPOSITORY_PLAN.md) | Deep structural plan |

---

## 19. README Requirements

Root README must include:

1. Brand/logo + one-line product definition  
2. Repo status (foundation vs scaffolded)  
3. Stack table  
4. Documentation index (architecture + engineering)  
5. Quick link to Development guide  
6. Contributing pointer  
7. Security pointer  
8. License  

---

## 20. Development Guide Requirements

See [DEVELOPMENT.md](./DEVELOPMENT.md): prerequisites, bootstrap, run web/api/workers, compose deps, tests, codegen, troubleshooting, offline PWA notes.

---

## 21. Success Criteria

1. New engineers find layout, ownership, and contribution rules without tribal knowledge.  
2. CI design covers Rust, Go, Next.js, contracts, security scans.  
3. Releases are tagged, staged, and approvable to prod.  
4. Templates enforce module boundaries and security hygiene.  
5. Dependabot + CODEOWNERS keep supply chain and reviews intentional.  
6. Terraform has a clear future home without blocking current deploy configs.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Staff Engineering | Complete GitHub repository design |

---

*End of Complete GitHub Repository Design*
