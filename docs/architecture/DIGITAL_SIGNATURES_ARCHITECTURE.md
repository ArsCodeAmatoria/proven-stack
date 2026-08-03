# Proven — Enterprise Digital Signature Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Enterprise Digital Signature Architecture |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Digital Signature / Compliance Architecture |
| **Audience** | Engineering, Security, Legal/Compliance, Product |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Signatures Domain](./SIGNATURES_DOMAIN.md), [Documents Domain](./DOCUMENTS_DOMAIN.md), [Authentication](./AUTHENTICATION_ARCHITECTURE.md), [Audit Logging](./AUDIT_LOGGING_ARCHITECTURE.md), [Temporal Workflows](./TEMPORAL_WORKFLOWS.md), [Offline Sync](./OFFLINE_SYNC_ARCHITECTURE.md), [Security](./SECURITY_ARCHITECTURE.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document designs Proven’s **enterprise digital signature** capability: internal and guest signers, magic links, QR codes, email delivery, approval chains, document version validation, audit trail, hash validation, certificates, legal evidence posture, reminder workflows, and offline signing.

**Hard rules**

1. Signatures owns **proof of assent**; subject modules own **what was meant** (FLHA, SWP, inspection, …).  
2. Completed packages are **immutable**; corrections use **void + new package**.  
3. **Guest signing ≠ platform login** — package-scoped tokens only ([Authentication](./AUTHENTICATION_ARCHITECTURE.md)).  
4. **Auth magic links ≠ guest sign links**.  
5. Legal evidence = sealed package + hashes + identity assurance + Core audit + optional certificate—not a screenshot.

**Documentation only — no implementation.**

---

## 2. Capability Map

| Requirement | Design home |
| --- | --- |
| Internal users | Authenticated seal via session + AuthZ |
| Guest users | Guest principal + magic link / QR |
| Magic links | Signatures `MagicLink` (guest/auth-sign) |
| QR codes | `QrSignSession` |
| Email | Notifications delivery of links/reminders |
| Approval chains | Sequential/parallel slots + subject workflows |
| Version validation | Seal-time Documents query + content hash pin |
| Audit trail | Core Audit + signature projections |
| Hash validation | Subject + capture + manifest digests |
| Certificates | Evidence certificate PDF/artifact |
| Legal evidence | Evidence bundle + retention + immutability |
| Reminder workflows | Temporal + Notifications |
| Offline signing | Policy-gated capture + sync seal |

---

## 3. Core Concepts

| Term | Meaning |
| --- | --- |
| **Signature Package** | Multi-signer unit of work bound to a subject |
| **Signer Slot** | Required seal: role (author, reviewer, approver, crew, guest), order, assignee |
| **Seal** | Successful capture completing a slot |
| **SigningPolicy** | Tenant rules: assurance, guest allowed, order, expiry, offline, version pin |
| **Identity Assurance Record** | How identity was verified at seal |
| **Evidence Certificate** | Exportable proof artifact (manifest + seals + subject binding) |
| **Content / Subject Hash** | Digest of what was presented for assent |

```text
Subject module ──create package──► Signatures
                                      │
                 slots ←── policy ────┤
                                      ▼
              notify (email) / QR / in-app My Actions
                                      ▼
                         seal (online or offline sync)
                                      ▼
              version + hash checks ──► complete package
                                      ▼
                         certificate + audit + subject callback
```

---

## 4. Internal Users

| Aspect | Design |
| --- | --- |
| **AuthN** | Better Auth / Core session; access JWT with `sid`, `amr`/`acr` |
| **AuthZ** | `signatures.slot.seal_self` or assignee match + package permission; create/void need manage permissions |
| **UX** | My Actions queue; Place context; signature pad in authenticated shell |
| **IDV** | Session sufficient for standard; step-up MFA when policy/`acr` requires |
| **Binding** | Slot assignee = `user_id` and/or `person_id` |
| **Delegation** | If acting under AuthZ delegation, assurance record notes `delegation_of` |

Internal seal flow: open package → verify pending slot → present subject summary + hash → capture → server validates → seal → progress package.

---

## 5. Guest Users

| Aspect | Design |
| --- | --- |
| **Who** | External GC/client, sub contact, visitor without Proven account |
| **AuthN** | Opaque package/slot token only—**not** a Better Auth user session |
| **UI** | Isolated `guest/sign/[token]` layout—no Command Center |
| **Identity capture** | Name, email, optional company, optional OTP; stored on assurance record |
| **Scope** | Cannot call general API; only redeem, view bound subject snapshot, seal, decline |
| **Revocation** | Void package / revoke link / expiry |

Guest never receives tenant-wide roles.

---

## 6. Magic Links

### 6.1 Guest / signing magic links

| Property | Spec |
| --- | --- |
| **Issue** | Signatures API when slot is guest-eligible |
| **Storage** | **Hash only** at rest; plaintext shown once to email channel |
| **URL** | HTTPS app route with high-entropy token |
| **TTL** | Short; policy per SigningPolicy |
| **Use** | Single-use or multi-use until sealed/expired (policy)—prefer single-use redeem → short session cookie bound to package |
| **Rate limit** | Issue + redeem (Cloudflare + API) |
| **Audit** | Issued/redeemed/revoked/expired **without** secret |
| **Workflow** | `GuestSignatureWorkflow` / `MagicLinkSignatureWorkflow` |

### 6.2 Distinction from auth magic links

| Auth magic link (Better Auth) | Sign magic link (Signatures) |
| --- | --- |
| Logs into Proven | Opens one signing package |
| Creates user session | Guest/package scope only |

---

## 7. QR Codes

| Aspect | Design |
| --- | --- |
| **Purpose** | Field scan-to-sign (toolbox, gate, equipment tag context) |
| **Binding** | QR payload → `QrSignSession` → package (or creates package per policy) |
| **Documents** | Documents may own QR *targets*; Signatures owns session when purpose is seal |
| **TTL** | Session expiry independent of package expiry |
| **Security** | Non-guessable session id; optional one-time unlock; HTTPS only |
| **Flow** | Scan → land on sign UI → IDV per policy → seal |
| **Workflow** | `QrSignSessionWorkflow` |
| **Offline** | QR redeem generally requires network; pre-cached session optional later |

---

## 8. Email

| Use | Design |
| --- | --- |
| **Invite to sign** | Notifications sends template with magic link / deep link (token not logged) |
| **Reminders** | Temporal timers → Notifications |
| **Completion** | Optional notify requester/subject module watchers |
| **Delivery** | Go `notify-worker`; Signatures does not own SMTP |
| **Consent** | Channel prefs / quiet hours honored by Notifications |

Email is a **transport**, not an assurance factor by itself (unless OTP-in-email policy explicitly used).

---

## 9. Approval Chains

### 9.1 Slot ordering

| Mode | Behavior |
| --- | --- |
| **Sequential** | Slot N unlocked only after N−1 sealed |
| **Parallel** | Any pending required slot may seal |
| **Mixed** | Stages of parallel groups in sequence |

### 9.2 Roles on slots

Author · Reviewer · Approver · Crew · Guest · Witness (policy).

### 9.3 Split of ownership

| Concern | Owner |
| --- | --- |
| Who must approve / business routing | Subject module + Temporal (e.g. DocumentApprovalWorkflow) |
| Capture of assent seals | Signatures package/slots |
| Document multi-step publish rules | Documents domain |

Pattern: Documents/Safety workflow **creates** signature packages at approval steps; Signatures **executes** seals; workflow **signals** on `SignaturePackageCompleted`.

### 9.4 Decline / reject

Slot decline → package failed/rejected path → subject workflow handles return-to-draft; audit both layers.

---

## 10. Document Version Validation

At **create** and again at **each seal** (and at package complete):

| Check | Rule |
| --- | --- |
| **Pinned version** | Package stores `DocumentVersionId` + **content hash** at create |
| **Still intended** | Documents API: version not withdrawn/superseded contrary to policy |
| **Hash match** | Recompute or compare stored hash of canonical content presented |
| **Failure** | Block seal; optionally void package; notify; audit `signatures.validation.failed` |

Non-document subjects (FLHA activity id, inspection id) pin **subject content hash / snapshot id** analogously.

---

## 11. Hash Validation

| Hash | Covers |
| --- | --- |
| **Subject content hash** | Canonical bytes/JSON of what was shown for assent |
| **Capture hash** | Stroke/image file checksum (Core FileObject) |
| **Slot seal digest** | H(subject_hash \|\| capture_hash \|\| identity_record \|\| timestamp \|\| slot_id) |
| **Package manifest hash** | Merkle/list digest of all slot digests + metadata |
| **Certificate hash** | Digest of certificate artifact |

### 11.1 Validation moments

- Upload complete (file checksum)  
- Seal (re-verify subject hash)  
- Certificate generation  
- Later verification API: re-hash files + compare manifest (tamper detection)

Failed validation → deny seal / mark certificate verify fail—never “fix” sealed data.

---

## 12. Audit Trail

### 12.1 Dual layer

| Layer | Content |
| --- | --- |
| **Core Audit** | `signatures.package.*`, `slot.sealed`, magic/QR lifecycle, void, certificate ([Audit Logging](./AUDIT_LOGGING_ARCHITECTURE.md)) |
| **Signatures projections** | Signer-facing history on package; assurance records |
| **Temporal history** | Operational timers/signals—not legal SoR |

### 12.2 Seal audit minimum

Actor (user or guest), package/slot/subject ids, assurance method, subject hash, capture file id, `sid`/`amr` if internal, IP/UA per policy, correlation id, outcome.

No stroke payloads in audit.

---

## 13. Certificates

| Aspect | Design |
| --- | --- |
| **Trigger** | Package completed → `EvidenceCertificateWorkflow` |
| **Content** | Package id, subject refs, each slot (name, time, assurance, digests), manifest hash, tenant branding |
| **Render** | Go PDF activity on `proven-io` |
| **Storage** | Core FileObject; immutable attach to package |
| **Verify** | API returns valid/invalid against current hashes |
| **Failure** | Package remains completed; cert status failed; retry |

Certificates **summarize** evidence; the package + files remain authoritative.

---

## 14. Legal Evidence Posture

### 14.1 Evidence bundle (logical)

1. Subject identity + content hash / version pin  
2. SigningPolicy snapshot at create  
3. Each seal: identity assurance, capture artifact, timestamps (captured_at + server received_at)  
4. Package manifest hash  
5. Core audit entries referencing package  
6. Optional evidence certificate PDF  
7. Chain of custody: void records if superseded  

### 14.2 Legal / compliance principles

| Principle | Application |
| --- | --- |
| **Intent** | Clear disclosure text in sign UI (“I acknowledge…”) from policy/subject |
| **Identity** | Session or guest capture + optional OTP/step-up |
| **Integrity** | Hashes + immutability + void-only corrections |
| **Time** | Server timestamps; offline captured_at disclosed |
| **Retention** | Long retention for signature evidence class; legal hold support |
| **Export** | Certificate + package export for counsel/auditor |
| **Jurisdiction** | Tenant region settings; not a substitute for counsel review |

Proven provides **technical evidence controls**; customers remain responsible for legal acceptance of e-sign in their jurisdictions.

### 14.3 What is not evidence

UI screenshots, email alone without seal, OCR guesses, unverified offline drafts.

---

## 15. Reminder Workflows

| Workflow | Role |
| --- | --- |
| `SignaturePackageWorkflow` | Remind pending authenticated slots; escalate |
| `GuestSignatureWorkflow` | Remind guest via email; expire link |
| `QrSignSessionWorkflow` | Session TTL; optional host notify |
| Subject workflows | FLHA/Document ack wait on package completion |

### 15.1 Reminder policy

| Knob | Example |
| --- | --- |
| Cadence | T+1d, T+3d, … |
| Channels | In-app + email |
| Escalation | Assignee manager / package requester |
| Stop | On seal, void, decline, or package complete |

Notifications send; Signatures owns “who is pending.”

---

## 16. Offline Signing

### 16.1 Policy gate

Allowed only when SigningPolicy enables offline for that subject type—typically **authenticated** workers with cached package/slot snapshot. Guest magic-link offline generally **denied**.

### 16.2 Flow

```text
Cache package + subject hash + slot
  → capture stroke → local media store
  → enqueue seal_slot mutation (pending seal UX—not “Proven”)
  → online: upload capture → seal API with offline flag + captured_at
  → server re-validates version/hash/order/AuthZ
  → ACK → certificate progress
```

### 16.3 Rules

| Rule | Detail |
| --- | --- |
| UX language | **Pending seal** until server ACK |
| Step-up MFA | Block offline if required assurance needs online challenge |
| Sequential slots | Cannot seal N offline if N−1 not known complete |
| Conflicts | Server sealed/voided wins; client conflict UI |
| Hash | Subject hash validated at sync; stale content → reject |

Align with [Offline Sync](./OFFLINE_SYNC_ARCHITECTURE.md).

---

## 17. End-to-End Scenarios

### 17.1 Internal multi-approver document

1. Documents approval workflow creates sequential Signatures package on version hash.  
2. Email/in-app to reviewers.  
3. Each seals; version validated each time.  
4. Package complete → certificate → Documents records approval seals → publish step may continue.

### 17.2 Guest client SWP ack

1. Ack campaign creates guest slot + magic link.  
2. Email via Notifications.  
3. Guest redeems, captures identity, seals.  
4. Audit + certificate; Documents marks ack complete.

### 17.3 Toolbox QR

1. Activity creates package; QR session for crew.  
2. Workers scan, sign in parallel.  
3. Reminders for missing; seal complete → Safety sealed.

### 17.4 Offline FLHA crew sign

1. Policy allows offline; captures queued.  
2. Sync seals; package completes; FLHA review workflow continues.

---

## 18. Security Controls

| Control | Apply |
| --- | --- |
| Token storage | Hash only |
| TLS | Everywhere |
| Rate limit | Issue/redeem/seal |
| AuthZ | Internal seal; void; create |
| Isolation | Guest cannot escalate |
| AV | Capture images through FileApi + scan |
| Secrets | Never in Temporal payloads/logs/events |

---

## 19. API Surface (Logical)

| Capability | API intent |
| --- | --- |
| Create package | Subject modules / authorized users |
| Issue magic link / QR | Signatures |
| Redeem guest | Public-ish scoped routes |
| Seal slot | Auth or guest token |
| Void package | Privileged + audit |
| Get status / verify hashes | AuthZ read |
| Certificate download | AuthZ + package complete |

See [REST API](./REST_API.md) signatures section for path catalog.

---

## 20. Testing Guidance

- Sequential unlock enforcement  
- Version hash mismatch blocks seal  
- Guest token scope isolation  
- Magic link single-use / expiry  
- Offline seal sync + conflict  
- Certificate verify after tamper simulation  
- Reminder stops on complete  
- Void prevents further seals  

---

## 21. Success Criteria

1. Internal and guest signers can complete policy-compliant seals with clear assurance records.  
2. Magic links and QR sessions are time-boxed, hashed, and audited without leaking secrets.  
3. Approval chains compose subject workflows with Signatures slots.  
4. Document/subject version and hash validation block stale seals.  
5. Certificates and manifests support legal evidence export and later verification.  
6. Reminders and offline signing behave safely without false “Proven/sealed” UX.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Digital Signature Architecture | Enterprise e-sign design |

---

*End of Enterprise Digital Signature Architecture*
