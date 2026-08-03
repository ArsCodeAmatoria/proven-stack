# Proven — Architectural Security Review

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Enterprise Architectural Security Review |
| **Version** | 1.0 |
| **Status** | Draft |
| **Reviewer role** | Enterprise Security Consultant |
| **Scope** | Foundation architecture documentation (no application code review) |
| **Last updated** | 2026-08-03 |
| **Inputs** | Security, AuthN, AuthZ, Audit, R2, Postgres, PWA/Offline, REST, Temporal, Deployment, Integrations, AI, Notifications, Observability, Testing |

---

## 1. Executive Verdict

Proven’s **documented** security posture is **strong for a compliance SaaS foundation**: clear trust boundaries, Core-centric AuthZ, append-only audit, sealed evidence immutability, private R2 with AV, edge controls, and explicit AI/human-review constraints.

The largest residual risks are **not missing principles** but **integration seams and operationalization**: Better Auth ↔ Core dual identity, Redis/NATS/Temporal hardening specifics, PWA offline token storage, webhook SSRF/agency data flows, and PIPEDA operational runbooks (DPRAs, retention enforcement, subprocessors).

**Overall residual risk (architecture-as-documented):** Medium — acceptable to proceed to scaffolding if recommendations below are tracked as security backlog before production data.

**No code was reviewed** (none implemented). Re-review when AuthN adapter, AuthZ middleware, and upload/AV paths land.

---

## 2. Review Method

| Lens | Applied |
| --- | --- |
| STRIDE | Spoofing, Tampering, Repudiation, Info disclosure, DoS, Elevation |
| OWASP Top 10 / API Top 10 | Mapping to controls |
| PIPEDA fair information principles | Mapping to design |
| Defense in depth | Edge → app → AuthZ → RLS → object |
| Trust boundary analysis | Guest, workers, AI, integrations |

Severity: **P0** (must fix before prod PII), **P1** (fix before GA), **P2** (harden within first releases), **P3** (improve).

---

## 3. Domain-by-Domain Findings

### 3.1 Authentication

| Strengths | Gaps / risks |
| --- | --- |
| Better Auth + Core session SoR; short JWT; refresh rotation + reuse detection | Dual-write AuthN identity can drift if adapter not transactional |
| MFA, step-up, OAuth PKCE, guest ≠ session | MFA mandatory defaults not prescribed as “on by default” for new tenants |
| Password policy, lockout, magic-link TTL | Device fingerprinting privacy vs tracking needs explicit DPIA |
| Offline refresh pause | Remember-me + PWA storage on shared devices remains abuse-prone |

**Recommendations**

1. **P0** — ADR: single transactional identity adapter (Better Auth ↔ Core User/Session); define conflict resolution if one side fails.  
2. **P1** — Tenant default: MFA required for privileged roles at provision; offer “all users” preset for enterprise.  
3. **P1** — Document session fixation / cookie flags matrix (SameSite, `__Host-`, CSRF for cookie-mutating browser calls).  
4. **P2** — Shared-device UX: warn + shorter idle for field phones; remote logout-all.

---

### 3.2 Authorization

| Strengths | Gaps / risks |
| --- | --- |
| Sole `AuthzApi`; scopes; ABAC; no perms in JWT | OrgUnit ancestor rules need precise formalization |
| Delegation + temporary grants with audit | Nested delegation forbidden—good; enforce in tests |
| Feature entitlement + RBAC composition | List endpoints leaking existence of restricted docs via timing/count |

**Recommendations**

5. **P0** — Automated AuthZ/IDOR suite as merge gate (already in Testing Strategy—treat as security exit criterion).  
6. **P1** — Formalize OrgUnit inheritance and Company-vs-Project visibility matrix with worked GC/Sub examples.  
7. **P1** — Uniform responses for restricted document miss vs deny where enumeration matters.  
8. **P2** — Dual control for granting `tenant_admin` / break-glass.

---

### 3.3 Storage (R2) & Files

| Strengths | Gaps / risks |
| --- | --- |
| Private buckets; server keys; Intent→Presign→AV→Available | Presign TTL/max-size must be enforced in implementation |
| Quarantine path; checksum; no public list | Orphan pending uploads if sweeper lags |
| Retention classes + legal hold concept | Object Lock/WORM optional—not mandated for evidence |

