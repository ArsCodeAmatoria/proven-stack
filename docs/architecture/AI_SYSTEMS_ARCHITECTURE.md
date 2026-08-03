# Proven — AI Systems Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | AI Systems Architecture |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | AI / Platform Architecture |
| **Audience** | Engineering, Security, Product, Compliance |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Search Architecture](./SEARCH_ARCHITECTURE.md), [Security Architecture](./SECURITY_ARCHITECTURE.md), [Audit Logging](./AUDIT_LOGGING_ARCHITECTURE.md), [Data Warehouse](./DATA_WAREHOUSE_ARCHITECTURE.md), [Go Worker Catalog](./GO_WORKER_CATALOG.md), [COR Domain](./COR_DOMAIN.md), [Safety Domain](./SAFETY_DOMAIN.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document designs Proven’s **AI module**: natural language search, document summaries, OCR assist, hazard suggestions, incident analysis, domain assistants (FLHA, COR, Equipment, Training), report writing, **RAG**, **pgvector**, prompt templates, model isolation, security, audit, and **human review**.

**Hard rules**

1. AI **assists**; Rust domain modules remain **authoritative** for compliance decisions, readiness, scores, and seals ([AGENTS.md](../../AGENTS.md)).  
2. AI outputs are **candidates** until a human (or explicit module command) accepts them.  
3. **AuthZ on every retrieval and tool call** — RAG never bypasses Core `AuthzApi` / document ACL.  
4. **No secrets, stroke bitmaps, or medical note bodies** in prompts, logs, or vector stores.  
5. **Tenant isolation** in embeddings, caches, and model routes.

**Documentation only — no implementation.**

---

## 2. Strategic Placement

| Aspect | Design |
| --- | --- |
| **Module** | `ai` (supporting / generic capability) — crate `proven-ai` when implemented |
| **Not SoR for** | Safety activities, COR scores, training currency, sealed signatures |
| **Provides** | Inference orchestration, prompt catalog, RAG retrieval plans, suggestion APIs, review queues |
| **Consumes** | Search projections, Documents text, warehouse metrics (read), module query APIs |
| **Runs I/O** | Go workers for OCR/heavy embed jobs; model providers via isolated gateways |

```text
User / Assistant UI
        │
        ▼
┌───────────────────────────┐
│  proven-ai (Rust)         │
│  Intent · Tools · Prompts │
│  RAG planner · Review     │
└───────┬─────────┬─────────┘
        │         │
        ▼         ▼
  AuthZ + Module APIs    Vector + Search (pgvector / FTS / future OpenSearch)
        │
        ▼
  Model Gateway (isolated)
        │
        ▼
  Providers (LLM / embed / OCR)
```

---

## 3. Capability Catalog

### 3.1 Natural Language Search

| Aspect | Design |
| --- | --- |
| **Role** | Interpret utterance → structured search and/or analytics tools ([Search](./SEARCH_ARCHITECTURE.md) L1–L3) |
| **Flow** | Classify intent → extract entities/filters → call `search_entities` / `run_analytics_query` / `open_saved_report` |
| **Grounding** | Return citations (entity ids, deep links)—**no invented IDs** |
| **Hybrid** | Keyword FTS + pgvector fusion when `mode=hybrid` |
| **AuthZ** | Same as search: scope allowlist before return |

### 3.2 Document Summaries

| Aspect | Design |
| --- | --- |
| **Input** | Authorized published/effective document text (or accepted OCR) |
| **Output** | Short summary, key obligations, acknowledgement highlights |
| **Storage** | Optional cached summary artifact on version (regenerable) |
| **Review** | Auto for low-risk FYI; human review for external/legal distribution |
| **Never** | Summarize restricted docs user cannot read |

### 3.3 OCR

| Aspect | Design |
| --- | --- |
| **Execution** | Go OCR worker ([Go Worker Catalog](./GO_WORKER_CATALOG.md)); AI module may post-process structure |
| **Output** | Text + field candidates + confidence |
| **Authority** | Documents/module **accept** before SoR/search index |
| **RAG** | Only accepted text enters tenant corpus |

### 3.4 Hazard Suggestions

| Aspect | Design |
| --- | --- |
| **Context** | Task description, trade, project type, historical FLHA patterns (AuthZ-scoped) |
| **Output** | Ranked hazard/control suggestions from library ids—not free-text-only |
| **Accept** | User selects into FLHA draft; Safety validates on submit |
| **Ban** | Auto-submit FLHA without human confirmation |

### 3.5 Incident Analysis

| Aspect | Design |
| --- | --- |
| **Assist** | Timeline draft, similar past incidents (same tenant), suggested investigation checklist, CA themes |
| **Not** | Regulatory determination, blame assignment, automatic close |
| **Review** | Safety lead review required before attaching to case as “AI-assisted notes” |
| **Sensitivity** | Restricted; minimize PII in prompts |

### 3.6 FLHA Assistant

| Aspect | Design |
| --- | --- |
| **Help** | Wizard copilot: suggest hazards/controls, clarify incomplete sections, weather/context prompts |
| **Offline** | Suggestions from on-device cached libraries only; no cloud LLM offline unless prefetched pack |
| **Submit** | Human submits; server invariants unchanged |

### 3.7 COR Assistant

| Aspect | Design |
| --- | --- |
| **Help** | Map evidence candidates to elements, draft gap descriptions, readiness narrative for humans |
| **Not** | Change readiness score directly; forge package contents |
| **Grounding** | Only cite evidence refs user can access |
| **Review** | COR admin accepts mappings/gap text |

### 3.8 Equipment Assistant

| Aspect | Design |
| --- | --- |
| **Help** | Explain readiness blockers in plain language, suggest next inspection/cert actions, binder checklist hints |
| **Not** | Override readiness or clear deficiencies |
| **Tools** | Readiness query + asset APIs only |

### 3.9 Training Assistant

| Aspect | Design |
| --- | --- |
| **Help** | Explain gaps, suggest assignments, draft reminder copy |
| **Not** | Record completions or close gaps without Training API + AuthZ |

### 3.10 Report Writing

| Aspect | Design |
| --- | --- |
| **Help** | Draft executive narrative from **authorized** analytics metrics + cited entities |
| **Output** | Draft in review queue → human edit → export |
| **Ban** | Fabricated KPIs; must pull numbers via Analytics tools |

---

## 4. RAG (Retrieval-Augmented Generation)

### 4.1 Corpus

| Source | Indexing rule |
| --- | --- |
| Published document text | After publish + AuthZ class |
| Accepted OCR | After module accept |
| Hazard/control library | Tenant packs |
| COR element guidance | Framework packs |
| Playbooks / help articles | Platform + tenant |
| **Excluded** | Drafts (default), restricted without ACL, medical notes, raw signatures, secrets, other tenants |

### 4.2 Pipeline

```text
1. Authorize user + purpose
2. Embed query (tenant-scoped model)
3. Retrieve top-K via pgvector (+ FTS hybrid)
4. Post-filter every chunk with AuthZ / ACL
5. Build prompt with citations only from surviving chunks
6. Generate
7. Validate citations resolve; strip unsupported claims when possible
8. Audit + optional human review
```

### 4.3 Chunking

- Version-aware chunks with `document_version_id`, `tenant_id`, `acl_fingerprint`.  
- Re-embed on publish/withdraw.  
- Manifest hash for grounding verification.

---

## 5. pgvector

| Aspect | Design |
| --- | --- |
| **Home** | Postgres `ai` or `search` schema embeddings tables ([Search](./SEARCH_ARCHITECTURE.md)) |
| **Columns** | `tenant_id`, `chunk_id`, `source_ref`, `embedding vector(N)`, `model_version`, `acl_fingerprint` |
| **Index** | HNSW/IVFFlat per ops guidance; **always** filter `tenant_id` |
| **Workers** | Batch embed via Go/Temporal jobs on content change |
| **Future** | OpenSearch k-NN for scale; same AuthZ post-filter |
| **Not** | Cross-tenant ANN |

---

## 6. Prompt Templates

| Aspect | Design |
| --- | --- |
| **Catalog** | Versioned templates in `proven-ai` / Admin builder (prompt pack) |
| **Keys** | `ai.flha.hazard_suggest.v2`, `ai.cor.map_evidence.v1`, … |
| **Structure** | System policy + developer instructions + user/context slots + tool schemas |
| **Variables** | Allowlisted; validated; size-capped |
| **Change control** | Template publish with owner + eval checklist; no hot-edit prod without version bump |
| **Locale** | Locale-specific variants where needed |

System prompts include: “Only use provided tools/context; do not invent entity IDs; refuse medical/legal definitive advice; comply with tenant policy.”

---

## 7. Model Isolation

| Isolation | Design |
| --- | --- |
| **Tenant** | No multi-tenant prompt batching across tenants |
| **Environment** | Separate API keys/projects for staging vs prod |
| **Purpose** | Different routes/models for embed vs chat vs OCR post-process |
| **Data residency** | Prefer providers/regions matching tenant residency commitments |
| **Network** | Egress via allowlisted model gateway; no direct browser→LLM with raw corpus |
| **Weights** | No customer fine-tune mixing tenants without contractual isolation |
| **Caching** | Prompt/response cache keyed by tenant + template version + input hash |

Browser may call Proven AI API only—not provider keys in the PWA.

---

## 8. Security

| Control | Design |
| --- | --- |
| **AuthN/Z** | Every AI request authenticated; tools re-check AuthZ |
| **Prompt injection** | Treat retrieved docs as untrusted data; instructions in system channel; tool allowlist only |
| **Exfiltration** | Block tools that dump arbitrary tables; max tokens; no raw SQL tool |
| **PII** | Minimize; redact patterns where feasible; Restricted modes for incidents |
| **Rate limit** | Per principal/tenant |
| **Abuse** | Content policy; refuse disallowed categories |
| **Secrets** | Provider keys in vault; never in prompts |
| **Supply chain** | Pin model versions; log model id on every completion |

---

## 9. Audit

| Event | Audit |
| --- | --- |
| AI request started/completed | Action `ai.completion.*` with template key, model id, token counts (approx), purpose |
| Tool invocations | Tool name + resource ids touched |
| RAG chunk ids used | Citation list |
| Accept/reject suggestion | Human review decision |
| Prompt template publish | Admin audit |

Payloads: store **hashes** of prompts/responses or redacted excerpts—not full sensitive bodies by default. Retention per Restricted class for incident analysis.

Align with [Audit Logging](./AUDIT_LOGGING_ARCHITECTURE.md).

---

## 10. Human Review

### 10.1 Review requirement matrix

| Capability | Default review |
| --- | --- |
| NL search answers | Citations required; no separate review for navigation |
| Hazard suggestions | Human pick before draft commit |
| Document summary (internal) | Optional |
| Document summary (external share) | Required |
| Incident analysis notes | Required (safety lead) |
| COR mapping/gap drafts | Required (COR admin) |
| Report writing | Required before export |
| OCR field candidates | Module accept workflow |
| Auto-apply to SoR | **Never** without explicit product exception + dual control |

### 10.2 Review queue

| Field | Meaning |
| --- | --- |
| Suggestion id | Stable |
| Type | hazard / summary / report / cor_map / … |
| Status | `pending` \| `accepted` \| `rejected` \| `edited` |
| Reviewer | Principal |
| Edited payload | Final text/ids accepted into module API |

Accepted items call **module commands** (not AI writing Postgres directly).

---

## 11. Tooling Interface (Logical)

AI orchestrator may call only allowlisted tools:

| Tool | Backing |
| --- | --- |
| `search_entities` | Search API |
| `get_document_text` | Documents (AuthZ) |
| `get_readiness` | Equipment |
| `get_training_gaps` | Training |
| `get_cor_readiness` | COR |
| `get_analytics_metric` | Analytics |
| `list_hazard_library` | Safety library |
| `create_review_item` | AI review queue |

No generic “run SQL” or “send email as user” tools.

---

## 12. Runtime Components

| Component | Responsibility |
| --- | --- |
| **`proven-ai` API** | Assistants, summarize, suggest, NL search bridge |
| **Model gateway** | Provider adapters, timeouts, isolation |
| **Embed worker** | Batch vectors → pgvector |
| **OCR worker** | Existing Go OCR |
| **Eval harness** | Offline golden tests per template version |
| **Feature flags** | Per-tenant enablement + license entitlement |

---

## 13. UX Integration

| Surface | AI entry |
| --- | --- |
| Find / Command palette | NL search |
| Document viewer | Summarize |
| FLHA wizard | Assistant panel |
| Incident case | Analysis assist |
| COR readiness | Mapping assistant |
| Equipment asset | Explain blockers |
| Training gaps | Explain / suggest |
| Analytics | Draft narrative (review) |

Always label **AI-generated** content; show citations; Accept/Edit/Reject.

---

## 14. Offline & PWA

| Mode | Behavior |
| --- | --- |
| Offline | Local library suggestions only; no cloud LLM |
| Sync | No AI auto-accept into outbox mutations |
| Cached packs | Optional on-device suggestion models later (separate ADR) |

---

## 15. Observability & Quality

| Signal | Use |
| --- | --- |
| Latency / cost per template | Capacity |
| Citation validity rate | Grounding quality |
| Accept/reject rates | Product usefulness |
| AuthZ empty retrieval rate | Over-filter vs under-index |
| Safety refusals | Policy health |

Golden eval sets per assistant before promoting template versions.

---

## 16. Phased Rollout

| Phase | Scope |
| --- | --- |
| **P0** | OCR pipeline + NL keyword understanding; hazard suggest from library (rules/embeddings light) |
| **P1** | RAG over published docs; document summaries; pgvector hybrid search |
| **P2** | FLHA/Equipment/Training assistants + human review queue |
| **P3** | COR assistant, incident analysis, report writing with strict tools |

License/feature flags gate each capability.

---

## 17. Success Criteria

1. Every AI capability has a clear authority boundary (assist vs SoR).  
2. RAG retrieval is AuthZ-filtered and cited.  
3. pgvector is tenant-scoped and rebuildable.  
4. Prompt templates are versioned and evaluable.  
5. Models are isolated by tenant/env/purpose.  
6. Security, audit, and human review gates prevent silent compliance writes.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | AI Systems Architecture | Assistants, RAG, pgvector, review |

---

*End of AI Systems Architecture*
