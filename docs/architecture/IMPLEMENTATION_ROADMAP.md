# Proven — Complete Implementation Roadmap

| Field | Value |
| --- | --- |
| **Product** | Proven — Construction Compliance Operating System |
| **Document type** | CTO Implementation Roadmap (Executive) |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Chief Technology Officer |
| **Audience** | Executives, Engineering Leadership, Product, Security, SRE |
| **Last updated** | 2026-08-03 |
| **Basis** | Full `docs/architecture/*` foundation corpus + engineering guides |
| **Constraint** | **No application code in this document** — planning only |

---

## 1. Executive Summary

Proven’s architecture is **documented and coherent**: modular monolith (Rust), Next.js PWA, Go I/O workers, Temporal, Postgres + R2 + ClickHouse, Core AuthZ, sealed evidence, offline field sync.

This roadmap converts that architecture into **sequenced milestones** from scaffolding through **MVP** (field-defensible proof) to an **enterprise** platform (COR, analytics depth, integrations, AI, multi-region readiness).

| Horizon | Outcome |
| --- | --- |
| **Now** | Architecture complete; implementation not started |
| **MVP** | Workers complete FLHA/inspections offline with photos & seals; supervisors review; audit & AuthZ solid |
| **Enterprise** | COR readiness, portfolio analytics, deep integrations, AI assistants, hardened multi-tenant ops at scale |

**Guiding principles (non-negotiable)**

1. Business rules only in Rust domains.  
2. Server AuthZ + RLS; sealed evidence immutable.  
3. Small PRs; trunk-based; migrate expand/contract.  
4. Security P0 backlog cleared before production PII ([Security Review](./SECURITY_ARCHITECTURE_REVIEW.md)).

---

## 2. Cross-Cutting Delivery Standards (All Milestones)

| Topic | Standard |
| --- | --- |
| **Git branches** | Trunk-based: `feat|fix|chore|docs|hotfix/<ticket>-<slug>` → PR → `main`. No long-lived `develop`. |
| **PR size** | Prefer **< 400 LOC** meaningful diff; max ~800 LOC unless pure docs/generated. Split vertical slices. |
| **Release strategy** | Continuous deploy **staging** from `main`; **production** via SemVer tag `vX.Y.Z` + approval. Hotfix = patch tag. Feature flags for incomplete surfaces. |
| **Testing** | Per [Testing Strategy](./TESTING_STRATEGY.md): Rust domain first; AuthZ/IDOR; Playwright `@critical` before MVP GA. |
| **Documentation** | Update architecture only when decisions change; ADRs for seams (AuthN adapter, Redis, NATS auth). Module README when crate scaffolds. |
| **Deployment** | Per [Deployment](./DEPLOYMENT_ARCHITECTURE.md): Vercel web, Fly API/workers, Cloudflare edge, separate env data planes. |

---

## 3. Milestone Map (Overview)

| ID | Name | Complexity | Primary outcome |
| --- | --- | --- | --- |
| **M0** | Platform Scaffolding | M | Repo runnable; CI; empty modules |
| **M1** | Identity, AuthZ, Tenancy | L | Login, sessions, RBAC, RLS, audit spine |
| **M2** | Projects & People | M | Places, memberships, worker directory |
| **M3** | Files & Media Pipeline | M | R2 upload, AV, attachments |
| **M4** | Signatures Foundation | L | Packages, internal + guest seal, certificates |
| **M5** | Safety Field MVP (FLHA) | L | FLHA lifecycle + review workflows |
| **M6** | Equipment Pre-Use | M | Assets, readiness, pre-use inspections |
| **M7** | Offline PWA Sync | L | Outbox, photos, Sync Center |
| **M8** | Notifications & Reminders | M | In-app, email, push; Temporal reminders |
| **M9** | Documents & Acknowledgements | M | Controlled docs, publish, ack |
| **M10** | Training Currency | M | Assignments, completions, gaps |
| **M11** | MVP Hardening & GA | M | Security P0, e2e, prod readiness |
| **M12** | COR Audit Readiness | L | Frameworks, gaps, packages |
| **M13** | Analytics & Warehouse | L | ClickHouse KPIs, executive dashboards |
| **M14** | Admin & Search | M | Admin console, FTS global search |
| **M15** | Integrations Pack | L | Teams/WA/Outlook; agency connectors |
| **M16** | AI Assistants | L | RAG, suggestions, human review |
| **M17** | Enterprise Scale & Ops | L | Multi-region posture, SLO maturity, pen test |