**Recommendations**

9. **P0** — AV fail-closed: never Available without clean scan (or explicit admin override audited).  
10. **P1** — Enable versioning on evidence prefixes; scheduled orphan reconcile FileObject↔R2.  
11. **P1** — Separate quarantine IAM; restrict human break-glass.  
12. **P2** — Consider Object Lock for certificate/evidence exports for high-assurance tenants.

---

### 3.4 Database

| Strengths | Gaps / risks |
| --- | --- |
| Tenant column + RLS; no cross-schema FKs; expand/contract | RLS bypass roles must be tightly controlled |
| Append-only audit; migrator vs app roles | Mis-set `app.tenant_id` GUC is classic footgun |
| Soft delete discipline | Analytics CH is separate—good; avoid dual SoR |

**Recommendations**

13. **P0** — Middleware must set RLS GUC from auth context only; integration tests prove cross-tenant deny even if query omits tenant predicate.  
14. **P1** — Continuous policy lint: RLS enabled + forced on all tenant tables.  
15. **P1** — Break-glass DB access: SSO, MFA, session recording, ticket required.  
16. **P2** — Column-level encryption ADR for highest-sensitivity fields (beyond provider at-rest).

---

### 3.5 PWA / Offline

| Strengths | Gaps / risks |
| --- | --- |
| Allowlist; server wins conflicts; no fake sealed | IndexedDB holds drafts/PII/photos—device theft risk |
| BG Sync best-effort; AuthZ on sync | Refresh token storage policy still “per Security”—needs concrete PWA choice |
| Guest online-only (good) | Malicious replay of outbox if device compromised |

**Recommendations**

17. **P0** — Logout/remote wipe clears IDB outbox/media; document MDM guidance for enterprise.  
18. **P1** — Prefer HTTP-only refresh cookies over IDB refresh; if IDB needed, encrypt with device-bound keys and short absolute TTL.  
19. **P1** — Outbox encryption-at-rest on device where platform allows; minimize cached ACL-sensitive docs.  
20. **P2** — Jailbreak/root detection is limited on PWA—rely on short sessions + step-up for seal.

---

### 3.6 API

| Strengths | Gaps / risks |
| --- | --- |
| `/api/v1`; problem codes; idempotency; AuthZ | Mass assignment / oversized payloads need hard limits |
| Guest surface minimal | BFF (Better Auth on Vercel) expands attack surface |
| Rate limits (Redis + CF) | Consistent 429 semantics under abuse |

**Recommendations**

21. **P0** — Threat model Better Auth routes on Vercel: CSRF, open redirect on OAuth, cookie theft.  
22. **P1** — Global body size limits; timeout; request ID; deny unknown JSON fields on write DTOs.  
23. **P1** — Separate rate-limit tiers for auth, seal, upload, search, AI.  
24. **P2** — API schema fuzzing in CI for parsers.

---

### 3.7 Temporal

| Strengths | Gaps / risks |
| --- | --- |
| Domain vs I/O queues; no secrets in payloads | Workflow args may still contain PII summaries |
| Compensation without deleting seals | Task queue ACLs / namespace isolation per env |
| Escalation patterns | Signal auth—who may signal `cancel`/`approved`? |

**Recommendations**

25. **P0** — Only API/service principals with AuthZ may start/signal workflows; workers cannot self-approve domain signals.  
26. **P1** — Payload minimization standard for workflow inputs (ids not blobs).  
27. **P1** — Separate Temporal namespaces staging/prod; mTLS/API keys rotated.  
28. **P2** — Query workflows for PII leakage in histories retention policy.

---

### 3.8 Redis

| Strengths | Gaps / risks |
| --- | --- |
| Cache-only mandate (strong) | Accidental use as session SoR would break revoke |
| Rate-limit backing | No AUTH/TLS/network policy detailed |

**Recommendations**

29. **P0** — Enforce: sessions/grants never Redis-authoritative; document forbidden key patterns in ADR.  
30. **P1** — Redis AUTH + TLS in-transit; private network only; no public bind.  
31. **P2** — Key prefix per env; flush protection on prod.

---

### 3.9 NATS

