# Proven — Enterprise Security Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Enterprise Security Architecture |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Enterprise Security Architecture |
| **Audience** | Security, Engineering, Compliance, SRE, Legal (privacy) |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [System Architecture](./SYSTEM_ARCHITECTURE.md), [Core Domain](./CORE_DOMAIN.md), [REST API](./REST_API.md), [PostgreSQL](./POSTGRESQL_ARCHITECTURE.md), [Temporal Workflows](./TEMPORAL_WORKFLOWS.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document defines **Proven’s security architecture**: authentication, authorization (RBAC + ABAC), JWT/session model, MFA, audit, encryption, secrets, Cloudflare edge controls, OWASP alignment, input validation, file upload / malware scanning, rate limiting, privacy (GDPR / PIPEDA), password policy, and threat modeling.

**Hard rules**

1. **Server AuthZ is authoritative** — UI permission hiding is UX only.  
2. **Tenant isolation is mandatory** — tenant comes from auth context, never from client as authority.  
3. **Fail closed** — missing scope, ambiguous grant, or disabled module → deny.  
4. **Sealed evidence is immutable** — security controls protect integrity; voids are compensating records, not silent deletes.  
5. **Secrets never in git, client bundles, logs, or events.**

**Documentation only — no implementation.**

---

## 2. Security Objectives & Principles

| Objective | Meaning for Proven |
| --- | --- |
| **Confidentiality** | Tenant/project data, PII, medical notes, signature media, guest tokens |
| **Integrity** | Compliance evidence, audit trail, signed packages, COR packages |
| **Availability** | Field ops (FLHA, pre-use, guest sign) under attack and peak load |
| **Accountability** | Who did what, when, on which subject, under which session/assurance |
| **Least privilege** | Grants scoped to Tenant / Org / Project / Team / Self |
| **Defense in depth** | Cloudflare → API middleware → AuthzApi → RLS → object ACLs |
| **Privacy by design** | Data minimization, purpose limitation, retention, subject rights |

**Trust boundaries**

```text
Internet / Field devices
        │
        ▼
┌───────────────────────┐
│ Cloudflare Edge       │  DNS · WAF · DDoS · Bot · CDN · Access
└───────────┬───────────┘
            ▼
┌───────────────────────┐
│ Next.js (Vercel)      │  Session cookies · BFF patterns · no secrets
└───────────┬───────────┘
            ▼
┌───────────────────────┐
│ Rust API (Fly.io)     │  AuthN · AuthZ · Domain · Outbox
└─────┬─────┬─────┬─────┘
      │     │     │
      ▼     ▼     ▼
 Postgres  R2   Temporal / NATS / Redis / ClickHouse / Workers
 (RLS)   (ACL)  (service auth)
```

---

## 3. Identity Model

| Concept | Owner | Notes |
| --- | --- | --- |
| **User** | Core | Human account that authenticates |
| **Principal** | Core | Security subject (User or service/API client) |
| **Person** | People | Workforce identity referenced by Core (`PersonRef`) |
| **Tenant** | Core | Isolation boundary |
| **Session** | Core | Server-side session; revocation source of truth |
| **Service principal** | Core/Admin | Workers, integrations; constrained credentials |
| **Guest principal** | Signatures | Ephemeral, package/slot-scoped; not a full User |

Mapping: SSO IdP subject → Proven User → optional Person linkage within tenant.

---

## 4. Authentication

### 4.1 Modes

| Mode | Use | Assurance notes |
| --- | --- | --- |
| **Password + MFA** | Direct login (when tenant allows) | MFA required for privileged roles (policy) |
| **OIDC SSO** | Enterprise tenants (Authorization Code + PKCE) | Preferred for GC/enterprise |
| **Magic link (auth)** | Optional low-friction login | Short TTL; single-use; rate-limited |
| **Guest / magic-link sign** | External signer on signature package | Not a platform session; package-scoped token |
| **API key** | Partner/integration clients | Hashed at rest; scoped; rotatable |
| **Service credentials** | Go workers → API | Mutual/service JWT or mTLS-equivalent secret; no user impersonation without audited reason |
| **OAuth client credentials** | Machine clients | Bound to tenant + scopes |

### 4.2 Flows (Logical)

**Password / MFA**

1. Credential verify → if MFA enrolled, challenge → establish Session → issue access token (+ refresh/cookie).  
2. Failed attempts: progressive lockout / backoff; audit `LoginFailed`.

**OIDC**

1. `/auth/oauth/authorize` → IdP → `/auth/oauth/callback` → map subject → Session.  
2. Enforce email/domain/tenant binding rules; reject unknown tenants unless invite flow.

**Guest sign**

1. Issue opaque single-use or short-lived token (hash stored).  
2. Redeem on guest routes only; cannot call general API.  
3. Expiry/revoke via Signatures + Temporal workflows.

### 4.3 Unauthenticated Surface (Minimal)

Health, OIDC callbacks, guest redeem, optional public OpenAPI, marketing CDN assets. Everything else requires auth.

---

## 5. Session Management

| Control | Spec |
| --- | --- |
| **SoR** | Server-side `Session` in Core (not JWT-only auth) |
| **Access token** | Short-lived JWT (minutes) |
| **Refresh** | Rotate on use; bind to `sid`; revoke session → all tokens dead |
| **Web cookies** | HTTP-only, Secure, SameSite=Lax/Strict as appropriate; `__Host-` prefix where possible |
| **Logout** | Revoke session server-side; clear cookies |
| **Concurrent sessions** | Policy: allow with device list; force logout-all on password change / MFA reset / compromise |
| **Idle / absolute timeout** | Tenant-configurable within platform bounds; privileged roles shorter idle |
| **Step-up** | Sensitive actions (role grant, API key create, export PII) may require recent MFA (`acr`/`amr`) |
| **Device binding** | Optional fingerprint/UA anomaly signals → step-up or revoke |
| **Offline PWA** | Offline allowlisted mutations use idempotency + re-auth on sync; no long-lived secrets in IndexedDB beyond refresh policy |

Session events: `SessionEstablished`, `SessionRevoked`, `SessionRefreshed` (security analytics optional).

---

## 6. JWT

### 6.1 Access Token Claims (Logical)

| Claim | Meaning |
| --- | --- |
| `sub` | Principal / User id |
| `tid` | Tenant id |
| `sid` | Session id |
| `iat` / `exp` | Issued / expiry |
| `iss` / `aud` | Proven issuer / API audience |
| `amr` / `acr` | Auth methods / assurance level (MFA) |
| Optional | `cid` client id for machine tokens |

**Do not** put permissions, full role lists, or PII dumps in JWT. Authorization is evaluated server-side (grants change faster than token life; tokens stay small).

### 6.2 Validation

- Signature (asymmetric preferred: RS256/ES256)  
- `iss`, `aud`, `exp`, clock skew bound  
- Session still active (`sid` lookup / bloom deny-list for revoked)  
- Tenant active / license not suspended  

### 6.3 API Keys vs JWT

API keys are **not** JWTs: opaque secrets, hashed at rest, presented as Bearer/`X-Api-Key`, resolved to service principal + grant scopes. Rotation and expiry via Admin workflows.

---

## 7. MFA

| Aspect | Design |
| --- | --- |
| **Factors** | TOTP (primary); WebAuthn/passkeys (preferred where available); backup codes (one-time, hashed) |
| **Enrollment** | User settings; admin can require; recovery via verified channel + step-up |
| **Mandatory** | Tenant policy: all users **or** privileged roles (tenant admin, role admin, API key admin, export, COR finalize) |
| **SSO** | Honor IdP MFA when `amr`/`acr` indicates; optionally require Proven MFA for break-glass local accounts |
| **Guest sign** | Not MFA of platform account; package token + optional signer identity capture |
| **Recovery** | Admin-assisted reset audited; invalidate old factors |
| **Storage** | TOTP secrets encrypted at rest; never logged |

---

## 8. Authorization

### 8.1 Authority

All decisions go through **Core `AuthzApi.Authorize(principal, permission_code, scope)`** (and list-scope helpers). Modules never invent parallel permission systems.

Deny → `403`; missing auth → `401`; module entitlement off → `403` `module_disabled`.

### 8.2 RBAC

| Concept | Definition |
| --- | --- |
| **Permission** | Atomic capability code (`safety.activity.create`, `documents.version.publish`, …) |
| **Role** | Named bundle of permissions (system roles + tenant-custom where allowed) |
| **Grant** | Role assigned to principal **within a Scope** |

**System role examples (illustrative)**

| Role | Typical use |
| --- | --- |
| `platform_ops` | Proven internal ops (break-glass, env-scoped) |
| `tenant_admin` | Tenant configuration, branding, IdP |
| `company_admin` | Company/org administration |
| `project_admin` / `pm` | Project configuration |
| `safety_coordinator` | Safety cases, CA, incidents |
| `supervisor` | Crew actions, reviews |
| `worker` | Field My Actions |
| `equipment_manager` | Assets, maintenance, readiness |
| `training_admin` | Courses, assignments |
| `document_control` | Controlled docs |
| `cor_admin` | COR engagements |
| `auditor_readonly` | Evidence read / package view |
| `integration` | API client role |

### 8.3 Scopes

| Scope | Meaning |
| --- | --- |
| **Tenant** | Entire customer workspace |
| **OrgUnit** | Org subtree (ancestor rules defined) |
| **Project** | Place / construction project |
| **Team** | Operational team |
| **Self** | Own person/user resources only |

**Project Membership** (Core) is an authorization binding, not Projects lifecycle. GC/Sub visibility = participation + least privilege—not shared tenant-wide admin.

### 8.4 ABAC (Attribute-Based Constraints)

RBAC answers “has permission in scope?” ABAC adds **attribute conditions** evaluated with the grant:

| Attribute class | Examples |
| --- | --- |
| **Resource** | `project.status`, `document.classification`, `incident.severity`, `file.access_class`, `signature.package_state` |
| **Subject** | `assurance_level` (MFA), `person.trade`, `membership.active`, `license.seat` |
| **Environment** | `ip_allowlist`, `network_zone`, `time_window` (optional tenant policies) |
| **Action sensitivity** | Medical note access, PII export, seal void, role grant |

**Policy composition (logical)**

```text
Allow iff
  principal authenticated
  ∧ tenant active
  ∧ license/feature allows module
  ∧ RBAC grant covers permission + scope
  ∧ ABAC constraints on resource/subject/env pass
  ∧ resource not in forbidden state (e.g., sealed immutability)
```

UI may hide controls; **server re-checks** every command/query.

### 8.5 List Filtering

List endpoints filter by authorized scopes at query level. Client-supplied project filters are never trusted as the sole boundary.

### 8.6 Postgres RLS

Application sets `app.tenant_id` (and related GUCs) per request. RLS is **defense in depth**, not a substitute for AuthzApi. Cross-tenant reads fail closed.

---

## 9. Audit Logging

### 9.1 Principles

- **Append-only** Core `AuditEntry` for significant actions  
- Sync write in the same transactional boundary as the command when feasible  
- Bus fan-out optional for analytics—**Core audit is SoR**  
- No passwords, TOTP secrets, magic-link secrets, signature stroke raw data, or medical note bodies in audit payloads  

### 9.2 Minimum Fields

| Field | Purpose |
| --- | --- |
| `tenant_id`, `actor_principal_id` | Who |
| `action`, `resource_type`, `resource_id` | What |
| `project_id?`, `correlation_id` | Context |
| `session_id?`, `amr/acr?` | Auth context |
| `ip` / `user_agent` | Policy-limited retention |
| `outcome` | Success / deny / failure |
| `before/after` refs | Diff pointers or version ids—not full PII dumps |
| `occurred_at` | Time |

### 9.3 Must-Audit Classes

Auth (login fail/success, logout, MFA enroll/reset), role/grant changes, membership changes, document publish/withdraw, signature seal/void, CA close, incident close, COR finalize, API key lifecycle, exports of personal data, file quarantine, admin builder publish, break-glass access.

### 9.4 Integrity

Tamper-evident storage (append-only table, restricted roles, optional hash-chain/WORM export for high-assurance tenants). Retention separate from operational soft-delete.

---

## 10. Encryption

| Layer | Control |
| --- | --- |
| **In transit** | TLS 1.2+ everywhere (browser↔CF↔origins, API↔Postgres/Redis/NATS/R2/Temporal/providers) |
| **At rest** | Provider encryption for Postgres, Redis, R2, ClickHouse volumes/objects |
| **Application** | Encrypt highly sensitive fields when required (MFA secrets, guest token hashes already one-way; optional field encryption for medical notes) |
| **Object storage** | R2 SSE; private buckets; presigned URLs short-lived, method/path constrained |
| **Backups** | Encrypted backups; access audited |
| **Key management** | Cloud KMS / platform secrets; app keys never in repo |
| **Client** | No encryption theater in browser as SoR; HTTPS + secure cookies |

---

## 11. Secrets Management

| Secret class | Storage | Rotation |
| --- | --- | --- |
| DB, Redis, NATS, Temporal | Fly secrets / secret store | Scheduled + on incident |
| R2 access keys | Secret store; prefer temporary credentials | Rotate |
| IdP client secrets | Secret store per env | Rotate with IdP |
| Provider API keys (email, WhatsApp, OCR) | Secret store | Rotate; dual-key during cutover |
| JWT signing keys | KMS/JWKS; key id (`kid`) | Regular rotation; overlap |
| API keys (customer) | **Hash only** at rest | Customer/admin rotate; expiry workflow |
| Guest/magic tokens | **Hash only**; plaintext once to channel | TTL + revoke |
| CI | GitHub Actions secrets | Separate from prod; OIDC to cloud where possible |

**Never:** secrets in git, Docker images, OpenAPI examples, NATS events, Temporal payloads (beyond ids), client bundles, support screenshots without redaction.

Workers load secrets at process start from platform injection—not from Postgres as plaintext.

---

## 12. Cloudflare

| Capability | Use in Proven |
| --- | --- |
| **DNS** | Authoritative DNS for product domains |
| **TLS** | Edge certificates; origin TLS to Vercel/Fly |
| **WAF** | OWASP rulesets, custom rules for auth/sign paths |
| **DDoS / Bot** | Absorb volumetric; challenge suspicious bots on login/guest |
| **CDN** | Public marketing/static only; **no** authenticated JSON caching |
| **R2** | Private object storage; no public bucket ACLs for tenant data |
| **Rate limiting (edge)** | Auth, guest redeem, password reset |
| **Zero Trust / Access** | Staging, admin portals, internal tools |
| **Logging** | WAF/firewall events to SIEM/ops (retention policy) |

**Public surface:** Web, API, OIDC callbacks, guest redeem, short-lived R2 presigns. Admin and staging behind Access where practical.

Cloudflare does **not** replace application AuthZ.

---

## 13. OWASP Alignment (Top Risks)

| OWASP risk | Proven control summary |
| --- | --- |
| **Broken Access Control** | AuthzApi + scopes + RLS; IDOR tests; deny by default |
| **Cryptographic Failures** | TLS; hashed passwords/tokens; no sensitive data in JWT/events |
| **Injection** | Parameterized SQL (SQLx); no string-built queries; strict content types |
| **Insecure Design** | Threat model (§20); Temporal compensation without evidence deletion |
| **Security Misconfiguration** | Hardened defaults; separate envs; minimal public surface |
| **Vulnerable Components** | SCA in CI; patch SLAs |
| **Auth Failures** | MFA, lockout, session revoke, SSO preferred |
| **Software & Data Integrity** | Signed releases; sealed evidence; checksums on files |
| **Logging/Monitoring Failures** | Audit SoR + security alerts on auth anomalies |
| **SSRF** | Block worker/API fetches to internal metadata; allowlist egress |
| **XSS** | React escaping; CSP; sanitize untrusted HTML (rare) |
| **CSRF** | SameSite cookies; CSRF tokens if cookie session for state-changing browser calls |

API security also follows **OWASP API Top 10**: object-level authz, function-level authz, mass assignment prevention (explicit DTOs), unrestricted resource consumption (rate limits), unsafe API consumption (workers).

---

## 14. Input Validation

| Layer | Responsibility |
| --- | --- |
| **Client (Zod/RHF)** | UX shape only — **non-authoritative** |
| **API DTO validation** | Types, lengths, enums, formats, required fields |
| **Application** | Cross-field rules, existence, AuthZ |
| **Domain** | Aggregate invariants (authoritative) |

**Rules**

- Explicit allowlists for enums and status transitions  
- Max payload sizes; reject unexpected fields (no mass assignment)  
- UUID path params validated before DB  
- Locale/timezone normalized server-side  
- File metadata: content-type allowlist ≠ trust; verify magic bytes after upload  
- Guest tokens: constant-time compare on hash  

Errors: stable problem codes; no stack traces or SQL to clients.

---

## 15. File Upload Security

### 15.1 Flow

```text
Authorize → CreateFileUploadIntent (Core)
  → Presigned PUT to R2 (short TTL, content-length/type constraints)
  → CompleteFileUpload (checksum verify)
  → Async AV / content processing workflow
  → Available | Quarantined
```

### 15.2 Controls

| Control | Spec |
| --- | --- |
| **AuthZ** | Intent creation requires permission; download via `AuthorizeFileAccess` |
| **Presign** | Method, key prefix (`tenant_id/...`), max size, expiry minutes |
| **Checksum** | Client-declared hash verified on complete |
| **Type allowlist** | Per feature (images, PDF, office)—tenant policy may tighten |
| **Magic-byte check** | After upload; mismatch → quarantine |
| **Path safety** | Server-generated object keys only |
| **No execute** | Objects never served as executable from app origin |
| **Download** | Authenticated redirect or short-lived GET presign; disposition safe |
| **Retention** | Class-based lifecycle; legal hold hooks |
| **Quarantine** | Unavailable to normal download; admin review |

Controlled documents add Documents-module meaning; **bytes** still Core FileObject + R2.

---

## 16. Virus / Malware Scanning

| Aspect | Design |
| --- | --- |
| **When** | After successful upload complete; before `Available` |
| **How** | `FileMediaProcessingWorkflow` → Go I/O activity (AV engine / cloud scanner) |
| **On clean** | Mark Available; optional image derivatives/OCR |
| **On detect / scan fail (policy)** | `QuarantineFileObject`; notify uploader/admin; audit |
| **Retries** | Transient scanner errors retry; permanent fail → quarantine or fail-closed per policy |
| **Workers** | No domain authority—report result to Core API |
| **Exports / inbound email attachments** | Same pipeline if stored as FileObjects |

OCR text is **candidate only**; modules validate before acceptance as compliance fact.

---

## 17. Rate Limiting

| Layer | Target |
| --- | --- |
| **Cloudflare** | IP / path: login, refresh, guest redeem, password reset |
| **API (Redis)** | Per principal, per IP, per tenant; stricter on writes and seal endpoints |
| **Auth** | Credential stuffing protection; CAPTCHA/bot challenge at edge when triggered |
| **Guest sign** | Per link + per IP |
| **Exports / reports** | Concurrent job caps per tenant |
| **Webhooks / API keys** | Per key quota |
| **Workers → providers** | Respect provider limits; backoff |

Responses: `429` with retry semantics; auth endpoints do not leak whether user exists via timing where avoidable.

---

## 18. Privacy — GDPR & PIPEDA

Proven operates in **Canada, US, AU, NZ**; privacy design targets **PIPEDA** (and applicable provincial laws) and **GDPR** readiness for EU data subjects when processed.

### 18.1 Principles

| Principle | Application |
| --- | --- |
| **Lawful purpose** | Safety/compliance operations, employment/contractor coordination, audit evidence |
| **Minimization** | Collect only needed fields; medical notes restricted permission + audit |
| **Limitation** | No secondary use of evidence for unrelated marketing |
| **Accuracy** | Corrections via domain commands; sealed evidence void+replace patterns |
| **Retention** | Per record class (sessions short; audit/evidence long; guest tokens short) |
| **Safeguards** | Controls in this document |
| **Openness** | Tenant-facing privacy notices; DPA with customers |
| **Individual access** | Export/access workflows for subject requests |
| **Consent** | Where required (e.g., WhatsApp), Notifications module gates channel |

### 18.2 Roles

| Role | Typical |
| --- | --- |
| **Customer (tenant)** | Controller for workforce/project personal data |
| **Proven** | Processor (or controller for platform account data)—contract clarifies |
| **Subprocessors** | Vercel, Fly, Cloudflare, email/WhatsApp, etc.—listed in DPA |

### 18.3 Data Subject Rights (Operational)

| Right | Mechanism (architectural) |
| --- | --- |
| **Access / Portability** | Admin/subject export jobs (Analytics/Export workflows); scoped AuthZ |
| **Rectification** | Domain update commands |
| **Erasure** | Soft-delete + retention policy; **legal/compliance hold** may defer erasure of sealed safety evidence—document lawful basis |
| **Restriction / Objection** | Flags + access denial where applicable |
| **Breach notification** | Incident runbook; customer notify timelines per law |

### 18.4 Cross-Border

Region-aware topology; document storage region commitments; SCCs/appropriate transfer tools when GDPR applies; tenant data residency options as product matures.

### 18.5 Events & Analytics

Event catalog forbids secrets and minimizes PHI; ClickHouse facts prefer pseudonymous ids; raw PII exports are audited high-sensitivity actions.

---

## 19. Password Policy

| Rule | Spec (baseline; tenant may tighten) |
| --- | --- |
| **Length** | Minimum 12 characters (prefer 15+ for admins) |
| **Complexity** | Blocklist common passwords; encourage passphrase; optional charset rules without absurd composition theater |
| **History** | Disallow last N passwords |
| **Hashing** | Modern KDF (Argon2id or bcrypt/scrypt with appropriate params)—never reversible |
| **Storage** | Hash + unique salt; pepper optional in KMS |
| **Change** | Require current password or MFA; revoke other sessions |
| **Reset** | Time-boxed single-use token; rate-limited; does not reveal account existence unnecessarily |
| **SSO tenants** | Local passwords disabled or break-glass only |
| **Service accounts** | No shared human passwords; API keys / client credentials |

Password never logged, never in audit `before/after`, never in support tools plaintext.

---

## 20. Threat Modeling

### 20.1 Method

STRIDE per trust boundary + asset. Assets: tenant data, evidence integrity, credentials, availability of field flows, COR packages, guest sign links.

### 20.2 Key Threats & Mitigations

| Threat | Example | Mitigations |
| --- | --- | --- |
| **Spoofing** | Stolen password/API key | MFA, short JWT, session revoke, key hash+rotate, SSO |
| **Tampering** | Alter sealed FLHA / signature | Immutability; void-only; checksums; audit; R2 versioning |
| **Repudiation** | Deny approval | Audit + signature evidence certificates; `sid`/`amr` on sensitive acts |
| **Information disclosure** | IDOR across project/tenant | AuthzApi, RLS, list scope filters, guest token scope |
| **DoS** | Credential stuffing / seal spam | Cloudflare + Redis limits; WAF; idempotency |
| **Elevation of privilege** | Self-assign tenant_admin | Dual control / step-up MFA; grant change audit; least privilege roles |
| **SSRF** | Worker fetch internal cloud metadata | Egress allowlists; no user-controlled URLs without validation |
| **Magic-link theft** | Forwarded guest email | Short TTL, single-use, bind to package/slot, revoke, anomaly alerts |
| **Offline replay** | Replay stale field mutation | Idempotency keys; server authority; conflict detection |
| **Supply chain** | Compromised dependency | SCA, lockfiles, signed CI, minimal attack surface |
| **Insider** | Over-broad support access | Break-glass audited; Zero Trust admin; least privilege ops roles |
| **Malware upload** | Weaponized PDF | AV quarantine; type checks; no inline execute |

### 20.3 Abuse Cases (Product)

- Sub tries to read GC-only documents → deny by participation + doc ACL.  
- Worker elevates via client-side role claim → JWT has no roles; AuthZ denies.  
- Quarantined file linked into COR package → package assembly skips/fails closed.  
- Revoked user with stolen refresh → session revoke list denies.

### 20.4 Security Testing Cadence

| Activity | Cadence |
| --- | --- |
| SAST / dependency scan | CI every PR |
| Secret scan | CI |
| AuthZ integration tests (IDOR matrix) | CI + release |
| Pen test | At least annually / major release |
| WAF rule review | Quarterly |
| Access review (admin grants) | Quarterly tenant + platform ops |
| Tabletop breach / ransomware | Annually |

---

## 21. Service-to-Service & Worker Security

| Control | Spec |
| --- | --- |
| **Auth** | Service credentials; constrained permissions |
| **Impersonation** | Forbidden by default; if required, explicit audited reason + short TTL |
| **Network** | Private networking where possible; no public worker admin |
| **Payload** | Tenant id must match credential scope |
| **Temporal** | Activity payloads without secrets; workflow ids not authorization |
| **NATS** | Subject ACLs; no PII-heavy payloads |

---

## 22. Frontend Security

- No business AuthZ in React  
- No long-lived secrets in bundles  
- CSP, Trusted Types where feasible  
- `Cache-Control: private, no-store` for authenticated JSON  
- Guest sign routes isolated from app shell privileges  
- PWA: encrypt-at-rest reliance on device OS; clear on logout  

---

## 23. Secure Development & Operations

| Practice | Requirement |
| --- | --- |
| **Environments** | Isolated staging/prod; separate R2/DB/secrets |
| **IaC review** | Fly / Cloudflare / Vercel config reviewed |
| **Change management** | Production access audited |
| **Vulnerability SLA** | Critical: expedited; High: defined days |
| **Incident response** | Runbooks for credential leak, ransomware, data exposure |
| **DR** | Backups encrypted; restore tested; RPO/RTO documented |
| **Logging** | Centralize API/WAF/auth anomalies; alert on spikes |

---

## 24. Control Mapping Summary

| Topic | Primary controls |
| --- | --- |
| Authentication | Password/MFA, OIDC, API keys, guest tokens, service auth |
| Authorization | Core AuthzApi |
| RBAC | Roles, permissions, scoped grants |
| ABAC | Resource/subject/env constraints + license/flags |
| JWT | Short-lived access; session-backed revoke |
| MFA | TOTP/WebAuthn; mandatory for privileged |
| Audit | Append-only Core audit |
| Encryption | TLS + at-rest + field secrets |
| Secrets | Platform secret store / KMS; hashed API/guest tokens |
| Cloudflare | WAF, DDoS, bot, R2, Access, edge RL |
| OWASP | §13 controls |
| Input validation | DTO + domain; client non-authoritative |
| File upload | Presign, checksum, type, AuthZ |
| Virus scanning | Async AV → Available/Quarantine |
| Rate limiting | Cloudflare + Redis API |
| GDPR / PIPEDA | Minimization, DPA, rights workflows, retention/holds |
| Session | Server session SoR; rotate; logout-all |
| Password | Length, KDF, history, reset hygiene |
| Threat modeling | STRIDE + continuous testing |

---

## 25. Success Criteria

Security architecture succeeds when:

1. Every request is authenticated (except minimal public surface) and authorized via Core.  
2. Cross-tenant and cross-project IDOR attempts fail under RLS + AuthZ.  
3. Evidence integrity holds under void/compensate patterns.  
4. Secrets never appear in git, events, or client bundles.  
5. Malicious uploads quarantine before general availability.  
6. Privacy rights and retention are operable without destroying lawful compliance holds.  
7. Edge + API rate limits keep auth and seal endpoints resilient.  
8. Privileged actions require appropriate assurance (MFA/step-up) and leave an audit trail.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Enterprise Security Architecture | Initial Proven security design |

---

*End of Enterprise Security Architecture*