Complexity: **S** small · **M** medium · **L** large · **XL** extra-large (team-months).

---

## 4. Milestone Details

---

### M0 — Platform Scaffolding

| Field | Content |
| --- | --- |
| **Objectives** | Monorepo builds; Compose deps; empty Rust/Go/Next shells; CI lint/test smoke; deploy staging skeletons. |
| **Modules** | `apps/web`, `apps/api`, `crates/proven-shared|platform`, stub `proven-core`, `go/cmd` stubs, `db/migrations/platform`, `docker/`, `.github/workflows` |
| **Dependencies** | Architecture docs (done); GitHub org/teams; Fly/Vercel/CF accounts |
| **Acceptance Criteria** | `main` CI green; local `make bootstrap` documented; staging URL healthz; no prod PII |
| **Estimated Complexity** | M |
| **Risks** | Over-building generators; monorepo tooling bikeshedding |
| **Testing Requirements** | CI smoke; empty cargo/pnpm/go test |
| **Documentation Requirements** | DEVELOPMENT.md commands become real; ADR tool choices (migrator, SW lib) |
| **Deployment Goals** | Staging web+API hello-world |
| **Branches** | `chore/scaffold-*` short-lived |
| **PR size** | Multiple small PRs (web / rust / go / ci / docker) |
| **Release strategy** | No prod tag; staging only |

---

### M1 — Identity, AuthZ, Tenancy

| Field | Content |
| --- | --- |
| **Objectives** | Better Auth ↔ Core adapter; JWT/session revoke; RBAC/ABAC; RLS; audit append; MFA path; security P0 auth items. |
| **Modules** | `proven-core`, auth routes, Admin minimal tenant provision workflow |
| **Dependencies** | M0; AuthN/AuthZ/Audit/Security Review docs |
| **Acceptance Criteria** | Login/logout/refresh; cross-tenant IDOR tests fail closed; audit on login & grant; RLS forced |
| **Estimated Complexity** | L |
| **Risks** | Dual identity drift (**P0**); cookie/CSRF mistakes |
| **Testing Requirements** | AuthZ matrix; session revoke before JWT expiry; MFA enroll smoke |
| **Documentation Requirements** | ADR: AuthN adapter; cookie matrix |
| **Deployment Goals** | Staging auth end-to-end behind CF |
| **Branches** | `feat/core-authz-*`, `feat/auth-adapter-*` |
| **PR size** | Slice: sessions → grants → RLS → audit |
| **Release strategy** | Flag `auth.mfa`; staging continuous |

---

### M2 — Projects & People

| Field | Content |
| --- | --- |
| **Objectives** | Project Place lifecycle; memberships; person directory; Command Center / My Actions shells wired to real data. |
| **Modules** | `proven-projects`, `proven-people`, web features projects/people/actions |
| **Dependencies** | M1 |
| **Acceptance Criteria** | Create project; grant membership; list workers scoped by AuthZ; Place overview loads |
| **Estimated Complexity** | M |
| **Risks** | GC/Sub visibility bugs |
| **Testing Requirements** | Membership AuthZ; project isolation API tests |
| **Documentation Requirements** | None beyond API catalog updates |
| **Deployment Goals** | Staging demo tenant with 2 projects |
| **Branches** | `feat/projects-*`, `feat/people-*` |
| **PR size** | Vertical: API + UI per capability |
| **Release strategy** | Staging; flag incomplete admin |

---

### M3 — Files & Media Pipeline

