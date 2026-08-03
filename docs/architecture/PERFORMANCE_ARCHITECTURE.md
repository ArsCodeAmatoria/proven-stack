# Proven — Performance Architecture & Targets

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Performance Targets & Budgets |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Performance Engineering |
| **Audience** | Frontend, Backend, SRE, Data, Product |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Observability](./OBSERVABILITY_ARCHITECTURE.md), [Frontend Architecture](./FRONTEND_ARCHITECTURE.md), [PWA Architecture](./PWA_ARCHITECTURE.md), [Offline Sync](./OFFLINE_SYNC_ARCHITECTURE.md), [Search](./SEARCH_ARCHITECTURE.md), [Data Warehouse](./DATA_WAREHOUSE_ARCHITECTURE.md), [Deployment](./DEPLOYMENT_ARCHITECTURE.md), [Testing Strategy](./TESTING_STRATEGY.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document defines Proven’s **performance targets**: frontend, backend, database, search, analytics, caching, images, CDN, offline sync, scalability, benchmarking, and **performance budgets**.

**Hard rules**

1. Targets are **SLIs with environments** (field mobile vs office desktop; staging ≠ prod).  
2. **Correctness > speed** — never skip AuthZ, audit, or checksums to “go faster.”  
3. Budgets are **enforced in CI/release** where measurable; tuned from production baselines.  
4. Documentation only — no implementation.

---

## 2. Personas & Contexts

| Context | Network | Device | Priority UX |
| --- | --- | --- | --- |
| **Field worker (PWA)** | 3G–4G, intermittent | Mid-tier phone | My Actions, FLHA, offline |
| **Supervisor mobile** | 4G/Wi‑Fi | Phone | Queues, review |
| **Office desktop** | Broadband | Laptop | Command Center, admin, analytics |
| **API integrations** | Datacenter | N/A | Stable p95, rate limits |

Separate Web Vitals and API budgets for **field** vs **office** entry points.

---

## 3. Frontend Targets

### 3.1 Web Vitals (field entry: `/actions`, login)

| Metric | Target (p75, field) | Notes |
| --- | --- | --- |
| **LCP** | ≤ 2.5s on mid 4G | Shell + primary content |
| **INP** | ≤ 200ms | Tap My Actions / CTA |
| **CLS** | ≤ 0.1 | Stable layout |
| **TTFB (HTML)** | ≤ 800ms | Vercel edge/origin |

Office Command Center: LCP ≤ 3.0s acceptable for denser chrome; charts lazy.

### 3.2 Interaction budgets

| Action | Target |
| --- | --- |
| Navigate tab (cached shell) | ≤ 100ms to paint chrome |
| Open FLHA wizard step | ≤ 150ms UI response |
| Typeahead suggest | ≤ 300ms to first suggestions (API+render) |
| Optimistic mark-read | ≤ 50ms local |

### 3.3 Bundle budgets

| Budget | Limit (illustrative) |
| --- | --- |
| Initial JS (field route) | Tight; defer admin/charts/PDF viewers |
| Per-route async chunk | Prefer < 150–200KB gzipped for wizards |
| Fonts | Subset; swap; avoid blocking |

CI: bundle analyzer on web PRs; fail on unexplained +10% growth to field entry.

---

## 4. Backend (API) Targets

| Class | p95 latency | Notes |
| --- | --- | --- |
| **Auth session validate + light read** | ≤ 100–150ms | Cache-friendly |
| **Standard CRUD read** | ≤ 300ms | Single aggregate |
| **Standard write (field)** | ≤ 500ms–1s | Incl. AuthZ + audit + outbox |
| **List (cursor, 50)** | ≤ 500ms | Scoped queries |
| **Search `/search`** | ≤ 400ms (simple); ≤ 800ms (hybrid) | See §6 |
| **File presign** | ≤ 200ms | |
| **Heavy export start** | ≤ 1s ACK; async job | 202/queued pattern |
| **Guest seal** | ≤ 1s | Hot path |

Availability SLO: align [Observability](./OBSERVABILITY_ARCHITECTURE.md) (e.g. 99.9%).

Error budget excludes client 4xx except surge of 429 indicating saturation.

---

## 5. Database Targets

| Concern | Target / practice |
| --- | --- |
| **OLTP read p95** (app-facing queries) | Contribute to API budgets; hot queries < 50–100ms |
| **Write TX** | Keep short; no external I/O inside TX |
| **Pool saturation** | Alert > 85% |
| **Slow query** | Log/trace > 200ms; index review |
| **Migrations** | Expand online; concurrent indexes for large tables |
| **RLS** | Policies must remain index-friendly (`tenant_id` leading) |
| **Connection** | Size pools per Fly instance; avoid thundering herd |

Analytics **must not** run heavy scans on OLTP—use ClickHouse.

---

## 6. Search Targets

| Mode | p95 | Notes |
| --- | --- | --- |
| **Suggest** | ≤ 150–250ms | Prefix/trgm; Redis cache hit ≪ 50ms |
| **Keyword FTS** | ≤ 400ms | Tenant + AuthZ filter |
| **Hybrid (FTS+vector)** | ≤ 800ms | Early phase; tune |
| **Zero-result rate** | Monitor | UX quality |

Index lag (event → searchable): ≤ 30–60s typical; not on request path.

Cache: short TTL keyed by scope hash ([Search](./SEARCH_ARCHITECTURE.md)).

---

## 7. Analytics Targets

| Class | Target |
| --- | --- |
| **Dashboard tile (rollup)** | p95 ≤ 1–2s |
| **Executive scorecard load** | ≤ 3s |
| **Custom report interactive** | ≤ 5–10s or async |
| **Export job ACK** | ≤ 1s; completion minutes by size |
| **Ingest freshness** | Operational tiles 5–15m; exec ≤ 1h ([Warehouse](./DATA_WAREHOUSE_ARCHITECTURE.md)) |

CH queries: pre-aggregated tables preferred; guard max bytes/rows scanned.

---

## 8. Caching Targets

| Layer | Hit / latency intent |
| --- | --- |
| **CDN (static)** | High hit for icons/JS/CSS; TTFB edge ≪ origin |
| **Redis (API)** | Authz/session assist, search suggest, hot settings — p99 get < 5ms intra-region |
| **TanStack Query** | Soft cache; stale-while-revalidate for lists |
| **SW precache** | Shell instant offline |
| **Negative caching** | Short TTL only |

**Not cached:** sealed proof mutations, AuthZ decisions as sole authority, authenticated JSON on CDN.

Invalidate on grant/version events without global flush storms.

---

## 9. Images & Media

| Concern | Target |
| --- | --- |
| **Capture → IDB persist** | ≤ 200ms feel after shutter (async encode) |
| **Client downscale** | Cap long edge (e.g. 1600–2048px) before upload |
| **Thumb display** | Local blob instant; remote thumb < 300ms on 4G |
| **Upload throughput** | Non-blocking UI; resume multipart for large |
| **AV + Available** | Async; don’t block submit UX beyond binding intent |
| **PDF first page** | Lazy viewer; don’t block LCP |

R2 GET via short presign; CDN **not** for private evidence (security).

---

## 10. CDN

| Asset | Strategy |
| --- | --- |
| Marketing / public static | Cloudflare CDN cache-first |
| Next static `_next/static` | Long-cache hashed assets |
| API JSON | **No CDN cache** |
| Presigned R2 | Direct to R2; short TTL |

Budget: static cache hit ratio high enough that field shell loads from edge.

---

## 11. Offline Sync Targets

| Metric | Target |
| --- | --- |
| **Cold start offline shell** | Usable < 2s from install cache |
| **Draft autosave** | ≤ 100ms to IDB ack (debounced) |
| **Enqueue mutation** | ≤ 50ms local |
| **Drain** | N pending: aim ≤ 500ms–1s per light mutation on good 4G |
| **Photo drain** | Bound by upload size; progress deterministic |
| **Conflict surface** | < 1s after ACK error to banner |
| **Background Sync** | Best-effort; not in SLO |

Correctness (idempotency) overrides drain speed.

---

## 12. Scalability Targets

| Dimension | Design point (illustrative) |
| --- | --- |
| **Tenants** | Thousands; isolate noisy tenants via rate limits |
| **Concurrent field writers / project** | Burst FLHA submits without TX collapse |
| **API horizontal scale** | Linear with Fly count until DB bound |
| **Workers** | Scale notify vs OCR independently |
| **Search** | FTS until OpenSearch cutover criteria met |
| **Analytics** | CH scale independent of OLTP |
| **Fan-out notify** | Queue depth SLO; backpressure |

Capacity tests in [Testing Strategy](./TESTING_STRATEGY.md) load suite validate these annually/pre-peak.

---

## 13. Benchmarking

### 13.1 Cadence

| Bench | When |
| --- | --- |
| **Micro (Rust domain)** | PR optional / main |
| **API k6 smoke** | Nightly staging |
| **API k6 full** | Pre-release |
| **Web Vitals CI** | Lighthouse CI on field routes (lab) |
| **RUM** | Continuous prod (sampled) |
| **Search/CH** | Scheduled |
| **Offline drain** | E2E perf harness |

### 13.2 Method

- Fixed synthetic tenant + datasets (sizes documented).  
- Warm vs cold cache called out.  
- Region-matched clients.  
- Publish trends; gate on severe regressions (e.g. p95 +20%).

### 13.3 Tools (intent)

k6/vegeta, Lighthouse CI, Playwright timing, Grafana latency panels, bundle analyzer.

---

## 14. Performance Budgets (Release Gates)

| Budget | Gate |
| --- | --- |
| Field LCP lab | Fail PR if > target + margin on critical routes |
| Field JS entry size | Fail on unexplained growth past threshold |
| API p95 staging smoke | Fail release if write/read classes exceed |
| Search p95 smoke | Fail release if keyword > budget |
| Zero critical console/network errors in e2e | Fail |
| Image path non-blocking | Manual/e2e assert UI not frozen |

Budgets stored as config alongside CI; adjust via ADR when product complexity grows.

---

## 15. Anti-Patterns

| Avoid | Why |
| --- | --- |
| Unbounded `SELECT *` lists | Latency/memory |
| N+1 queries | API p95 blowups |
| Sync CH queries from OLTP TX | Blocks field writes |
| Huge SSR payloads for field | LCP |
| Caching authenticated JSON on CDN | Security + staleness |
| Fake sealed UX while syncing | Trust |
| Metric labels with tenant_id | Cardinality |

---

## 16. Ownership

| Area | Owner |
| --- | --- |
| Web Vitals / bundles | Frontend |
| API latency | Backend |
| DB / indexes | Backend + SRE |
| Search | Search/platform |
| Analytics / CH | Data |
| CDN / edge | SRE |
| Offline sync | Frontend mobile |
| Budgets in CI | DevEx + Perf |

---

## 17. Success Criteria

1. Field workers meet LCP/INP targets on representative networks.  
2. API write/read classes stay within published p95 budgets.  
3. Search and analytics have explicit, separate latency classes.  
4. Caching and CDN speed safe assets without caching private JSON.  
5. Offline sync feels instant locally and drains predictably online.  
6. Benchmarks and budgets catch regressions before production pain.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Performance Engineering | Targets and budgets |

---

*End of Performance Architecture & Targets*
