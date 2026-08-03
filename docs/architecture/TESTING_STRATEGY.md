# Proven — Testing Strategy

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Principal QA / Testing Strategy |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Principal QA Architecture |
| **Audience** | Engineering, QA, SRE, Security, Product |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [GitHub Repository Design](../engineering/GITHUB_REPOSITORY.md), [Repository Plan](./REPOSITORY_PLAN.md), [Rust Backend](./RUST_BACKEND_ARCHITECTURE.md), [Go Worker Catalog](./GO_WORKER_CATALOG.md), [Frontend Architecture](./FRONTEND_ARCHITECTURE.md), [Security Architecture](./SECURITY_ARCHITECTURE.md), [Offline Sync](./OFFLINE_SYNC_ARCHITECTURE.md), [Database Migration Strategy](./DATABASE_MIGRATION_STRATEGY.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document defines Proven’s **end-to-end testing strategy**: Rust, Go, Next.js, API, Playwright, integration, performance, security, accessibility, regression, load, CI/CD, coverage, and test data.

**Hard rules**

1. **Business invariants are tested in Rust domain modules**—not in React or Go workers ([AGENTS.md](../../AGENTS.md)).  
2. **AuthZ/IDOR tests are mandatory** for multi-tenant APIs.  
3. **No production customer data** in fixtures; synthetic/anonymized only.  
4. **CI is path-filtered** but release trains run critical cross-cutting suites.  
5. Flaky tests are quarantined with owners—not ignored silently.

**Documentation only — no implementation.**

---

## 2. Quality Goals

| Goal | Measure |
| --- | --- |
| Correctness | Domain invariants, API contracts, offline sync idempotency |
| Security | AuthZ isolation, injection, secret hygiene |
| Reliability | Worker retries, Temporal/activities, migrations |
| Usability | A11y AA on primary flows; field PWA offline happy path |
| Performance | API p95 budgets; Web Vitals on field entry |
| Maintainability | Fast unit feedback; selective e2e |

---

## 3. Test Pyramid

```text
            ┌──────────────┐
            │  Load / Chaos│   rare, scheduled
            ├──────────────┤
            │  E2E (Playwright) │  critical journeys
            ├──────────────┤
            │  Integration / API / Contract │
            ├──────────────┤
            │  Component / Worker unit │
            ├──────────────┤
            │  Domain unit (Rust) │  largest layer
            └──────────────┘
```

Prefer many fast Rust domain tests; fewer brittle UI e2e; load only on schedule/pre-release.

---

## 4. Rust Testing

| Layer | Scope | Location |
| --- | --- | --- |
| **Domain unit** | Aggregates, invariants, state machines | `crates/modules/proven-*/src` |
| **Application** | Use cases with fake ports | same |
| **Infrastructure integ** | SQLx + ephemeral Postgres + RLS | `proven-test-support` |
| **HTTP** | Axum router + AuthZ fixtures | module `tests/` or platform |
| **AuthZ matrix** | Cross-tenant/project IDOR | `proven-core` + each module |
| **Idempotency** | Mutation keys, webhook receipts | modules + integrations |

### 4.1 Rules

- No `unwrap` in production paths without justification; tests may assert errors explicitly.  
- Deterministic clocks/UUIDs via ports.  
- Module tests must not open another module’s schema.  
- Migration tests: empty DB → head ([Migration Strategy](./DATABASE_MIGRATION_STRATEGY.md)).

### 4.2 Commands (intent)

`cargo test -p proven-<module>`; workspace clippy/fmt as gate.

---

## 5. Go Testing

| Layer | Scope |
| --- | --- |
| **Unit** | Parsers, mappers, retry classifier, template render |
| **Provider fakes** | Email/Teams/WA/OCR adapters |
| **Integration** | NATS, R2 stub, CH stub, Temporal test env for activities |
| **Idempotency** | Same job id → one side effect |
| **Chaos** | Kill mid-heartbeat; 429; poison → DLQ |

### 5.1 Rules

- Do **not** assert compliance outcomes (those are Rust).  
- Assert delivery attempt callbacks and artifact checksums.  
- `go test ./...`; staticcheck/vet in CI.

---

## 6. Next.js / Frontend Testing

| Layer | Scope |
| --- | --- |
| **Unit** | View-model mappers, Zod UX schemas, sync queue helpers |
| **Component** | Testing Library: Sync Pill, forms, tables |
| **Feature** | Wizard flows with MSW API mocks |
| **No domain SoR tests** | Do not re-implement Safety invariants in Jest |

### 6.1 Tools

Vitest/Jest + Testing Library + MSW; Playwright for e2e (§8).

---

## 7. API Testing

| Type | Purpose |
| --- | --- |
| **Contract** | OpenAPI conformance (`tests/contract`); consumer-driven checks for `packages/api-client` |
| **Integration API** | Real HTTP against ephemeral stack: auth, CRUD, problem codes |
| **AuthZ suites** | Same identity, wrong tenant/project → 401/403/empty |
| **Idempotency** | Replay `Idempotency-Key` |
| **Validation** | 422 shapes; reject mass assignment |
| **Guest scope** | Guest token cannot hit `/admin` or unrelated packages |

API tests run as Rust integ and/or dedicated contract jobs—not only from UI.

---

## 8. Playwright (E2E)

### 8.1 Critical journeys (must stay green on `main`/nightly)

| Journey | Notes |
| --- | --- |
| Login (password/SSO stub) + home redirect | Auth |
| My Actions → open assignment | Worker UX |
| FLHA draft → photo → submit (online) | Safety |
| Offline FLHA → reconnect sync | PWA |
| Guest sign redeem → seal | Signatures |
| Pre-use inspection | Equipment |
| Document ack (smoke) | Documents |
| Admin deny on mobile warn (smoke) | Admin |

### 8.2 Practices

- Stable `data-testid` sparingly; prefer role/label.  
- Seed via API fixtures—not UI setup sprawl.  
- Trace on failure; redact secrets.  
- Tag `@critical` vs `@extended`.  
- Parallelize with isolated tenants per worker.

Location: `tests/e2e` (monorepo).

---

## 9. Integration Testing

| Scope | Stack |
| --- | --- |
| API + Postgres + RLS | Compose profile |
| Outbox → NATS → worker callback | NATS in Compose |
| Temporal activity + API | Temporal dev |
| File intent → R2 stub → complete → AV fake | MinIO/R2 stub |
| Migrations + seed CI | `ci-db` |

Integration tests prove **wiring**; domain truth still unit-tested in Rust.

---

## 10. Performance Testing

| Focus | Method |
| --- | --- |
| **API p95** | k6/vegeta against staging-like; write paths + search |
| **Web Vitals** | Field vs admin entry (LCP/INP) on preview |
| **Offline drain** | Outbox N items time-to-ACK |
| **CH queries** | Analytics dashboard budgets |
| **Budgets** | Documented per endpoint class; regress in CI selectively |

Perf tests are **non-flaky gated** on schedule or labeled PR—not every PR by default.

---

## 11. Security Testing

| Activity | Cadence |
| --- | --- |
| **SAST** | CodeQL (or equiv) every PR |
| **Secret scan** | Every PR |
| **Dependency SCA** | Dependabot + CI audit |
| **AuthZ/IDOR suite** | PR when `crates/` `/api` touch; full on main |
| **DAST smoke** | Staging scheduled (auth/guest/upload) |
| **Pen test** | Annual / major release |
| **Webhook unsigned** | Integrations tests |
| **Presign abuse** | Expired/wrong key rejected |

Security failures **block merge** for high severity.

---

## 12. Accessibility Testing

| Layer | Practice |
| --- | --- |
| **CI axe** | Smoke on My Actions, login, FLHA wizard, guest sign, Sync Center |
| **Manual** | Keyboard, SR, Site High Contrast, 44px targets |
| **Standard** | WCAG 2.2 AA on primary flows |
| **Motion** | `prefers-reduced-motion` |

A11y defects on `@critical` flows block release.

---

## 13. Regression Testing

| Mechanism | Use |
| --- | --- |
| **Critical Playwright pack** | Every main / release |
| **Module regression packs** | Expanded e2e/API when area changes (path labeler) |
| **Golden files** | PDF/certificate hash fixtures (stable templates) |
| **AI eval goldens** | Template version promotion ([AI Systems](./AI_SYSTEMS_ARCHITECTURE.md)) |
| **Visual** | Optional later for design system; not day-one gate |

Bug fixes require a **reproducing test** when feasible (Rust preferred for domain bugs).

---

## 14. Load Testing

| Scenario | Intent |
| --- | --- |
| Auth login storm | Rate limit / lockout behavior |
| Concurrent FLHA submit | Idempotency + DB |
| Sync drain many clients | API + R2 |
| Notify fan-out | Provider/worker limits |
| Search QPS | FTS health |
| Analytics heavy report | Export/CH path |

Run against **non-prod** with synthetic tenants; monitor error budgets. Never load-test production without SRE approval.

Tooling: k6 (or equiv) in `tests/load`.

---

## 15. CI/CD Mapping

| Workflow / job | Tests |
| --- | --- |
| `ci-rust` | fmt, clippy, unit/integ (affected crates) |
| `ci-go` | vet, unit, staticcheck |
| `ci-web` | lint, typecheck, unit, component |
| `ci-contracts` | OpenAPI/event validate + contract tests |
| `ci-db` | migrate lint + migrate empty |
| `e2e` | Playwright `@critical` on main/nightly/labeled PR |
| `codeql` / secret-scan | Security |
| `container` | Image build smoke |
| `deploy-staging` | Post-deploy smoke e2e subset |
| `deploy-prod` | Smoke after promote |

### 15.1 Gates

- PR: unit + lint + affected integ + AuthZ if API touched.  
- `main`: + critical e2e.  
- Release tag: + extended smoke + migration rehearsal notes.  
- Required checks on protected `main`.

---

## 16. Coverage

| Area | Policy |
| --- | --- |
| **Rust domain** | High coverage on invariants; enforce floor per core modules (e.g. safety, core authz) via CI threshold—tune without gaming |
| **Go** | Meaningful coverage on retry/parsers; not vanity 100% |
| **Frontend** | Focus critical components; don’t chase % on generated UI |
| **E2E** | Coverage of **journeys**, not line % |
| **Reports** | Publish coverage artifacts on main; trend, don’t only gate |

Exclude generated code, bindings, and pure fixtures from denominators.

---

## 17. Test Data

### 17.1 Principles

| Rule | Detail |
| --- | --- |
| **Synthetic** | Factories for tenant, users, projects, workers, assets |
| **No PII** | No real names/emails from customers |
| **Deterministic** | Seeded RNG / fixed UUIDs where needed |
| **Tenant-per-test** | Isolation for parallel e2e |
| **Idempotent cleanup** | Drop schema / unique tenant slug |
| **Reference packs** | Minimal COR/hazard fixtures in `db/seeds/ci` |

### 17.2 Factories

- Rust: test builders in `proven-test-support`.  
- API: admin/service fixtures for Playwright.  
- Avoid UI clicking through full onboarding for every test.

### 17.3 Sensitive scenarios

- Guest tokens generated per test; never committed.  
- MFA: test doubles / bypass flag **only** in test env.  
- Quarantine/malware: EICAR-like safe samples in isolated CI.

---

## 18. Environments

| Env | Testing role |
| --- | --- |
| **Local** | Unit + Compose integ |
| **CI ephemeral** | PR gates |
| **Staging** | E2E, DAST smoke, load (scheduled) |
| **Prod** | Synthetic smoke only; no destructive load |

---

## 19. Ownership & Process

| Area | Owner |
| --- | --- |
| Domain unit quality | Module CODEOWNERS |
| E2E critical pack | QA + Frontend |
| AuthZ suites | Security + Core |
| Load/perf budgets | SRE + Backend |
| A11y | Frontend + Design |
| Flaky quarantine | Requiring fix within SLA |

Definition of Done: tests updated; Docs/ADR if contract changes; migration tests if `db/` touched.

---

## 20. Traceability (Requirements → Tests)

| Risk area | Primary tests |
| --- | --- |
| Tenant isolation | AuthZ integ |
| Offline sync | Playwright offline + Rust idempotency |
| Signatures/guest | E2E guest + API scope |
| Workers/DLQ | Go integ |
| Migrations | ci-db |
| A11y field flows | axe + manual |
| Search AuthZ | API search suites |
| AI accept path | Review queue + no silent write tests |

---

## 21. Success Criteria

1. Critical field journeys are automated and green on every release.  
2. Domain bugs are caught in Rust before UI.  
3. AuthZ regressions cannot merge unnoticed.  
4. CI stays fast via path filters while `main` keeps cross-cutting e2e.  
5. Coverage policies improve risk areas without theatrical percentages.  
6. Test data never exposes real customer information.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Principal QA Architecture | Full testing strategy |

---

*End of Testing Strategy*