| Field | Content |
| --- | --- |
| **Objectives** | FileObject intent/presign/complete; AV quarantine; thumbnails; R2 keys/lifecycle sweeper. |
| **Modules** | Core Files; `media-worker` / temporal-io activities; upload UI |
| **Dependencies** | M1; R2 buckets staging |
| **Acceptance Criteria** | Upload→Available; malware→Quarantine; AuthZ on download; orphan sweeper |
| **Estimated Complexity** | M |
| **Risks** | AV fail-open (**P0**); orphan cost |
| **Testing Requirements** | Upload integ; quarantine path; presign expiry |
| **Documentation Requirements** | Ops runbook: R2 credentials rotation |
| **Deployment Goals** | Staging R2 private; workers on Fly |
| **Branches** | `feat/files-*`, `feat/media-worker-*` |
| **PR size** | API then worker then UI |
| **Release strategy** | Staging |

---

### M4 — Signatures Foundation

| Field | Content |
| --- | --- |
| **Objectives** | Packages/slots; internal seal; guest magic link; hash pin; evidence certificate; Core audit; Temporal reminders stub. |
| **Modules** | `proven-signatures`, guest routes, certificate PDF activity |
| **Dependencies** | M1, M3 |
| **Acceptance Criteria** | Multi-signer package completes; guest cannot hit admin API; void immutable; certificate verify |
| **Estimated Complexity** | L |
| **Risks** | Token leakage in logs; version skew |
| **Testing Requirements** | Guest scope e2e; sequential slots; hash mismatch deny |
| **Documentation Requirements** | Legal evidence posture reviewed by counsel (light) |
| **Deployment Goals** | Staging guest HTTPS only via CF |
| **Branches** | `feat/signatures-*` |
| **PR size** | Package CRUD → seal → guest → certificate |
| **Release strategy** | Flag guest; staging |

---

### M5 — Safety Field MVP (FLHA)

| Field | Content |
| --- | --- |
| **Objectives** | FLHA create/submit/review/close; hazards/controls; photos; signature package integration; review workflow. |
| **Modules** | `proven-safety`, Temporal `FLHAReviewWorkflow`, web safety wizards, My Actions |
| **Dependencies** | M2, M3, M4 |
| **Acceptance Criteria** | End-to-end FLHA online with photo + seal + audit; supervisor review |
| **Estimated Complexity** | L |
| **Risks** | Scope creep into incidents/permits |
| **Testing Requirements** | Domain invariants; Playwright FLHA journey |
| **Documentation Requirements** | Activity type config notes |
| **Deployment Goals** | Staging field demo day |
| **Branches** | `feat/safety-flha-*` |
| **PR size** | Activity core → submit → review → UI wizard |
| **Release strategy** | Keep incidents out of MVP flag |

---

### M6 — Equipment Pre-Use

| Field | Content |
| --- | --- |
| **Objectives** | Assets; readiness; pre-use inspection; block/ready signals; photos. |
| **Modules** | `proven-equipment`, web equipment features |
| **Dependencies** | M2, M3, M4 (optional seal) |
| **Acceptance Criteria** | Pre-use complete updates readiness; AuthZ project scope |
| **Estimated Complexity** | M |
| **Risks** | Over-building binders/tower packs early |
| **Testing Requirements** | Readiness transition tests; e2e pre-use smoke |
| **Documentation Requirements** | — |
| **Deployment Goals** | Staging |
| **Branches** | `feat/equipment-preuse-*` |
| **PR size** | Asset → inspection → readiness → UI |
| **Release strategy** | Flag advanced cert/binder |

---

### M7 — Offline PWA Sync

