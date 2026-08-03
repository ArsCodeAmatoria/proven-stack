# Proven — Deployment & DevOps Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Deployment / DevOps Architecture |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | DevOps / SRE Architecture |
| **Audience** | Engineering, SRE, Security |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [System Architecture](./SYSTEM_ARCHITECTURE.md), [GitHub Repository Design](../engineering/GITHUB_REPOSITORY.md), [Security Architecture](./SECURITY_ARCHITECTURE.md), [R2 Storage](./R2_STORAGE_ARCHITECTURE.md), [Testing Strategy](./TESTING_STRATEGY.md), [Database Migration Strategy](./DATABASE_MIGRATION_STRATEGY.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document designs Proven’s **deployment architecture**: Development, Testing, Staging, Production; **Vercel**, **Fly.io**, **Docker**, **Cloudflare**; secrets, monitoring, logging, backups, rollback, and scaling.

**Hard rules**

1. **Trunk-based:** `main` → staging auto; production via **version tag + approval**.  
2. **Migrate then serve** — expand/contract DB migrations in deploy pipeline ([Migration Strategy](./DATABASE_MIGRATION_STRATEGY.md)).  
3. **No secrets in git** — platform secret stores only.  
4. **Separate data planes** per environment (DB, R2, NATS, Temporal, CH).  
5. Documentation only — no IaC/app implementation in this doc.

---

## 2. Environment Topology

| Environment | Purpose | Deploy source | Data |
| --- | --- | --- | --- |
| **Development** | Local engineer machines | Compose + host processes | Local/ephemeral volumes |
| **Testing (CI)** | PR/ephemeral validation | GitHub Actions | Ephemeral containers; disposable |
| **Staging** | Pre-prod integration, e2e, partner sandboxes | Auto from `main` | Persistent non-prod; synthetic + anonymized |
| **Production** | Paying tenants | Tag `vX.Y.Z` + manual approval | Hardened, backed-up, residency-aware |

Optional **preview** web deployments per PR (Vercel previews) talking to **staging API** or mocked—never prod data.

```text
Developer laptop          GitHub Actions              Cloudflare Edge
  Compose deps      →       CI ephemeral      →         DNS / WAF / CDN
                                   │
                    ┌──────────────┴──────────────┐
                    ▼                             ▼
                 Staging                      Production
         Vercel web + Fly API/workers    Vercel web + Fly API/workers
         Staging Postgres/Redis/...      Prod Postgres/Redis/...
```

---

## 3. Runtime Placement

| Component | Platform | Notes |
| --- | --- | --- |
| **Next.js web/PWA** | **Vercel** | App Router; preview + staging + prod projects |
| **Rust API** | **Fly.io** | `proven-api` Docker image |
| **Go workers** | **Fly.io** | Split binaries (notify, media, temporal-io, …) |
| **Postgres** | Managed Postgres (Fly/other ADR) | OLTP SoR |
| **Redis** | Managed / Fly Redis | Cache only |
| **NATS** | Fly / managed | Event bus |
| **Temporal** | Temporal Cloud or self-host on Fly | Workflows |
| **ClickHouse** | Managed CH | Analytics |
| **R2** | **Cloudflare R2** | Objects |
| **Edge** | **Cloudflare** | DNS, TLS, WAF, bot, CDN (static/marketing) |

---

## 4. Development

| Practice | Design |
| --- | --- |
| **Deps** | `docker/compose` — Postgres, Redis, NATS, Temporal, optional CH/R2 stub, mail catcher |
| **API/Web** | Run on host for fast iteration (`cargo` / `pnpm dev`) against Compose |
| **Workers** | Optional local Go processes |
| **Config** | `.env` / `config/local` from examples—gitignored |
| **Migrations** | `scripts/db/migrate` against local DB |
| **Seeds** | `db/seeds/local` only |
| **No** | Pointing local tools at prod |

See [Development Guide](../engineering/DEVELOPMENT.md).

---

## 5. Testing (CI Environment)

| Practice | Design |
| --- | --- |
| **Ephemeral** | Compose/`docker-compose.ci.yml` or service containers in GHA |
| **Jobs** | Path-filtered lint/unit/integ; `ci-db` migrate; contract tests |
| **E2E** | Playwright against ephemeral or staging ([Testing Strategy](./TESTING_STRATEGY.md)) |
| **Artifacts** | Logs, coverage, Playwright traces (redacted) |
| **Secrets** | GHA environment secrets; OIDC to cloud where possible—no long-lived keys in PRs from forks |

---

## 6. Staging

| Aspect | Design |
| --- | --- |
| **Trigger** | Push/merge to `main` (green CI) |
| **Web** | Vercel staging project |
| **API/Workers** | Fly staging apps (`proven-api-staging`, workers) |
| **Migrate** | Apply pending migrations before/at API deploy |
| **Verify** | Smoke + `@critical` e2e subset post-deploy |
| **Access** | Cloudflare Access / Zero Trust for admin & staging URLs |
| **Data** | Synthetic tenants; refresh from scrubbed dumps optional |
| **Integrations** | Sandbox credentials (Graph, WA, agencies) |

Staging ≈ production topology at smaller scale.

---

## 7. Production

| Aspect | Design |
| --- | --- |
| **Trigger** | GitHub Release / tag `vX.Y.Z` + **required reviewers** |
| **Order** | (1) backup check (2) migrate expand (3) deploy API/workers (4) deploy web (5) smoke (6) monitor |
| **Change window** | Prefer low-traffic for dangerous migrations |
| **Feature flags** | Incomplete features off via Core flags |
| **Access** | Minimal public surface; admin behind strong AuthZ (+ Access optional) |

Hotfix: `hotfix/*` → patch tag → same pipeline accelerated ([GitHub Repository](../engineering/GITHUB_REPOSITORY.md)).

---

## 8. Vercel

| Concern | Design |
| --- | --- |
| **Projects** | `proven-web-staging`, `proven-web-prod` (and PR previews) |
| **Build** | `apps/web` via pnpm monorepo filter |
| **Env** | `NEXT_PUBLIC_API_URL`, auth URLs—**no** provider private keys in client |
| **Regions** | Align with primary user geography |
| **Headers** | Security headers / CSP via config |
| **PWA** | SW assets cached carefully; API calls to Fly origin |
| **Rollback** | Instant promote previous deployment in Vercel |

Web never talks to Temporal/NATS/Postgres directly.

---

## 9. Fly.io

| Concern | Design |
| --- | --- |
| **Apps** | One app per binary (api, notify-worker, media-worker, temporal-io-worker, …) |
| **Images** | `Dockerfile.api`, `Dockerfile.workers` multi-stage; tag `:sha` and `:vX.Y.Z` |
| **Secrets** | `fly secrets` / platform store |
| **Networking** | Private networking for DB/NATS where possible; public HTTPS for API |
| **Health** | `/healthz`, `/readyz` gates rolling deploy |
| **Regions** | Primary + optional read replicas later |
| **Scale** | §15 |
| **Release command** | Optional migrate job machine before traffic |

`deploy/fly/` holds app configs (toml) as documentation/source of truth for ops.

---

## 10. Docker

| Artifact | Use |
| --- | --- |
| `Dockerfile.api` | Rust API → Fly |
| `Dockerfile.workers` | Go workers → Fly (per binary or shared base + cmd) |
| `compose/*.yml` | Local/CI dependencies |
| **Not required** | Prod web image (Vercel) |

Image scan in CI; SBOM on release; non-root user; minimal distroless/runtime base.

---

## 11. Cloudflare

| Capability | Deployment role |
| --- | --- |
| **DNS** | `app.`, `api.`, staging hosts |
| **TLS** | Edge certs; origin to Vercel/Fly |
| **WAF / Bot / DDoS** | Auth, guest sign, webhooks |
| **CDN** | Marketing/static only—not authenticated JSON |
| **R2** | Private object storage |
| **Access** | Staging + optional admin |
| **Rate limit** | Edge auth/redeem paths |

Cloudflare is edge—not application AuthZ.

---

## 12. Secrets

| Class | Where |
| --- | --- |
| DB, Redis, NATS, Temporal, CH | Fly secrets / managed secret store |
| R2 access | Secret store; prefer temporary creds |
| JWT signing keys | KMS / JWKS rotation |
| OAuth client secrets | Per env |
| Provider keys (email, WA, OCR) | Per env |
| Vercel env | Server-only vs `NEXT_PUBLIC_*` split |
| GHA | Environment-scoped; OIDC deploy |

### 12.1 Practices

- Separate staging/prod values always.  
- Rotation runbooks; dual-key JWT overlap.  
- No secrets in logs, images, or `fly.toml` committed values.  
- Break-glass access audited.

---

## 13. Monitoring

| Signal | Examples |
| --- | --- |
| **Golden signals** | Latency, traffic, errors, saturation (API + workers) |
| **Platform** | Fly metrics, Vercel analytics, Cloudflare analytics |
| **App** | Request p95, 5xx rate, AuthZ deny spikes |
| **Workers** | Queue lag, DLQ depth, Temporal backlog, heartbeat timeouts |
| **Data** | Postgres connections, replication lag, R2 errors, CH ingest lag |
| **SLO** | API availability; notify delivery; sync drain (product) |
| **Alerting** | Pager for prod SEV; staging slack-only |

Dashboards per environment; correlate with `correlation_id`.

---

## 14. Logging

| Aspect | Design |
| --- | --- |
| **Format** | Structured JSON (API, workers, edge where available) |
| **Fields** | `correlation_id`, `tenant_id`, `request_id`, `service`, `version` |
| **Sink** | Central log platform (ADR); retain per class |
| **Redaction** | Tokens, passwords, magic links, stroke data |
| **Trace** | OpenTelemetry → tracing backend |
| **Access** | Least privilege; prod logs restricted |

Align with Security / Go / Rust logging rules.

---

## 15. Backups

| System | Policy |
| --- | --- |
| **Postgres** | Automated continuous/PITR; daily snapshots; encrypt; test restore quarterly |
| **R2** | Versioning on evidence prefixes; cross-account replication future |
| **ClickHouse** | Separate backup/snapshot per provider |
| **Temporal** | Provider backup / namespace export per Temporal ops |
| **Redis** | Ephemeral—no backup as SoR |
| **Secrets** | Dual custody / recovery process—not “back up to git” |
| **Config** | IaC/git for non-secret config |

Pre-migrate: confirm backup freshness. Legal hold: retain DB+R2 evidence.

---

## 16. Rollback

| Layer | Strategy |
| --- | --- |
| **Web (Vercel)** | Instant previous deployment |
| **API/Workers (Fly)** | Redeploy previous image tag `:vX.Y.Z-1` |
| **DB** | **Roll forward** preferred; PITR only for catastrophe |
| **Feature** | Flag off new paths without redeploy when possible |
| **Mixed version** | Expand migrations keep old API compatible during rollback window |
| **Contract** | Never roll back past contract consumers without coordination |

Rollback decision tree in runbooks: error rate / SEV → freeze deploys → revert web → revert api → assess DB.

---

## 17. Scaling

| Component | Scale lever |
| --- | --- |
| **Vercel web** | Platform auto; CDN for static |
| **Fly API** | Horizontal VM count; concurrency; region |
| **Workers** | Per-binary scale (notify vs OCR independently) |
| **Temporal workers** | Task queue concurrency |
| **Postgres** | Vertical + read replicas for heavy read/search later |
| **NATS** | Cluster / JetStream limits |
| **R2** | Inherent object scale |
| **CH** | Cluster sizing for analytics |
| **Cloudflare** | Absorbs volumetric attacks |

Autoscale rules: CPU/RPS/queue depth—with cost caps. Isolate noisy tenants via rate limits before infra thrash.

---

## 18. Deploy Pipeline (Happy Path)

```text
PR → CI green → merge main
  → build images + web
  → deploy staging (migrate → fly → vercel)
  → smoke e2e
  → tag vX.Y.Z + approval
  → backup verify
  → migrate prod
  → deploy Fly API/workers (rolling)
  → deploy Vercel prod
  → smoke + monitor error budget
  → publish SBOM/release notes
```

---

## 19. Ownership

| Area | Owner |
| --- | --- |
| Vercel / frontend deploy | Frontend + DevEx |
| Fly / Docker / workers | Backend + SRE |
| Cloudflare / R2 / DNS | SRE + Security |
| DB backups / migrate | SRE + Backend |
| Secrets rotation | Security + SRE |
| On-call alerts | SRE rotation |

---

## 20. Success Criteria

1. Dev → CI → staging → prod path is explicit and repeatable.  
2. Vercel and Fly roles are clear; Docker images are scanned and tagged.  
3. Cloudflare protects edge without replacing AuthZ.  
4. Secrets never ship in git or client bundles.  
5. Monitoring/logging diagnose incidents via correlation ids.  
6. Backups are restore-tested; rollbacks prefer compatible expand/contract.  
7. Scaling is per-service, not a single monolith dial.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | DevOps Architecture | Environments, deploy, ops |

---

*End of Deployment & DevOps Architecture*
