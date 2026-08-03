# Proven — Enterprise Search Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Enterprise Search Architecture |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Search / Platform Architecture |
| **Audience** | Backend, Frontend, Analytics, Security, Product |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [System Architecture](./SYSTEM_ARCHITECTURE.md), [PostgreSQL](./POSTGRESQL_ARCHITECTURE.md), [REST API](./REST_API.md), [Security](./SECURITY_ARCHITECTURE.md), [Analytics](./ANALYTICS_DOMAIN.md), [Core Domain](./CORE_DOMAIN.md), [Frontend](./FRONTEND_ARCHITECTURE.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document defines Proven’s **enterprise search architecture**: unified and type-scoped search across workers, projects, equipment, documents/SWPs, hazards, incidents, training, and analytics surfaces—built first on **PostgreSQL Full Text Search (FTS)**, with a clear path to **OpenSearch** and **pgvector** for scale and natural-language / semantic retrieval.

**Hard rules**

1. **AuthZ is mandatory on every hit** — search never bypasses Core `AuthzApi` / scope filters / RLS.  
2. **Modules own source truth** — search indexes are **projections**, rebuildable from SoR.  
3. **No cross-module SQL joins for indexing authority** — project via events/public APIs into search projections.  
4. **PII/PHI minimization** — medical notes and signature strokes are not full-text indexed.  
5. **v1 = Postgres FTS**; OpenSearch and pgvector are **additive**, not rewrite-the-product.

**Documentation only — no implementation.**

---

## 2. Goals & Non-Goals

### 2.1 Goals

- Fast “Find” on mobile and Command Center global search.  
- Type filters + faceted refinement (project, status, type).  
- Permission-trimmed results (tenant → project → document ACL).  
- Stable ranking that prefers exact codes/tags, then names, then body text.  
- Path to semantic / NL search without abandoning keyword precision.  
- Analytics search that queries **ClickHouse metadata + report catalog**, not raw OLTP dumps.

### 2.2 Non-Goals (v1)

- Replacing ClickHouse for trend analytics.  
- Indexing R2 binary bytes in-place without extracted text projections.  
- Unscoped “search everything in the company including other GC tenants.”  
- Client-side search over full corpora.

---

## 3. Capability Roadmap

| Phase | Engine | Primary use |
| --- | --- | --- |
| **v1 — Now** | **PostgreSQL FTS** (`tsvector` / `tsquery`, GIN) + trigram (`pg_trgm`) for typo/code assist | Global `/search`, type endpoints, documents search |
| **v1.5** | FTS + **search projection schema** hardened; async reindex workers | Scale indexing off request path |
| **v2 — Future** | **OpenSearch** cluster as read path for multi-type / high-QPS / highlight / aggregations | Large tenants, heavy document corpora |
| **v2+ — Future** | **pgvector** (and/or OpenSearch k-NN) embeddings | Semantic NL search, “similar hazard/SWP”, RAG-assist over allowlisted text |

Dual-run period: write to Postgres projections always; optionally dual-index OpenSearch; read traffic shifted by feature flag.

---

## 4. Logical Architecture

```text
 Domain modules (SoR)
        │ commands commit
        ▼
 Outbox → NATS events
        │
        ▼
┌───────────────────────┐
│  Search Indexer       │  (Rust activity and/or Go worker)
│  normalize → project  │
└───────────┬───────────┘
            │
            ├──────────────────────────┐
            ▼                          ▼ (future)
┌────────────────────┐      ┌────────────────────┐
│ search schema      │      │ OpenSearch indices │
│ Postgres FTS       │      │ + ingest pipelines │
│ (+ pgvector later) │      └─────────┬──────────┘
└─────────┬──────────┘                │
          └────────────┬──────────────┘
                       ▼
            Search API (Rust)
            AuthZ filter → rank → cache
                       ▼
            Web/PWA Find · Command Center
```

| Component | Responsibility |
| --- | --- |
| **Search module / facade** | Query API, ranking policy, cache keys, engine router |
| **Indexer** | Event-driven upsert/delete of projections; full reindex jobs |
| **Postgres `search` schema** | FTS documents, facets metadata, (future) embeddings |
| **OpenSearch** (future) | Distributed inverted index, aggs, highlight |
| **Redis** | Query result cache, rate-limit, suggestion cache |
| **ClickHouse** | Analytics KPI search via Analytics query layer—not FTS SoR |

---

## 5. Search Document Model

Every indexed entity is normalized to a **SearchDocument** projection:

| Field | Purpose |
| --- | --- |
| `tenant_id` | Isolation |
| `doc_id` | Stable `{type}:{id}` |
| `entity_type` | `worker` \| `project` \| `equipment` \| `document` \| `swp` \| `hazard` \| `incident` \| `training_*` \| … |
| `entity_id` | UUID |
| `project_id?` | Scope for AuthZ |
| `org_unit_id?` | Optional scope |
| `company_id?` | Optional |
| `title` | Primary display / boost |
| `subtitle?` | Secondary line |
| `body` | Searchable text (extracted/summarized) |
| `codes[]` | Asset tags, project codes, doc numbers |
| `tags[]` | Controlled tags |
| `status` | Lifecycle filter |
| `locale` | `english` / `french` / … text config |
| `acl_fingerprint` | Hash of ACL inputs for cache invalidation |
| `permissions_hint` | Coarse flags (e.g. requires `documents.document.read`) — **not** a grant |
| `updated_at` | Recency boost / cursor |
| `weight_class` | Entity prior (e.g. project > note) |
| `search_vector` | `tsvector` (generated/stored) |
| `embedding?` | Future `vector` column |

**Rebuildable:** drop projections and replay events / batch crawl via module list APIs.

---

## 6. Indexed Entity Catalog

### 6.1 Workers (People)

| Source | Indexed fields (examples) |
| --- | --- |
| People + Core membership projections | Name, employee/external ref, trade labels, status, project memberships (as filter facets—not other workers’ private HR) |

**Excluded:** medical notes, SIN/SSN, raw contact secrets beyond policy-allow (phone/email searchable only if tenant policy + permission).

### 6.2 Projects

| Source | Fields |
| --- | --- |
| Projects | Name, code, address/site label, status, client name (if stored), tags |

### 6.3 Equipment

| Source | Fields |
| --- | --- |
| Equipment | Asset tag, name/description, make/model/serial (policy), type, status, readiness summary label, project assignment ids for filter |

### 6.4 Documents & SWPs

| Source | Fields |
| --- | --- |
| Documents | Title, document number, type (`SWP`, `SJP`, policy…), version label, status, effective dates, folder/path labels, extracted plain text from **published** versions only |

SWPs are documents with type facet `swp` (and related), not a separate SoR—search type filter exposes them cleanly.

OCR text enters body only after Documents accept extraction (Go OCR is non-authoritative until accepted).

### 6.5 Hazards

| Source | Fields |
| --- | --- |
| Safety libraries + activity-derived | Hazard code/title, description, category, control library titles (linked), project id when instance-scoped |

Catalog hazards (tenant library) vs activity instance hazards distinguished by `entity_type` / facet.

### 6.6 Incidents

| Source | Fields |
| --- | --- |
| Safety incident cases | Case number, title/summary, severity, status, project, location label, dates |

**Excluded:** regulated medical detail bodies; use coarse summary fields only.

### 6.7 Training

| Source | Fields |
| --- | --- |
| Courses, assignments, completions (projections) | Course title/code, person display name (if permitted), status, due/expiry dates, project requirement labels |

### 6.8 Analytics

| Source | Approach |
| --- | --- |
| Report definitions, saved views, dashboard catalog | Postgres/Analytics metadata FTS (“find the COR readiness report”) |
| KPI/factual answers (“incidents last 30 days”) | **Not** FTS — route to Analytics query API / NL interpreter (§12) over ClickHouse with AuthZ |

Search may **deep-link** to analytics boards; it does not replace BI queries.

### 6.9 Additional (phased)

FLHAs/toolbox titles, permits, corrective actions, COR gaps, signature package subjects—same projection pattern when product prioritizes them.

---

## 7. PostgreSQL Full Text Search (v1)

### 7.1 Schema Placement

- Dedicated schema e.g. `search` (or module-owned `*.search_projections` with a unified view).  
- Documents already note `documents.search_projections`; enterprise search **unifies** behind one query API while allowing per-module projection tables if preferred.  
- RLS: `tenant_id` GUC; indexer roles write; app roles read via API only.

### 7.2 Indexing Technique

| Technique | Use |
| --- | --- |
| **`tsvector`** (weighted `A` title/codes, `B` subtitle/tags, `C` body) | Primary relevance |
| **GIN(`search_vector`)** | Lookup |
| **`pg_trgm`** on codes/title | Typo tolerance, partial asset tags |
| **B-tree** `(tenant_id, entity_type, updated_at)` | Filters / recency |
| **Partial indexes** | Exclude soft-deleted / archived if not searchable |

Text search configs: `english` default; tenant locale → `french` etc. Simple config for codes.

### 7.3 Query Parsing (v1)

1. Normalize whitespace; strip control chars.  
2. Detect **code-like** tokens (asset tags, doc numbers) → boost exact/`ILIKE`/`trgm`.  
3. Build `tsquery` from remaining terms (`plainto_tsquery` / `websearch_to_tsquery`).  
4. AND with structured filters: `types[]`, `project_id`, `status`, date range.  
5. Apply AuthZ scope SQL predicates (§11).  
6. `ts_rank_cd` / custom rank expression → limit → hydrate display DTOs.

### 7.4 Highlighting

Optional `ts_headline` on title/body snippets for Command Center; mobile may show title + subtitle only.

### 7.5 Limits

- Max query length; max result window; no unbounded `OFFSET` (cursor on `(rank, doc_id)` or `updated_at`).  
- Heavy tenants: indexer async; read replica for search if OLTP pressure (per Postgres architecture).

---

## 8. Future OpenSearch

### 8.1 When to Introduce

- FTS latency/CPU on primary OLTP unacceptable at tenant scale.  
- Need rich aggregations, percolators, or cross-field phrase features beyond Postgres comfort.  
- Very large extracted document corpora.

### 8.2 Index Design

| Index | Content |
| --- | --- |
| `proven-{env}-entities-vN` | Multi-type SearchDocuments (or per-type indices if mapping drift) |

Mappings mirror SearchDocument; `tenant_id` required on every doc; nested ACL fields only if carefully designed—**prefer query-time filter from AuthZ-resolved project allowlist** over embedding full ACL graphs.

### 8.3 Ingest

- Same NATS → Indexer path; bulk upsert by `doc_id`.  
- Pipeline: language detection optional; attachment pipeline only on extracted text already accepted by Documents.  
- Deletes/tombstones on soft-delete events.

### 8.4 Query Router

```text
Feature flag search.engine = postgres | opensearch | dual
```

API contract unchanged (`GET /search`). Dual mode: compare metrics in staging; production cutover per tenant.

### 8.5 Failure Mode

OpenSearch down → degrade to Postgres FTS for allowlisted types or show partial degradation banner—never unfiltered results.

---

## 9. Future pgvector

### 9.1 Role

Semantic similarity and NL query embedding **alongside** keyword FTS—not a replacement for exact code search.

### 9.2 Storage

- `search.embeddings` (or column on projection): `vector(N)`, model version, source `doc_id`.  
- HNSW/IVFFlat indexes per Postgres/pgvector practices.  
- Alternate: OpenSearch k-NN for larger scale; Postgres for smaller tenants / hybrid.

### 9.3 Embedding Content

Embed `title + subtitle + truncated body` of allowlisted types. Re-embed on content change; version model id for rolling upgrades.

### 9.4 Hybrid Retrieval

1. Embed user query.  
2. ANN top-K by vector.  
3. Keyword FTS top-K.  
4. **Fusion** (RRF / weighted sum) → AuthZ filter → re-rank.  
5. Exact code matches always short-circuit to top.

### 9.5 Safety

- Do not embed PHI-heavy fields.  
- Tenant-partitioned vectors (filter `tenant_id` before/with ANN).  
- Prompt-injection irrelevant for retrieval-only; if LLM answer layer added later, ground strictly in retrieved citations.

---

## 10. API Surface

| Endpoint | Purpose |
| --- | --- |
| `GET /api/v1/search` | Global multi-type search |
| `GET /api/v1/search/suggest` | Typeahead (trigram/prefix) |
| `GET /api/v1/documents/search` | Document-scoped (existing) |
| Type lists with `q=` | `/workers`, `/projects`, `/equipment`, … |
| `POST /api/v1/search/nl` (future) | Natural language interpret → structured search or analytics |

### 10.1 Global Search Request (Logical)

| Param | Meaning |
| --- | --- |
| `q` | Query string |
| `types` | CSV entity types |
| `project_id` | Scope filter (still AuthZ-checked) |
| `status` | Facet |
| `limit` / `cursor` | Pagination |
| `mode` | `keyword` (default) \| `hybrid` (future) |

### 10.2 Response Hit

| Field | Meaning |
| --- | --- |
| `entity_type`, `entity_id` | Navigation target |
| `title`, `subtitle`, `snippet?` | Display |
| `score` | Opaque relevance |
| `project_id?`, `status?` | Context |
| `url_path` | Client route hint |

Grouped sections for mobile Find optional (`workers`, `projects`, …).

---

## 11. Permissions

### 11.1 Principles

- Authentication required (except zero public search).  
- Resolve principal → **allowed project/org scopes** via Core.  
- Each hit must be visible under the permission for that entity type.  
- Document hits enforce Documents ACL / effective audience—not only project membership.  
- Analytics catalog hits require analytics/report permissions.  
- Denied docs are **omitted** (not 403 per hit); empty result ≠ existence leak for sensitive types—use careful uniformity for highly sensitive entities (incidents) per Security guidance.

### 11.2 Enforcement Points

| Layer | Role |
| --- | --- |
| **Query rewrite** | `tenant_id` + `project_id IN allowlist` (or org expansion) |
| **Permission code** | e.g. must have `equipment.asset.read` to include equipment type |
| **Post-filter** | Rare ACL exceptions (document restrict) |
| **RLS** | Defense in depth on projection tables |
| **Cache** | Keys include authz version / `acl_fingerprint` / scope hash |

Indexer may store `project_id` and document ACL version for invalidation—not a substitute for live AuthZ on restricted docs when ACL changes faster than reindex (document search may join/check Documents API for restricted classes).

### 11.3 Guest / Magic Link

No global search. Guest tokens access only package-bound resources.

---

## 12. Natural Language Search

### 12.1 Layers

| Layer | Behavior |
| --- | --- |
| **L0 — Keyword** | v1 default (`q` as FTS) |
| **L1 — Query understanding** | Detect intents: entity type, codes, filters (“open incidents on Site A”) → structured `/search` params |
| **L2 — Hybrid semantic** | pgvector/OpenSearch k-NN fusion |
| **L3 — Analytical NL** | Map to Analytics metrics/queries (“TRIF last quarter”) → ClickHouse via Analytics API with citations |

### 12.2 NL Service (Future)

```text
User utterance
  → Intent classifier / LLM constrained tool schema
  → Tools: search_entities | run_analytics_query | open_saved_report
  → AuthZ on chosen tool
  → Grounded response + deep links (no free-form inventing IDs)
```

Must not execute arbitrary SQL. Tool allowlist only. Audit NL analytical queries.

### 12.3 Ranking Interaction

NL-extracted filters apply **before** rank; semantic similarity does not override AuthZ or exact code matches.

---

## 13. Ranking

### 13.1 v1 Rank Signal (Logical)

```text
score =
  w1 * ts_rank_cd(vector, query)
+ w2 * exact_code_match
+ w3 * prefix_code_trgm
+ w4 * title_match
+ w5 * recency_decay(updated_at)
+ w6 * weight_class(entity_type)
+ w7 * project_boost(active_project_context)
```

| Signal | Intent |
| --- | --- |
| Exact code / asset tag | Field workers scanning tags |
| Title match | Human names / doc titles |
| Body FTS | SWP content discovery |
| Recency | Prefer active work |
| Weight class | Projects/docs over low-value noise |
| Active project context | Command Center Place scope boosts in-project hits |

### 13.2 Future

- Learning-to-rank later; start rule-based.  
- Hybrid RRF with vector scores.  
- Demote archived/voided unless `include_archived`.

---

## 14. Caching

| Cache | Key materials | TTL | Invalidate |
| --- | --- | --- | --- |
| **Suggest** | tenant + prefix + types + scope_hash | Short (30–120s) | Type-specific index updates |
| **Search results** | tenant + q_hash + filters + scope_hash + engine + authz_ver | Short (10–60s) | Event on indexed entity change; authz grant change bumps `authz_ver` |
| **Hydration** | entity display DTOs | Standard Redis entity cache | Module update events |
| **Negative** | empty popular queries | Very short | — |

**Do not cache** across principals unless scope_hash identical (same allowlist).  
**Do not cache** NL analytical answers long without personalization keys.

Redis is cache only—never search SoR.

---

## 15. Indexing Pipeline

### 15.1 Event-Driven

| Event (examples) | Action |
| --- | --- |
| `ProjectUpdated` | Upsert project doc |
| `PersonUpdated` / membership changed | Upsert worker doc |
| `AssetUpdated` / readiness changed | Upsert equipment |
| `DocumentVersionPublished` | Reextract body; upsert document/SWP |
| `DocumentWithdrawn` | Update status or remove from default index |
| `IncidentOpened/Updated` | Upsert incident summary |
| `HazardLibraryUpdated` | Upsert hazard |
| `CourseUpdated` / completion events | Upsert training projections |
| Soft-delete / void | Tombstone remove from default search |

### 15.2 Workers

| Worker | Role |
| --- | --- |
| **Search indexer (Go or Rust)** | Consume NATS; write Postgres projections; future OpenSearch bulk |
| **Reindex job** | Temporal/scheduled full rebuild per tenant/type |
| **Extraction** | Reuse Go OCR/PDF text → Documents accept → then search | 
| **Embedding job** (future) | Batch embed changed docs |

Indexer **does not** decide AuthZ visibility beyond storing scope foreign ids.

### 15.3 Consistency

- Near-real-time: seconds after event.  
- Search is **eventually consistent**; UI may show entity before it is searchable briefly.  
- Reindex for corruption / model change.

---

## 16. Frontend Integration

| Surface | Behavior |
| --- | --- |
| **Mobile `/find`** | Grouped global search; recent + suggest |
| **Command Center top bar** | Global search palette; project-scoped boost |
| **Place context** | Default `project_id` filter |
| **Type directories** | Workers, Equipment, Documents list `q=` |
| **Empty / stale** | Distinguish no access vs no matches carefully |

Offline: suggest/search generally **online-only**; optional tiny local recent history—not full corpus.

---

## 17. Observability & SLOs

| Signal | Use |
| --- | --- |
| p95 search latency | SLO |
| Index lag (event time → searchable) | Pipeline health |
| Zero-result rate | Query UX |
| AuthZ filter selectivity | Safety |
| Engine comparison (dual-run) | Cutover confidence |
| Cache hit ratio | Capacity |

Log `q` carefully (PII redaction / hashing for analytics).

---

## 18. Security & Privacy

- Rate-limit `/search` and `/search/nl` per principal/IP.  
- Max clause complexity to prevent query DoS.  
- No indexing of passwords, tokens, signature strokes, raw medical notes.  
- Audit optional for NL analytics and exports of search-driven bulk access.  
- Tenant isolation in OpenSearch indices or strict mandatory filters + separate clusters for high-assurance if required.

---

## 19. Analytics Search vs Entity Search

| Need | System |
| --- | --- |
| Find a person/asset/doc | Enterprise search (this doc) |
| Find a saved report/dashboard | Search over Analytics metadata |
| Compute a metric / trend | Analytics + ClickHouse |
| “Why is COR readiness down?” | Future NL → Analytics tools + deep links to evidence search |

---

## 20. Migration & Dual-Run

1. Ship unified projections + Postgres FTS API.  
2. Backfill historical published documents/text.  
3. Add OpenSearch dual write behind flag.  
4. Shadow queries; compare recall/latency.  
5. Cut read path; keep Postgres as fallback + vector home.  
6. Add embeddings; hybrid mode flag.

---

## 21. Success Criteria

1. Global search returns AuthZ-correct hits for workers, projects, equipment, documents/SWPs, hazards, incidents, training.  
2. Exact asset/doc codes rank above body noise.  
3. Index is rebuildable from events/APIs.  
4. Redis caching never serves cross-principal leakage.  
5. OpenSearch and pgvector can be introduced without changing client contracts.  
6. NL/analytics paths cannot bypass permissions or invent entities.  
7. OLTP remains healthy—heavy search offloaded when scale demands.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Search Architecture | FTS v1 + OpenSearch/pgvector roadmap |

---

*End of Enterprise Search Architecture*