| Field | Content |
| --- | --- |
| **Objectives** | Installable PWA; outbox/drafts/media; drain; Sync Center; offline FLHA/inspection; conflict UX; BG Sync best-effort. |
| **Modules** | `packages/pwa-sync`, SW, features/offline, allowlisted mutations |
| **Dependencies** | M5, M6 (consumers); M1 auth refresh behavior |
| **Acceptance Criteria** | Airplane mode FLHA+photo → online ACK; no fake sealed; logout clears IDB |
| **Estimated Complexity** | L |
| **Risks** | Device theft PII; flaky BG Sync reliance |
| **Testing Requirements** | Playwright offline; idempotency chaos |
| **Documentation Requirements** | PWA security ADR (refresh storage) |
| **Deployment Goals** | Staging PWA install Android; iOS guidance |
| **Branches** | `feat/pwa-sync-*` |
| **PR size** | Engine → FLHA wire → inspection wire → Sync UI |
| **Release strategy** | Flag offline seal if not ready |

---

### M8 — Notifications & Reminders

| Field | Content |
| --- | --- |
| **Objectives** | In-app inbox; email; push; prefs/quiet hours; Temporal signature/CA reminders; notify-worker. |
| **Modules** | `proven-notifications`, notify-worker, web notifications |
| **Dependencies** | M1; M4/M5 workflows |
| **Acceptance Criteria** | Assignment email+in-app; quiet hours defer; Critical bypass; delivery retries |
| **Estimated Complexity** | M |
| **Risks** | Provider spam; PII in push |
| **Testing Requirements** | Preference ceilings; idempotent delivery |
| **Documentation Requirements** | — |
| **Deployment Goals** | Staging ESP; push on Android |
| **Branches** | `feat/notifications-*` |
| **PR size** | In-app → email → push → prefs |
| **Release strategy** | Teams/WA later (M15) |

---

### M9 — Documents & Acknowledgements

| Field | Content |
| --- | --- |
| **Objectives** | Controlled docs; version publish; ack campaign; optional sign; approval workflow light. |
| **Modules** | `proven-documents`, DocumentApproval/Ack workflows |
| **Dependencies** | M3, M4, M8 |
| **Acceptance Criteria** | Publish version; workers ack; audit publish; version pin on seal |
| **Estimated Complexity** | M |
| **Risks** | ACL enumeration |
| **Testing Requirements** | Restricted doc AuthZ; publish e2e |
| **Documentation Requirements** | — |
| **Deployment Goals** | Staging |
| **Branches** | `feat/documents-*` |
| **PR size** | Versioning → publish → ack → UI |
| **Release strategy** | Staging → MVP+ |

---

### M10 — Training Currency

| Field | Content |
| --- | --- |
| **Objectives** | Courses; assignments; completions; gaps; expiry reminders. |
| **Modules** | `proven-training`, CompletionExpiry/Renewal workflows |
| **Dependencies** | M2, M8 |
| **Acceptance Criteria** | Assign → complete → gap on expiry; AuthZ |
| **Estimated Complexity** | M |
| **Risks** | Policy complexity |
| **Testing Requirements** | Gap state machine; reminder smoke |
| **Documentation Requirements** | — |
| **Deployment Goals** | Staging |
| **Branches** | `feat/training-*` |
| **PR size** | Course → assignment → completion → UI |
| **Release strategy** | Can slip after MVP GA if needed |

---

### M11 — MVP Hardening & GA

| Field | Content |
| --- | --- |
| **Objectives** | Clear Security Review **P0**; critical e2e green; observability baseline; prod deploy; DSAR/backup runbooks lite; pen-test light or external review scheduled. |
| **Modules** | Cross-cutting; freeze new features |
| **Dependencies** | M0–M8 required; M9–M10 preferred |
| **Acceptance Criteria** | P0 checklist done; `@critical` Playwright pass; staging≈prod topology; `v1.0.0` prod with synthetic then pilot tenant |
| **Estimated Complexity** | M |
| **Risks** | Premature enterprise scope |
| **Testing Requirements** | Full critical pack; load smoke; a11y axe |
| **Documentation Requirements** | Runbooks: incident, backup restore, DSAR draft; privacy notice |
| **Deployment Goals** | **Production pilot** |
| **Branches** | `fix/*`, `chore/sec-*`, `hotfix/*` only |
| **PR size** | Small fixes |
| **Release strategy** | `v1.0.0` tag; patch train `v1.0.x` |

