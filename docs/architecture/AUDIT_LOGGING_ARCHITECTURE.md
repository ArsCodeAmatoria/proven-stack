# Proven — Audit Logging Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Audit & Compliance Logging Architecture |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Audit / Compliance Architecture |
| **Audience** | Security, Compliance, Backend, SRE, COR/Audit sponsors |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Core Domain](./CORE_DOMAIN.md), [Security Architecture](./SECURITY_ARCHITECTURE.md), [Authorization RBAC](./AUTHORIZATION_RBAC_ARCHITECTURE.md), [Signatures](./SIGNATURES_DOMAIN.md), [Temporal Workflows](./TEMPORAL_WORKFLOWS.md), [Event Catalog](./EVENT_CATALOG.md), [Data Warehouse](./DATA_WAREHOUSE_ARCHITECTURE.md), [PostgreSQL](./POSTGRESQL_ARCHITECTURE.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document designs Proven’s **audit logging** system for compliance and security accountability: what is logged (actions, signatures, approvals, logins, permission changes, forms, workflows, documents), **immutability**, **retention**, **exports**, **search**, and **reporting**.

**Hard rules**

1. **Core `AuditApi` is the platform audit SoR** — modules append; they do not keep a second authoritative audit store.  
2. **Append-only** — never update or delete audit facts in place; archival is partition/export based.  
3. **Same transactional boundary when possible** — significant commands append audit with the state change (or reliable outbox → append).  
4. **No secrets in audit payloads** — passwords, magic-link secrets, TOTP seeds, raw signature strokes, medical note bodies forbidden.  
5. **Domain evidence ≠ audit log** — sealed FLHAs/signatures remain in owning modules; audit references them for provenance.

**Documentation only — no implementation.**

---

## 2. Objectives

| Objective | Meaning |
| --- | --- |
| **Accountability** | Who did what, when, on which subject, under which session/assurance |
| **Non-repudiation support** | Combined with signature evidence certificates |
| **Security forensics** | Login failures, grant changes, break-glass |
| **Compliance reconstruction** | Approvals, publishes, voids, workflow outcomes |
| **COR provenance assist** | Deep-link from audit hits to evidence subjects |

---

## 3. Architecture Overview

```text
Module command / AuthN event / Workflow terminal
        │
        ▼
 Core AuditApi.Append  (preferred: same TX as write)
        │
        ▼
 core.audit_entries  (append-only, RLS, integrity digest)
        │
        ├── Query / Search (Admin, auditors)
        ├── ExportJob → R2 artifact (workflow)
        └── Optional: Analytics facts (ANLY) for trends — not SoR
```

| Store | Role |
| --- | --- |
| **`core.audit_entries`** | Authoritative audit log |
| **Module SoR** | Business entities & sealed evidence |
| **Signatures evidence** | Assent proof (strokes/certs)—referenced by audit, not duplicated as bitmaps |
| **ClickHouse** | Optional audit *volume/trend* metrics—not legal SoR |
| **R2** | Export packages / WORM-style cold archives |

---

## 4. Audit Entry Model (Logical)

| Field | Purpose |
| --- | --- |
| `audit_entry_id` | Unique id |
| `tenant_id` | Isolation |
| `occurred_at` | Event time (server) |
| `recorded_at` | Insert time |
| `actor_principal_id` | Who (nullable for system) |
| `actor_type` | `user` \| `service` \| `system` \| `guest_token` |
| `impersonation` / `delegation_of` | If acting under delegation |
| `session_id` | `sid` when human |
| `amr` / `acr` | Auth methods / assurance |
| `device_id` | Optional device |
| `ip` / `user_agent` | Policy-limited; shorter retention class possible |
| `action` | Stable action code (see catalog) |
| `category` | `auth` \| `authz` \| `data` \| `signature` \| `workflow` \| `admin` \| … |
| `outcome` | `success` \| `deny` \| `failure` |
| `resource_type` / `resource_id` | Primary subject |
| `resource_labels` | Optional display codes (project code, doc number)—not PII dumps |
| `project_id` | When applicable |
| `company_id` | When applicable |
| `correlation_id` / `causation_id` | Trace |
| `workflow_instance_id` | When workflow-related |
| `module` | Owning module key |
| `before_ref` / `after_ref` | Version ids or hash pointers—not full row dumps |
| `payload_digest` | Hash of canonical payload |
| `payload_summary` | Redacted JSON summary (allowlisted fields only) |
| `integrity_prev_hash` / `integrity_hash` | Optional hash chain for tamper evidence |
| `sensitivity` | `standard` \| `restricted` |

---

## 5. What Must Be Audited

### 5.1 Every significant action (platform rule)

“Every action” means every **compliance- or security-significant command**, not every UI click or read of a public checklist.

| Include | Exclude (typical) |
| --- | --- |
| Creates, updates, submits, closes, voids, publishes | Benign list scrolls |
| Denies on sensitive resources (policy) | High-volume health checks |
| Exports of personal/audit data | Cache hits |
| File quarantine / delete | Pure display formatting |

Modules declare significant commands in their design; Core enforces append for catalogued actions.

### 5.2 Every signature

| Event | Audit action examples |
| --- | --- |
| Package created/voided | `signatures.package.created` / `voided` |
| Slot sealed (user or guest) | `signatures.slot.sealed` |
| Magic link issued/redeemed/revoked | `signatures.magic_link.*` (no secret) |
| QR session started/completed | `signatures.qr.*` |
| Evidence certificate generated | `signatures.certificate.generated` |

Payload: package id, slot id, subject ref, assurance method—**not** stroke bitmap.

### 5.3 Every approval

| Domain | Examples |
| --- | --- |
| Documents | Review submitted, version approved/rejected, publish, withdraw |
| Safety | Activity reviewed, permit approved, lift plan approved |
| Training | Waiver granted (if any) |
| Admin | Builder publish approved |
| Workflows | Human task approved/rejected signal |

Action codes under `documents.approval.*`, `safety.review.*`, etc.

### 5.4 Every login (and auth lifecycle)

| Event | Action |
| --- | --- |
| Login success/failure | `auth.login.success` / `auth.login.failure` |
| Logout | `auth.logout` |
| Refresh / session rotate | `auth.session.refresh` (may sample if volume high—document policy) |
| MFA enroll/challenge/fail | `auth.mfa.*` |
| Password reset request/complete | `auth.password_reset.*` |
| Magic link auth issued/redeemed | `auth.magic_link.*` |
| OAuth link/unlink | `auth.identity.*` |
| Session revoke / logout-all | `auth.session.revoked` |
| Step-up success | `auth.stepup.success` |

Failures: no password values; reason codes only.

### 5.5 Every permission change

| Event | Action |
| --- | --- |
| Role created/changed/retired | `authz.role.*` |
| Grant / revoke | `authz.grant.*` |
| Membership grant/update/revoke | `authz.membership.*` |
| Delegation create/revoke/expire | `authz.delegation.*` |
| Temporary / break-glass grant | `authz.temporary.*` |
| API key issue/rotate/revoke | `admin.apikey.*` |

Include scope, role id, permission set summary, expiry.

### 5.6 Every form (field compliance forms)

“Form” = Safety activities, inspections, acknowledgements, orientation, structured admin builders publishes—not every keystroke.

| Lifecycle | Audit |
| --- | --- |
| Draft created / updated (policy: optional for high-churn drafts; **required** on submit) | Prefer audit on **submit**, **seal**, **close**, **void** |
| Submit | `safety.activity.submitted`, `equipment.inspection.submitted`, … |
| Offline sync accepted | Include `mutation_id` / idempotency key in summary |
| Validation reject at API | Optional `outcome=failure` for abuse monitoring |

Autosave drafts: generally **not** per-save audit (noise); final submit is mandatory.

### 5.7 Every workflow

| Event | Action |
| --- | --- |
| Started | `workflow.instance.started` |
| Signaled (material) | `workflow.instance.signaled` |
| Escalation fired | `workflow.escalation.triggered` |
| Completed / failed / cancelled | `workflow.instance.*` |
| Human assignment completed | `workflow.assignment.completed` |

Link `workflow_instance_id` + subject. Temporal history is operational; **Core audit** is compliance-facing summary.

### 5.8 Every document

| Event | Action |
| --- | --- |
| Document created | `documents.document.created` |
| Version created / updated | `documents.version.*` |
| Published / withdrawn / superseded | `documents.version.published`, … |
| ACL changed | `documents.acl.changed` |
| Ack campaign started / completed / cancelled | `documents.ack_campaign.*` |
| Individual ack completed | `documents.ack.completed` |
| Guest doc sign (if any) | Via signatures actions + doc subject ref |

---

## 6. Action Code Catalog

Stable, namespaced strings:

```text
{domain}.{entity}.{verb}
```

Owned alongside permission catalog review. Breaking renames forbidden—deprecate.

Categories enable filter/search facets in Admin audit viewer.

---

## 7. Immutability & Integrity

| Control | Design |
| --- | --- |
| **Insert-only** | No UPDATE/DELETE on audit rows for app roles |
| **DB privileges** | Only `proven_migrator` / break-glass can alter table structure; no app delete |
| **Hash chain (optional tier)** | `integrity_hash = H(prev_hash + canonical_entry)`; verify job |
| **WORM export** | Periodic signed export to R2/Object Lock for high-assurance tenants |
| **Corrections** | Never edit; append `audit.correction` referencing original id with reason |
| **Time** | Server clock; record skew notes if client offline captured_at differs |

Soft-delete does **not** apply to audit tables ([PostgreSQL Architecture](./POSTGRESQL_ARCHITECTURE.md)).

---

## 8. Write Path Patterns

### 8.1 Preferred

```text
BEGIN
  domain state change
  AuditApi.Append(...)
COMMIT
```

### 8.2 Acceptable

Outbox event → Core consumer appends audit (at-least-once; idempotent on `audit_entry_id` / causation key).

### 8.3 Denies

Sensitive AuthZ denies may append `outcome=deny` without a domain write.

### 8.4 Guest actors

`actor_type=guest_token`; principal null; include package/slot ids; IP retained per policy.

---

## 9. Retention

| Class | Baseline retention | Notes |
| --- | --- | --- |
| **Security/authz audit** | 7+ years (or legal max needed) | Grants, login, break-glass |
| **Compliance operational** | 7–10+ years | Signatures, publishes, CA close, COR |
| **IP / UA fields** | Shorter (e.g. 12–24 months) then null via archival rewrite **only** through controlled archival process that preserves row id + hash policy—or store IP in side table with shorter TTL |
| **Standard payload summaries** | Align with parent class | |
| **Legal hold** | Suppress archival purge per tenant/case | |

Archival: detach old partitions → cold storage export → drop partition (not row delete). Document chain of custody.

OLTP sealed evidence retention is **separate** but complementary.

---

## 10. Exports

### 10.1 Flow

```text
Authorized request (core.audit.export)
  → ExportAuditLog command
  → Temporal Export / Audit export workflow
  → Filter by tenant + AuthZ project scope
  → Render CSV/JSON/PDF index
  → Store FileObject in R2
  → Audit the export itself
  → Notify requester
```

### 10.2 Controls

| Control | Spec |
| --- | --- |
| Permission | `core.audit.export` (+ often step-up MFA) |
| Scope | Never cross-tenant; project filter intersection |
| Redaction | Restricted fields omitted unless elevated |
| Caps | Max rows/bytes; async only |
| Format | CSV / JSON lines / PDF summary |
| Chain | Include integrity hashes when enabled |

### 10.3 COR / external auditor packs

COR evidence packages may **cite** audit entry ids as provenance; full audit export is a separate Admin/compliance tool.

---

## 11. Search

### 11.1 Product search vs audit search

| Need | System |
| --- | --- |
| Find workers/docs | [Search Architecture](./SEARCH_ARCHITECTURE.md) |
| Find who published doc X | **Audit query API** |

### 11.2 Audit query capabilities

| Filter | Examples |
| --- | --- |
| Time range | required for large tenants |
| Actor | principal id |
| Action / category | `documents.version.published` |
| Resource | type + id |
| Project | id |
| Outcome | success/deny/failure |
| Workflow instance | id |
| Correlation id | support |

### 11.3 Indexing

- B-tree/time + tenant leading keys; secondary on resource, actor, action.  
- Optional FTS on allowlisted `payload_summary` for admin search.  
- Heavy historical search may use read replica ([PostgreSQL](./POSTGRESQL_ARCHITECTURE.md)).  
- Do not index secrets (there should be none).

### 11.4 AuthZ

`core.audit.read` / `admin.audit.view`; further restrict by project allowlist for non-global auditors.

---

## 12. Reporting

| Report | Source |
| --- | --- |
| **Admin audit viewer** | Live query SoR |
| **Login success/fail trends** | Optional CH facts from audit/auth events |
| **Permission change report** | Audit filter `authz.*` |
| **Publish / approval register** | Audit `documents.*` / approvals |
| **Signature activity register** | Audit `signatures.*` + Signatures module |
| **Workflow SLA breach with actors** | Workflow + audit join via ids |
| **Export access report** | Who exported audit/PII |

Analytics dashboards may show **counts**; legal registers should export from **Core audit** or sealed evidence—not CH alone.

---

## 13. Privacy & Minimization

- Prefer ids + digests over body text.  
- Medical / fit details: never in audit payload.  
- Guest email may appear if captured as signer identity—treat as PII; retention class Restricted.  
- Subject access requests: export actor’s audit slice via controlled process.  
- Align with GDPR/PIPEDA sections in Security Architecture.

---

## 14. API Surface (Logical)

| API | Purpose |
| --- | --- |
| `AuditApi.Append` | Modules / auth pipeline |
| `QueryAuditEntries(filter)` | Paged search |
| `ExportAuditLog(request)` | Start export job |
| `VerifyAuditIntegrity(range)` | Optional hash-chain verify (platform ops) |

HTTP: Admin `/admin/audit` or `/audit` under Core routes—exact paths in REST catalog.

---

## 15. Permissions

| Code | Meaning |
| --- | --- |
| `core.audit.read` | Query logs |
| `core.audit.export` | Export |
| `admin.audit.view` | Admin console entry |
| Append | Implicit for services via internal API—not a user permission |

---

## 16. Operational Concerns

| Concern | Design |
| --- | --- |
| **Volume** | Partition by month; sampling only for ultra-noisy success refreshes if explicitly approved |
| **Latency** | Append in TX; async only when justified |
| **Failure** | If audit append fails, **fail the command** for mandatory classes (compliance-critical) |
| **SIEM** | Optional ship security category to external SIEM (redacted) |
| **Testing** | Assert mandatory actions produce entries; forbid secrets in fixtures |

---

## 17. Mapping Requirements → Design

| Requirement | Design section |
| --- | --- |
| Every Action | §5.1 significant commands |
| Every Signature | §5.2 |
| Every Approval | §5.3 |
| Every Login | §5.4 |
| Every Permission Change | §5.5 |
| Every Form | §5.6 submit/seal lifecycle |
| Every Workflow | §5.7 |
| Every Document | §5.8 |
| Immutable Logs | §7 |
| Retention | §9 |
| Exports | §10 |
| Search | §11 |
| Reporting | §12 |

---

## 18. Success Criteria

1. Compliance-critical state changes always produce an immutable Core audit entry.  
2. Signatures, approvals, auth, authz, forms (submit+), workflows, and documents are covered by stable action codes.  
3. Logs cannot be altered by application roles; corrections are append-only.  
4. Retention and legal hold are operable without destroying required history.  
5. Exports and search are AuthZ-scoped, redacted, and themselves audited.  
6. Reporting for auditors can reconstruct who approved/published/sealed what, with links to evidence SoR.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Audit Compliance Architecture | Platform audit logging design |

---

*End of Audit Logging Architecture*
