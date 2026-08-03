# Proven — Offline-First Synchronization Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Offline-First Synchronization Architecture |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Offline / Mobile Architecture |
| **Audience** | Frontend, Backend, Security, SRE, Product |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Frontend Architecture](./FRONTEND_ARCHITECTURE.md), [Security Architecture](./SECURITY_ARCHITECTURE.md), [REST API](./REST_API.md), [Safety Domain](./SAFETY_DOMAIN.md), [Signatures Domain](./SIGNATURES_DOMAIN.md), [Equipment Domain](./EQUIPMENT_DOMAIN.md), [Design System](../design/DESIGN_SYSTEM.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document defines Proven’s **offline-first synchronization architecture** for the worker PWA: allowlisted field mutations, drafts, photo queues, offline signatures, inspections, FLHAs, conflict resolution, optimistic UI, background sync, recovery, and error handling.

**Hard rules**

1. **Server is system of record** — offline queues are durable intent, not alternate truth.  
2. **Domain invariants stay on the server** — client Zod/draft rules are UX only.  
3. **Allowlist only** — what is not explicitly offline-capable is online-only.  
4. **Idempotent sync** — every mutation carries a stable `mutation_id` / `Idempotency-Key`.  
5. **Sealed server evidence wins** — never silently overwrite sealed state from a stale offline client.  
6. **No secrets in IndexedDB** beyond session refresh policy defined by Security.

**Architecture only — no implementation.**

---

## 2. Goals & Non-Goals

### 2.1 Goals

- Crews complete FLHAs, pre-use inspections, acknowledgements, and (policy-permitting) seals with intermittent connectivity.  
- Photos and signature media survive airplane mode and sync without user re-entry.  
- Sync state is visible (Sync Pill), recoverable, and actionable on failure.  
- Conflicts are explicit; illegal transitions are rejected without data loss of server proof.

### 2.2 Non-Goals (Initial)

- Full offline Admin / Command Center.  
- Offline COR package generation, role/grant changes, tenant config.  
- Peer-to-peer / multi-device CRDT merge of the same activity (v1 uses last-writer rules + server authority).  
- Treating cached reference libraries as AuthZ or compliance truth without sync.

---

## 3. Architectural Overview

```text
┌─────────────────────────────────────────────────────────────┐
│                     Proven PWA (Device)                      │
│  UI (optimistic) ──► Draft Store ──► Mutation Outbox         │
│                         │                    │               │
│                         ▼                    ▼               │
│                   Media Blob Store     Sync Engine           │
│                   (photos/strokes)     (drain + retry)       │
│                              │               │               │
│                              └───────┬───────┘               │
│                                      │ Service Worker        │
│                                      │ Background Sync /     │
│                                      │ foreground drain      │
└──────────────────────────────────────┼───────────────────────┘
                                       │ HTTPS + Idempotency-Key
                                       ▼
┌─────────────────────────────────────────────────────────────┐
│              Rust API (/api/v1) + Core AuthZ                 │
│   Idempotency store · Domain aggregates · File intents       │
│   Conflict / version checks · Signature seal APIs            │
└─────────────────────────────────────────────────────────────┘
```

| Package / layer (logical) | Responsibility |
| --- | --- |
| **`packages/pwa-sync`** | Outbox, draft store, media queue, sync engine primitives |
| **Service worker** | App shell cache, Background Sync / periodic sync hooks, upload resume assist |
| **Feature modules** | Declare offline capability + mutation builders (Safety, Equipment, Signatures UI) |
| **`packages/api-client`** | Transport; always sends idempotency + correlation ids |
| **Server** | Authoritative apply, conflict codes, file complete, seal |

---

## 4. PWA Foundation

| Capability | Design |
| --- | --- |
| **Install** | Web app manifest; mobile-first worker shell |
| **Service worker** | Precache app shell + static assets; **not** entire API corpus |
| **Update** | Prompt when new shell available; drain outbox before hard swap when possible |
| **Auth** | Online re-auth / refresh as needed before drain; expired session pauses sync with clear UX |
| **Themes** | Day / Dark / Site HC work offline (CSS/tokens cached with shell) |
| **Push** | Optional; not required for sync drain |

Offline mode is a **first-class state**, not an error: banners/Sync Pill use design tokens (`color.status.sync`).

---

## 5. Offline Allowlist

### 5.1 Allow (Typical)

| Capability | Module | Notes |
| --- | --- | --- |
| Create/update **draft** FLHA / toolbox / inspection | Safety | Local draft + outbox patches |
| **Submit** safety activity (typed) | Safety | Server validates hazards/controls |
| Pre-use / periodic **inspection responses** | Equipment / Safety | Checklist snapshot cached |
| Photo / attachment **capture queue** | Core Files + subject | Upload then bind |
| Acknowledgements (policy-allow) | Documents / Safety | May require later online seal |
| **Offline signatures** (policy-allow) | Signatures | Capture locally; seal sync with assurance rules |
| Weather manual snapshot | Safety | Attached to activity |
| Mark notification read (UX) | Notifications | Optimistic; low risk |

### 5.2 Deny (Online-Only, Initial)

Admin console, COR package generation, role/permission changes, void-after-seal, incident regulatory close-out, library admin, tenant branding, API key ops, bulk exports.

Capability flags live on **ActivityTypeDefinition** / feature flags so tenants can tighten further.

---

## 6. Local Persistence Model

### 6.1 Stores (IndexedDB Logical)

| Store | Contents |
| --- | --- |
| **`drafts`** | In-progress forms keyed by `(tenant, type, local_id \| server_id)` |
| **`outbox`** | Ordered mutation intents awaiting ACK |
| **`media`** | Blob refs for photos/signature strokes + upload state |
| **`snapshots`** | Cached reference data (hazards, checklists, crew) + `fetched_at` |
| **`sync_meta`** | Clock offset, last successful drain, device id, schema version |
| **`identity_cache`** | Minimal principal display; **no** password; refresh token only per Security policy |

### 6.2 Draft Record

| Field | Purpose |
| --- | --- |
| `draft_id` | Stable local UUID |
| `server_id?` | After first create ACK |
| `aggregate_type` | e.g. `safety.activity` |
| `schema_version` | Form schema / type version |
| `payload` | Current form JSON |
| `base_version?` | Last known server version/etag |
| `updated_at_local` | Device time |
| `status` | `editing` \| `queued_submit` \| `synced` \| `conflict` |

Drafts autosave on debounce and on step navigation; explicit “Save draft” is optional UX sugar.

### 6.3 Outbox Mutation Record

| Field | Purpose |
| --- | --- |
| `mutation_id` | UUID = HTTP `Idempotency-Key` |
| `aggregate_key` | e.g. `safety.activity:{id}` for ordering |
| `op` | `create` \| `patch` \| `submit` \| `upload_complete` \| `bind_attachment` \| `seal_slot` \| … |
| `body` | API DTO |
| `depends_on[]` | Prior `mutation_id`s (e.g. upload before bind) |
| `created_at` | Enqueue time |
| `attempts` | Retry count |
| `last_error?` | Structured error |
| `state` | `pending` \| `in_flight` \| `acked` \| `dead` \| `conflict` |

---

## 7. Synchronization Protocol

### 7.1 Drain Algorithm

1. Ensure network + valid session (refresh if needed).  
2. Select next outbox items whose `depends_on` are acked.  
3. Respect **per-aggregate FIFO** (never apply patch N+1 before N for same activity).  
4. Cross-aggregate: fair interleaving; prioritize `seal` / `submit` after their deps.  
5. Send mutation with `Idempotency-Key: mutation_id`, `X-Correlation-Id`.  
6. On success: mark acked; merge canonical server entity into TanStack Query + draft.  
7. On retryable failure: backoff; keep pending.  
8. On conflict / illegal transition: mark `conflict`; stop that aggregate’s queue until resolved.  
9. Update Sync Pill counts.

### 7.2 Transport Triggers

| Trigger | Behavior |
| --- | --- |
| **Online event** | Start drain |
| **Foreground focus** | Drain + selective refetch |
| **Background Sync API** | Re-register tag `proven-outbox`; drain when browser fires |
| **Manual Retry** | User from Sync Center |
| **After local enqueue** | Attempt immediate drain if online |

Background Sync is **best-effort** (browser/OS dependent). Foreground drain remains the reliable path; design must not require BG Sync for correctness.

### 7.3 Pull / Hydration

- On reconnect: invalidate allowlisted query keys; refetch My Actions + open drafts’ server state.  
- Snapshots refresh with staleness labels (“Checklist from 2d ago”).  
- No full DB dump to device.

### 7.4 Clock & Ordering

- Server timestamps are canonical.  
- Client stores `server_time_offset` from response headers when available.  
- Offline seal timestamps include device time **and** sync receipt time on server.

---

## 8. Optimistic Updates

| Class | Client behavior | On failure |
| --- | --- | --- |
| **Low-risk UX** | e.g. mark notification read — optimistic | Rollback UI |
| **Draft edits** | Local draft is source until sync | Keep local; show pending |
| **Submit / seal** | Optimistic “Submitting…” / “Sealing…” **not** “Sealed” until ACK | Revert to pending/conflict |
| **Photo thumb** | Show local blob immediately | Keep in media queue |

**Never** optimistically show **server-sealed / Closed** compliance states. Proof language waits for canonical ACK.

TanStack Query: optimistic updates only on allowlisted mutation hooks; others wait for server.

---

## 9. Conflict Resolution

### 9.1 Detection

| Signal | Meaning |
| --- | --- |
| Version / etag mismatch | Concurrent edit |
| Illegal status transition | e.g. patch after server Closed/Voided |
| Idempotency replay with different body | Client bug / tamper → hard fail |
| Duplicate create with same key | Return original resource (idempotent success) |
| Seal on completed/voided package | Reject |

### 9.2 Policies

| Situation | Resolution |
| --- | --- |
| **Server sealed / voided / closed** | **Server wins**; client draft moved to `conflict` read-only copy; user notified |
| **Two devices draft same activity** | Server version + merge UI: keep mine / take server / manual field merge (v1: choose side + re-edit) |
| **Offline submit vs online submit** | First successful submit wins; second gets conflict |
| **Attachment bind to missing activity** | Retry after create dep; else dead-letter |
| **Checklist schema version skew** | Reject submit; force refresh schema; preserve answers where mappable |

### 9.3 UX

- Conflict banner on the activity with Compare.  
- Aggregate outbox paused until user resolves or discards local.  
- Audit: server records rejected stale attempts with correlation id.

No automatic CRDT merge of hazard lists in v1.

---

## 10. Photo / Media Upload Queue

### 10.1 Pipeline

```text
Capture → store blob in `media` (local_id)
  → enqueue CreateFileUploadIntent (when online path starts)
  → PUT presigned R2 (chunk/resume if supported)
  → CompleteFileUpload (checksum)
  → enqueue BindAttachment(activity_id, file_object_id)
  → AV scan async on server (Available/Quarantine)
```

### 10.2 Rules

- Photos never only in memory—persist blob before leaving camera UI.  
- Outbox `depends_on` ensures bind cannot run before upload complete.  
- Size/type checks client-side (UX); server authoritative.  
- Failed AV → quarantine; activity shows attachment unavailable; user may retake.  
- Storage pressure: warn when device quota low; block new captures before silent eviction.  
- Logout / remote wipe: clear media store with session.

---

## 11. Draft Saving

| Mechanism | Design |
| --- | --- |
| **Autosave** | Debounced writes to `drafts` |
| **Step boundary** | Wizard next/back flushes draft |
| **Crash recovery** | On launch, reopen unsynced drafts in My Actions / Continue |
| **Server draft** | First sync create allocates `server_id`; further patches reference it |
| **Discard** | Local delete; if server draft exists, enqueue abandon/cancel if API supports |
| **Schema upgrades** | Migration functions on draft payload; fail to “review required” if incompatible |

Draft ≠ submitted evidence. Submitting enqueues explicit `submit` mutation.

---

## 12. Offline Signatures

### 12.1 Policy Gate

Offline seal allowed only when Signatures **SigningPolicy** + subject type allow it (typically authenticated worker on known package/slot; **not** guest magic-link in airplane mode without prior token materialization).

### 12.2 Capture Flow

1. Ensure package/slot snapshot cached (or create-package mutation already acked).  
2. Capture stroke/image → `media` blob + local hash.  
3. Build `seal_slot` mutation with identity assurance available offline (session still valid per policy).  
4. Persist intent; show **Pending seal** (not Proven/Completed).  
5. On sync: upload stroke file → seal API with assurance metadata + offline flag + captured_at.  
6. Server validates package state, document version (if any), slot order, AuthZ.  
7. On success: package progress updates; subject module callbacks as online.

### 12.3 Restrictions

- Step-up MFA that requires online challenge → block offline seal; queue “ready to seal” until online.  
- Sequential slots: cannot seal slot 2 offline if slot 1 not known complete.  
- Completed packages immutable; offline seal after void → conflict.  
- Guest QR/magic-link: generally **online** unless link token and package snapshot pre-cached under strict TTL (optional later).

---

## 13. Offline Inspections

| Phase | Offline behavior |
| --- | --- |
| **Start** | Create inspection activity/draft from cached checklist definition |
| **Respond** | Autosave answers + photo queue per item |
| **Asset context** | Cached asset tag/readiness snapshot with staleness label |
| **Submit** | Outbox submit; server recomputes readiness |
| **Blocked readiness implications** | May require online confirmation messaging after ACK |

Periodic inspection due workflows remain server-side; device only completes the field form.

---

## 14. Offline FLHAs

| Phase | Offline behavior |
| --- | --- |
| **Create** | Local draft with project/crew snapshot |
| **Hazards/controls** | From cached libraries + custom text; library ids validated on submit |
| **Participants** | Cached crew; cannot invent unauthorized persons—server checks membership |
| **Photos** | Media queue |
| **Risk** | Client advisory only; server may recompute/validate on submit |
| **Submit** | Outbox; starts review/signature workflows server-side |
| **Signatures** | Per §12 after submit/package create |

Toolbox talks follow the same activity pipeline with attendance/seal specifics.

---

## 15. Reference Data Caching

| Data | Strategy |
| --- | --- |
| Hazard/control libraries | Cache-first; refresh on connect |
| Checklist / activity type schemas | Versioned; pin to draft |
| Assigned projects / My Actions list | Soft cache; refetch on focus |
| Crew membership | Snapshot at draft start; refresh before submit if online |

Staleness UX required. Cache never grants permissions.

---

## 16. Optimistic UI vs Sync Pill

| Sync Pill state | Meaning |
| --- | --- |
| **Hidden / Clear** | No pending |
| **Pending N** | Outbox depth |
| **Syncing** | Drain in progress (deterministic progress when count known) |
| **Offline** | No network; accepting local work |
| **Action needed** | Conflicts / auth / dead letters |

Sync Center screen: list mutations, errors, Retry, Resolve conflict, Discard local (dangerous, confirmed).

Live regions announce sync failures for a11y.

---

## 17. Recovery

| Scenario | Recovery |
| --- | --- |
| **App kill mid-edit** | Draft store reload |
| **Kill mid-upload** | Media state `uploading` → resume or re-presign |
| **Kill mid-drain** | `in_flight` without ACK → retry same `mutation_id` (idempotent) |
| **SW update** | Persist outbox; migrate schema version in `sync_meta` |
| **Reinstall** | Local queue lost — mitigate via server-side drafts after first sync; educate users to connect early once |
| **Session expired** | Pause drain; re-auth; continue |
| **Tenant suspend** | Pause; surface error; do not invent success |
| **Storage quota exceeded** | Block new media; prompt sync/delete |
| **Clock skew extreme** | Prefer server time; warn user |

**Poison message:** after N retries or non-retryable error → `dead`; user must Retry with edit or Discard; ops can use correlation id.

---

## 18. Error Handling

### 18.1 Error Classes

| Class | Client action |
| --- | --- |
| **Network / 5xx / 429** | Retry with exponential backoff + jitter; honor `Retry-After` |
| **401** | Refresh; if fail, pause queue; re-login |
| **403** | Mark dead/conflict; do not infinite retry |
| **409 / conflict codes** | Aggregate pause; conflict UI |
| **422 validation** | Surface field errors; allow draft edit; new mutation id only if body changes per idempotency rules |
| **Idempotency key reuse different body** | Hard fail; generate support code |
| **Quarantined file** | Attachment error path; retake |

### 18.2 Idempotency Contract

- Same key + same body → same result (200/replay).  
- Clients **must not** change body for an existing `mutation_id`.  
- Edits after failure that need new payload → **new** `mutation_id` (and supersede prior if acked/failed policy).

### 18.3 User Messaging

- Plain language: “Saved on this device — not yet proven on server.”  
- Never claim sealed/closed until ACK.  
- Include correlation id under Details for support.

---

## 19. Security Considerations (Offline)

| Topic | Control |
| --- | --- |
| **AuthZ** | Rechecked on every sync apply |
| **Replay** | Idempotency + session validity |
| **Device storage** | OS protection; clear on logout; minimize PII in snapshots |
| **Stroke/photo blobs** | Treated as sensitive; not backed up to insecure shares |
| **Guest tokens** | Not written to long-lived shared stores without TTL |
| **Tampering** | Server validates; client integrity not trusted |

---

## 20. Server Responsibilities

| Responsibility | Detail |
| --- | --- |
| **Idempotency store** | Persist key → response for mutation window |
| **Version / status checks** | Emit stable conflict problem codes |
| **Offline-aware seal** | Accept captured_at + sync_received_at; assurance policy |
| **File pipeline** | Presign, complete, AV, bind |
| **Workflows** | Start only after successful authoritative submit/seal |
| **Partial failure** | Commands transactional per aggregate; no half-submit |

Workers (Go) may process AV/PDF after upload; they do not authorize offline business transitions.

---

## 21. Multi-Device & Multi-User

| Case | v1 approach |
| --- | --- |
| Same user, two devices | Per-device outboxes; server merges via versions; conflicts explicit |
| Crew co-edit one FLHA | Prefer single primary author offline; others online or read snapshot |
| Supervisor review | Online; not offline allowlisted initially |

---

## 22. Observability

| Metric / signal | Use |
| --- | --- |
| Outbox depth (client telemetry, privacy-safe) | Product health |
| Sync success/fail rates by `op` | Reliability |
| Conflict rate by aggregate type | UX/API tuning |
| Upload bytes / fail | Media pipeline |
| Time-to-ACK from enqueue | Field performance |

Client telemetry must not include stroke payloads or photo bytes.

---

## 23. Testing Strategy

| Layer | Focus |
| --- | --- |
| Unit | Outbox ordering, dependency graph, draft migration |
| Integration | Idempotent replay, conflict codes against API mocks |
| E2E | Airplane mode FLHA → photo → submit → online seal/ACK |
| Chaos | Kill app during upload; 409 mid-queue; 401 mid-drain |
| Storage | Quota full; schema upgrade |

---

## 24. Rollout Phases

| Phase | Scope |
| --- | --- |
| **P0** | Drafts + outbox + FLHA/inspection submit + photo queue + Sync Pill |
| **P1** | Background Sync hooks + resume uploads + conflict compare UX |
| **P2** | Offline authenticated seal (policy) |
| **P3** | Richer multi-device merge tooling; optional pre-cached guest sign |

---

## 25. Success Criteria

1. Worker can complete an allowlisted FLHA/inspection with photos entirely offline and reach server-proven state after reconnect without re-entering data.  
2. Sync is idempotent under repeated drains and app kills.  
3. Sealed server evidence cannot be overwritten by stale clients.  
4. Optimistic UI never lies about sealed/closed proof.  
5. Conflicts and auth failures are recoverable via Sync Center.  
6. Admin/COR/privileged mutations remain online-only.  
7. Security AuthZ and AV scanning still apply on sync path.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Offline-First Architecture | PWA sync, conflicts, media, field flows |

---

*End of Offline-First Synchronization Architecture*