---

### M12 — COR Audit Readiness

| Field | Content |
| --- | --- |
| **Objectives** | Framework packs; mappings; readiness; gaps; evidence package; engagement workflows. |
| **Modules** | `proven-cor`, package render workers |
| **Dependencies** | MVP GA; evidence from Safety/Training/Equipment/Documents/Signatures |
| **Acceptance Criteria** | Score/gaps; package generate AuthZ; no silent score forge |
| **Estimated Complexity** | L |
| **Risks** | Framework variability |
| **Testing Requirements** | Readiness idempotency; package AuthZ |
| **Documentation Requirements** | COR pack versioning notes |
| **Deployment Goals** | Prod flag per tenant |
| **Branches** | `feat/cor-*` |
| **PR size** | Framework → readiness → package → UI |
| **Release strategy** | `v1.x` minor; entitlement-gated |

---

### M13 — Analytics & Warehouse

| Field | Content |
| --- | --- |
| **Objectives** | Event→CH ingest; dims/facts; safety/equipment/training/project KPIs; executive scorecard; exports. |
| **Modules** | `proven-analytics`, analytics-worker, dashboards UI |
| **Dependencies** | Stable domain events from MVP+ |
| **Acceptance Criteria** | Freshness SLOs; AuthZ on queries; export audited |
| **Estimated Complexity** | L |
| **Risks** | OLTP load if mis-queried |
| **Testing Requirements** | Ingest idempotency; query AuthZ |
| **Documentation Requirements** | Metric catalog freeze process |
| **Deployment Goals** | Prod CH; dashboards behind entitlement |
| **Branches** | `feat/analytics-*` |
| **PR size** | Ingest → rollups → API → UI |
| **Release strategy** | Minor releases |

---

### M14 — Admin & Search

| Field | Content |
| --- | --- |
| **Objectives** | Admin facade (branding, API keys, flags); Postgres FTS global search; suggest. |
| **Modules** | `proven-admin`, search projections/indexer |
| **Dependencies** | M1; entity corpus from prior modules |
| **Acceptance Criteria** | Search AuthZ-trimmed; API key hashed; admin step-up |
| **Estimated Complexity** | M |
| **Risks** | Search leakage |
| **Testing Requirements** | Search AuthZ; admin permission tests |
| **Documentation Requirements** | — |
| **Deployment Goals** | Prod |
| **Branches** | `feat/admin-*`, `feat/search-*` |
| **PR size** | Split admin vs search |
| **Release strategy** | Minor |

---

### M15 — Integrations Pack

| Field | Content |
| --- | --- |
| **Objectives** | Integration framework; Teams + WhatsApp notify; Outlook optional; one agency connector pilot. |
| **Modules** | `proven-integrations`, notify adapters, Admin connectors UI |
| **Dependencies** | M8; Security SSRF P0 |
| **Acceptance Criteria** | Signed webhooks; secrets vaulted; SSRF denied; consent for WA |
| **Estimated Complexity** | L |
| **Risks** | Agency API change; over-scope OAuth |
| **Testing Requirements** | Webhook idempotency; SSRF suite |
| **Documentation Requirements** | DPRA per connector; subprocessor update |
| **Deployment Goals** | Prod entitlement per connector |
| **Branches** | `feat/integrations-*` |
| **PR size** | Framework → one connector at a time |
| **Release strategy** | Minor; flag each connector |

---

### M16 — AI Assistants

| Field | Content |
| --- | --- |
| **Objectives** | NL search bridge; hazard suggestions; doc summary; RAG+pgvector; human review queue; FLHA/COR assists. |
| **Modules** | `proven-ai`, embed jobs, review UI |
| **Dependencies** | M14 search; M12 for COR assist; provider DPA |
| **Acceptance Criteria** | No silent SoR writes; AuthZ on RAG; review accept→module API; audit completions |
| **Estimated Complexity** | L |
| **Risks** | Prompt injection; residency |
| **Testing Requirements** | Tool allowlist tests; injection cases; accept path |
| **Documentation Requirements** | AI use policy; model subprocessor |
| **Deployment Goals** | Opt-in tenants only |
| **Branches** | `feat/ai-*` |
| **PR size** | Suggest → RAG → assistant → review |
| **Release strategy** | Entitlement; separate minor |