| Strengths | Gaps / risks |
| --- | --- |
| Outbox → events; no secrets in catalog | Subject ACL / multi-tenant subject design unspecified |
| Queue groups for workers | Malicious/compromised publisher injecting events |

**Recommendations**

32. **P0** — Only outbox publisher (API) writes domain subjects; workers consume with least privilege.  
33. **P1** — NATS authn (NKey/JWT/creds), TLS, per-env clusters; deny wild-card publish from workers.  
34. **P1** — Validate event schemas; poison → DLQ; ignore unauthenticated messages.  
35. **P2** — Tenant id mandatory in envelope; consumers re-check tenant on apply.

---

### 3.10 Cloudflare

| Strengths | Gaps / risks |
| --- | --- |
| WAF, bot, DDoS, Access for staging, R2 private | False sense of AuthZ at edge |
| Rate limit auth/guest | WAF bypass via alternate origins if Fly URL exposed |

**Recommendations**

36. **P0** — Do not publish raw Fly API hostnames; only Cloudflare (or controlled) origins; deny direct where possible.  
37. **P1** — Tuned WAF for `/api/auth`, guest redeem, webhooks; review quarterly.  
38. **P2** — Cloudflare Access for admin/staging; log shipping to SIEM.

---

### 3.11 Integrations

| Strengths | Gaps / risks |
| --- | --- |
| Framework: verify webhooks, idempotency, vault secrets, SSRF allowlist | Agency (WorkSafeBC / BC Crane) data minimization & legal basis |
| No domain SQL from connectors | OAuth token theft → wide Graph access if scopes over-broad |
| WhatsApp consent | Inbound reply command injection |

**Recommendations**

39. **P0** — SSRF: deny link-local/RFC1918/metadata endpoints on all outbound connector URLs.  
40. **P1** — Least-privilege Graph/agency scopes; admin consent review checklist.  
41. **P1** — Inbound WhatsApp/Teams: never execute free-form commands; map to allowlisted actions only.  
42. **P2** — DPRA/legal review before enabling regulated connectors per tenant jurisdiction.

---

### 3.12 AI

| Strengths | Gaps / risks |
| --- | --- |
| Assist-only; AuthZ on RAG; human review; tool allowlist | Prompt injection via retrieved docs |
| Tenant/model isolation; no medical in vectors | Provider subprocessors / residency |
| Audit completions | Over-retention of prompts |

**Recommendations**

43. **P0** — Enforce: AI cannot call write tools that mutate compliance SoR without human-accept path.  
44. **P1** — Prompt-injection harderning tests; treat chunk content as data not instructions.  
45. **P1** — DPA with model providers; residency options; disable AI for tenants without agreement.  
46. **P2** — Default store prompt/response **hashes**; full text only when needed under Restricted retention.

---

## 4. OWASP Alignment Assessment

| OWASP (Web/API) | Status | Notes |
| --- | --- | --- |
| Broken Access Control | **Designed well** | Needs IDOR test evidence at build-out |
| Cryptographic Failures | **Good** | Flesh out Redis/NATS TLS; field encryption ADR |
| Injection | **Good** | Parameterized SQL; no raw SQL tools for AI |
| Insecure Design | **Good** | Threat models exist; keep ADRs for AuthN adapter |
| Security Misconfiguration | **Medium** | Env hardening checklists still thin |
| Vulnerable Components | **Planned** | Dependabot/SCA in repo design—implement early |
| Auth Failures | **Good** | MFA/session revoke; lockout |
| Software/Data Integrity | **Good** | Sealed evidence; CI signing planned |
| Logging/Monitoring | **Good** | Audit + observability designs |
| SSRF | **Called out** | Must enforce in integrations/workers |
| XSS/CSRF | **Partial** | CSP planned; CSRF for cookie session needs explicit matrix |

---

## 5. PIPEDA Alignment Assessment