---

### M17 — Enterprise Scale & Ops

| Field | Content |
| --- | --- |
| **Objectives** | SLO maturity; OpenSearch if needed; multi-region posture; pen test; Object Lock tier; chaos drills; Terraform as needed. |
| **Modules** | Cross-cutting SRE/security |
| **Dependencies** | Production load evidence |
| **Acceptance Criteria** | Pen test remediations; restore drill passed; error budgets enforced; residency story documented |
| **Estimated Complexity** | L |
| **Risks** | Premature multi-region complexity |
| **Testing Requirements** | Load tests; DR restore; security regression |
| **Documentation Requirements** | Enterprise admin guide; DR runbook |
| **Deployment Goals** | Multi-region **readiness** (implement only if sold) |
| **Branches** | `chore/sre-*`, `feat/opensearch-*` as needed |
| **PR size** | Small operational PRs |
| **Release strategy** | `v2.0.0` when enterprise pack marketed |

---

## 5. Complete MVP Plan

### 5.1 MVP Definition (Product)

**MVP = Pilot-ready Field Proof**

A general contractor pilot can:

1. Provision tenant, users, project, workers.  
2. Complete **FLHA** (and **pre-use inspection**) with **photos**, **signatures** (crew/internal; guest optional), online and **offline**.  
3. Supervisors **review** via My Actions / Place.  
4. See **in-app + email** notifications for assignments/remindings.  
5. Rely on **immutable audit** and **AuthZ isolation**.  
6. Run on **production** topology with backups and monitoring.

**Explicitly out of MVP:** COR packages, ClickHouse executive analytics, AI, agency integrations, full document control (nice-to-have if M9 lands), advanced equipment binders, WhatsApp/Teams.

### 5.2 MVP Milestone Sequence

```text
M0 → M1 → M2 → M3 → M4 → M5 → M6 → M7 → M8 → (M9/M10 optional) → M11 GA
```

Parallelism after M1: M2∥ early M3; after M3: M4∥ start M5 types; M6∥M5 late; M7 after M5/M6 APIs stable; M8∥M7 late.

### 5.3 MVP Exit Criteria (Executive Checklist)

| Gate | Criterion |
| --- | --- |
| **Security** | All P0 items in Security Architecture Review closed |
| **Quality** | Playwright `@critical` green on staging & post-prod smoke |
| **Privacy** | Data inventory + DPA draft + privacy notice |
| **Ops** | Backup restore drill; Grafana API/worker dashboards; on-call runbooks |
| **Product** | Pilot success rubric agreed (e.g. 2 weeks field use, ≤X sync conflicts) |
| **Release** | `v1.0.0` tagged; patch process rehearsed |

### 5.4 MVP Team Shape (Indicative)

| Squad | Focus |
| --- | --- |
| Platform | M0–M1, CI/CD, observability |
| Domain | M2–M6 Safety/Equipment/Signatures |
| Client | PWA/offline M7 + UX |
| Ops/Sec | M11 gates |

### 5.5 MVP Timeline Guidance (Planning Only)

Order-of-magnitude for a focused team (not a bid): **~2–4 calendar quarters** to MVP GA depending on headcount and AV/auth complexity—**re-estimate after M1**. Do not commit external dates until M1 AuthN adapter is proven.

---

## 6. Roadmap to Enterprise Proven

### 6.1 Enterprise Definition

**Enterprise Proven** adds what large GCs and crane/civil primes buy next:

| Capability | Milestone |
| --- | --- |
| COR / SECOR readiness & evidence packages | M12 |
| Executive & domain analytics | M13 |
| Admin at scale + enterprise search | M14 |
| Teams/WhatsApp/Outlook + agency APIs | M15 |
| AI assistants with human review | M16 |
| Scale, DR, pen test, residency | M17 |

Plus continuous deepening: incidents/permits, binders, training renewals, document approval chains—ship as minors atop MVP without waiting for M17.

### 6.2 Enterprise Sequencing

```text
MVP GA (v1.0)
  → M12 COR (v1.1–v1.2)
  → M13 Analytics (v1.3)
  → M14 Admin/Search (v1.4)
  → M15 Integrations (v1.5)
  → M16 AI opt-in (v1.6)
  → M17 Enterprise ops (v2.0)
```

Entitlements/license flags gate COR, Analytics, AI, each connector.

### 6.3 Enterprise Acceptance (Executive)

| Gate | Criterion |
| --- | --- |
| COR | Tenant completes prep engagement + package under AuthZ |
| Analytics | Exec scorecard freshness ≤1h; export audited |
| Integrations | One production connector with DPRA |
| AI | Accept/reject metrics reviewed; zero silent writes |
| Trust | External pen test + PIPEDA ops (DSAR/breach) exercised |
| Scale | Load test vs published performance budgets |

### 6.4 Commercial / Technical Alignment

| Sell motion | Technical need |
| --- | --- |
| Field pilot | MVP |
| Safety program standardization | Documents + Training + Notifications maturity |
| Audit season | COR M12 |
| Leadership reporting | Analytics M13 |
| IT enterprise | Integrations + Admin + SSO hardening |
| Differentiation | AI M16 (never before trust gates) |

---

## 7. Dependency Graph (Simplified)

```text
M0
└─ M1 AuthZ/AuthN/Audit/RLS
   ├─ M2 Projects/People
   │   ├─ M5 Safety ──┐
   │   └─ M6 Equipment┤
   ├─ M3 Files ────────┼─ M4 Signatures ─┐
   │                   └─────────────────┼─ M7 Offline
   ├─ M8 Notifications ←─────────────────┘
   ├─ M9 Documents
   └─ M10 Training
        └─ M11 MVP GA
             ├─ M12 COR
             ├─ M13 Analytics
             ├─ M14 Admin/Search
             │     └─ M16 AI
             ├─ M15 Integrations
             └─ M17 Enterprise Ops
```

---

## 8. Risk Register (Program-Level)

| Risk | Impact | Mitigation |
| --- | --- | --- |
| AuthN adapter delay | Blocks all | Timebox M1; spike week-1 |
| Offline complexity | MVP slip | Ship online-first field; flag offline |
| Scope creep (incidents/COR) | Delay GA | MVP charter freeze |
| AV/vendor lag | Uploads blocked | Dual scanner option; fail-closed still |
| Under-testing AuthZ | Breach | P0 IDOR gate |
| Analytics before events stable | Bad KPIs | M13 only post-MVP event freeze |

---

## 9. Investment Posture (Executive)

| Phase | Invest in | Avoid |
| --- | --- | --- |
| Pre-MVP | Platform, AuthZ, Safety, Signatures, Offline | AI, multi-region, OpenSearch |
| Post-MVP | COR, Analytics, Integrations | Rewriting monolith |
| Enterprise | SLO/DR/AI opt-in | Premature microservices |

---

## 10. Closing Directive

1. **Execute M0–M1 immediately** — architecture is sufficient; identity seam is the critical path.  
2. **Charter MVP** as M0–M8 (+M11) with written out-of-scope.  
3. **Hold enterprise epics** behind entitlements and trust gates.  
4. **Keep PR discipline** — small slices, trunk-based, staging continuous, tagged prod.  
5. **Re-forecast** dates only after M1 acceptance criteria are met.

Proven becomes enterprise by **earning trust in the field first**, then layering audit, insight, integration, and AI—without ever moving business authority out of domain modules or weakening sealed evidence.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | CTO | Complete implementation roadmap from architecture corpus |

---

*End of Complete Implementation Roadmap*