| Principle | Architecture fit | Gap |
| --- | --- | --- |
| Accountability | Audit, AuthZ, DPA intent | Appoint privacy roles; DPIA schedule |
| Identifying purposes | Compliance ops purpose stated | Per-feature purpose notices in product |
| Consent | WA/SMS/channel prefs | Document meaningful consent UX |
| Limiting collection | Event/AI minimization rules | Enforce OCR/AI corpus allowlists |
| Limiting use/disclosure | Module boundaries; no marketing from evidence | Integration egress allowlists |
| Accuracy | Void+replace seals | Subject access correction UX |
| Safeguards | TLS, RLS, R2 private, MFA | Operational encryption/key mgmt |
| Openness | Privacy policy TBD | Publish tenant-facing privacy doc |
| Individual access | Export workflows sketched | Implement DSAR runbook + timelines |
| Challenging compliance | — | Complaint handling process |

**PIPEDA recommendations**

47. **P0** — Data inventory (categories, systems, retention) before first Canadian production tenant.  
48. **P1** — DSAR (access/correction/deletion) runbook with legal hold exceptions for sealed safety evidence.  
49. **P1** — Subprocessor list (Vercel, Fly, Cloudflare, email, WA, LLM) in DPA.  
50. **P2** — Breach playbook with PIPEDA-aligned notification assessment.

---

## 6. Cross-Cutting Risks (Top 10)

| # | Risk | Sev |
| --- | --- | --- |
| 1 | AuthN adapter drift / session revoke not honored by API | P0 |
| 2 | RLS GUC not set → cross-tenant read | P0 |
| 3 | File Available without AV | P0 |
| 4 | Direct origin bypass of Cloudflare | P0 |
| 5 | NATS unauthenticated publish | P0 |
| 6 | Offline PWA data at rest on lost device | P1 |
| 7 | OAuth over-scoped Graph tokens | P1 |
| 8 | AI write path without human review | P0 |
| 9 | Temporal signal forgery | P0 |
| 10 | Incomplete DSAR/retention enforcement | P1 |

---

## 7. Prioritized Recommendation Backlog

### Before handling real PII (P0)

- [ ] AuthN↔Core adapter ADR + revoke tests  
- [ ] RLS forced + IDOR suite green  
- [ ] AV fail-closed on uploads  
- [ ] Lock down API origins (CF-only)  
- [ ] NATS auth + publisher restriction  
- [ ] Temporal start/signal AuthZ  
- [ ] AI no silent SoR writes  
- [ ] Data inventory + subprocessor DPA draft  

### Before GA (P1)

- [ ] Cookie/CSRF/MFA default matrix  
- [ ] Redis TLS/AUTH; session-not-in-Redis proof  
- [ ] PWA logout wipe + refresh storage decision  
- [ ] Webhook SSRF + WhatsApp allowlisted actions  
- [ ] R2 versioning + orphan sweeper  
- [ ] DSAR/legal hold runbooks  
- [ ] WAF tuning + Access for staging/admin  
- [ ] Security CI (SAST, secrets, SCA) live  

### First year hardening (P2–P3)

- [ ] Field-level encryption ADR  
- [ ] Object Lock for evidence tier  
- [ ] Pen test + tabletop breach  
- [ ] SIEM for auth anomalies  
- [ ] Privacy UX notices per feature  

---

## 8. What Is Working Well (Preserve)

1. Modular monolith with **no cross-module SQL** and Core AuthZ monopoly.  
2. **Sealed evidence** immutability + void compensation.  
3. **Guest signing** isolated from platform sessions.  
4. **Workers without domain authority**.  
5. **Audit append-only** as compliance SoR.  
6. **Offline honesty** (pending vs sealed).  
7. **AI assist + human review** framing.  
8. Defense in depth narrative: CF → API → AuthZ → RLS → R2.

---

## 9. Re-Review Triggers

Re-run this architectural review (and code-level security review) when:

- Better Auth adapter merges  
- First multipart upload/AV path ships  
- First external webhook connector enables  
- AI tools gain any write capability  
- Multi-region or new subprocessor added  

---

## 10. Conclusion

Architecturally, Proven is **designed like an enterprise compliance platform**, not a forms toy. Residual risk concentrates on **identity seam integrity**, **data-plane hardening (Redis/NATS/Temporal/R2)**, **edge origin discipline**, **device-side offline data**, and **privacy operations (PIPEDA)**.

Treat the P0 backlog as **release blockers for production PII**; keep the listed strengths invariant as code lands.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Enterprise Security Consultant | Full architectural security review |

---

*End of Architectural Security Review*
