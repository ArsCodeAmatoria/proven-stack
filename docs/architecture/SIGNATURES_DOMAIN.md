# Proven — Digital Signature Domain Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Digital Signature Domain Architecture (DDD) |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Principal Architecture |
| **Audience** | Engineering, Product, Security, Legal / Compliance |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Domain Model](./DOMAIN_MODEL.md), [Core Domain](./CORE_DOMAIN.md), [Documents Domain](./DOCUMENTS_DOMAIN.md), [Safety Domain](./SAFETY_DOMAIN.md), [Training Domain](./TRAINING_DOMAIN.md), [Equipment Domain](./EQUIPMENT_DOMAIN.md), [UX Architecture](../ux/UX_ARCHITECTURE.md), [System Architecture](./SYSTEM_ARCHITECTURE.md), [PRD](../PRD.md) |

---

## 1. Purpose

This document defines the **Digital Signature** bounded context for Proven.

Digital Signatures is a **strategic core domain** of the Construction Compliance Operating System. It owns **proof of assent**: signature assignments, review/approval signing requests, multi-party packages, guest signing, magic links, QR signing, identity verification at seal time, audit-grade evidence, certificate/evidence package generation, document version validation, and reminder workflows.

Signatures never owns the business meaning of what was signed (FLHA, SWP, inspection, evaluation). Subject modules own meaning; Signatures owns **sealed evidence**.

**Architecture documentation only — no implementation.**

---

## 2. Bounded Context

### 2.1 Context Card

| Field | Value |
| --- | --- |
| **Name** | Digital Evidence (Signatures) |
| **Module** | `signatures` |
| **Strategic type** | Core domain (differentiating) |
| **Product metaphor** | Seal = identity-bound, time-bound, version-bound proof of assent |
| **System of record for** | Signing policies, signature packages, signer slots/assignments, captured signatures, identity assurance records at seal, magic links, QR sign sessions, evidence certificates/artifacts, signature audit projections, reminder workflow state metadata |
| **Not system of record for** | Document content/versioning (Documents), safety/training/equipment subject records, user credentials (Core), file byte storage service (Core Files holds blobs; Signatures holds evidence refs + manifests), notification delivery (Notifications) |

### 2.2 Context Map

```text
Subject modules: Documents · Safety · Training · Equipment · COR (as consumer)
Core: AuthN/AuthZ · People link · Files · Platform Audit
        │
        ▼
┌────────────────────────────────────────────┐
│              SIGNATURES                    │
│  Policies · Packages · Links · QR · IDV    │
│  Evidence Certificates · Reminders         │
└──────────────────┬─────────────────────────┘
                   │
     Notifications · Workflows · COR · Analytics · Subject callbacks
```

### 2.3 Ubiquitous Language

| Term | Meaning |
| --- | --- |
| **Signature Package** | Unit of work for one or more signers against a subject |
| **Signer Slot / Assignment** | Who must sign, in what order, with what role (author, reviewer, approver, crew, guest) |
| **Review Request** | Package purpose requiring reviewer seal before completion |
| **Approval** | Package purpose / slot role for formal approve-to-proceed seal |
| **Guest Signing** | Signer without full Proven account access |
| **Magic Link** | Time-boxed secret URL granting access to a single signing session |
| **QR Signing** | Scan-initiated signing session bound to a package/subject |
| **Identity Verification (IDV)** | Assurance checks performed before/at seal (auth session, OTP, guest identity capture, step-up) |
| **Evidence Artifact / Certificate** | Immutable exportable proof bundle (manifest + signature data + subject binding) |
| **Document Version Validation** | Ensure subject DocumentVersion is still the intended/effective version at seal |
| **Seal** | Successful capture completing a signer slot |
| **Proven / Completed Package** | All required slots sealed |

---

## 3. Responsibilities (Mapped to Ownership)

| Responsibility | Signatures owns? | Clarification |
| --- | --- | --- |
| **Assignments** | Yes | Signer slots on packages |
| **Review Requests** | Yes (as package purpose/slots) | Subject module requests review seal; Signatures executes |
| **Approvals** | Yes (approval-type slots) | Not Documents approval workflow itself—**signature** of approval |
| **Guest Signing** | Yes | Guest sessions + identity capture |
| **Magic Links** | Yes | Issue/validate/expire/redeem |
| **QR Signing** | Yes | QR session bind/resolve/complete |
| **Identity Verification** | Yes (seal-time assurance) | Collaborates with Core AuthN for user step-up; does not replace IdP |
| **Audit Trail** | Yes (signature audit) + Core Audit | Dual layer |
| **Certificate Generation** | Yes | Evidence certificate/artifact generation |
| **Document Version Validation** | Yes (at seal) | Queries Documents effective/intended version APIs |
| **Reminder Workflows** | Orchestrates | Temporal + Notifications |

Documents may issue QR *targets* for documents; Signatures owns the signing session and evidence when the purpose is seal.

---

## 4. Aggregate Roots

| Aggregate | Responsibility |
| --- | --- |
| **SigningPolicy** | Tenant rules by process type / subject type: assurance level, order, expiry, guest allowed, version pin rules |
| **SignaturePackage** | Subject binding, slots, status, completion/void/expiry |
| **MagicLink** | Single-purpose link credential for a package/slot |
| **QrSignSession** | QR-initiated session bound to package (or creates package per policy) |
| **IdentityAssuranceRecord** | Snapshot of how identity was verified for a seal |
| **EvidenceCertificate** | Generated certificate/artifact for a completed package |
| **SignatureAuditProjection** | Queryable signature history (optional projection) |

---

## 5. Entities

### 5.1 Under SignaturePackage

| Entity | Description |
| --- | --- |
| **SubjectBinding** | Typed subject ref + version pins (DocumentVersionId, content hash, activity revision, etc.) |
| **SignerSlot** | Role, order index, assignee (User/Person/Guest), status |
| **CapturedSignature** | Seal event: stroke/image ref, typed name, timestamp, IP/UA policy fields |
| **SlotDecline** | Optional decline with reason |
| **PackageAttachmentRef** | Snapshot of displayed content hash / render |
| **ReminderState** | Last reminded at / count (or workflow-owned) |

### 5.2 Under Magic Link / QR

| Entity | Description |
| --- | --- |
| **LinkSecretHash** | Store hash only—never raw secret at rest |
| **Redemption** | When/who/device redeemed |
| **QrPayloadBinding** | Code → session/package |
| **SessionChallenge** | OTP/step-up challenge state if required |

### 5.3 Under IDV & Certificate

| Entity | Description |
| --- | --- |
| **AssuranceCheck** | Checks performed (session auth, email OTP, SMS OTP, shared code, ID fields) |
| **GuestIdentityCapture** | Name, company, email/phone, attestation |
| **CertificateManifest** | Hashes of subject snapshot, signatures, policy version |
| **CertificateArtifactRef** | FileObjectId of generated PDF/JSON evidence pack |

---

## 6. Value Objects

- `SignaturePackageId`, `SignerSlotId`, `MagicLinkId`, `QrSignSessionId`
- `EvidenceCertificateId`, `IdentityAssuranceId`
- `SubjectRef` — `{ type, id, versionId?, contentHash? }`
- `SubjectType` — DocumentVersion | SafetyActivity | Inspection | EvaluationAttempt | TrainingCompletion | PermitCase | LiftPlan | Custom
- `ProcessType` — Acknowledge | Review | Approve | Attendance | InspectionSignOff | Custom
- `SigningOrder` — Parallel | Sequential
- `PackageStatus` — Draft | Pending | PartiallySigned | Completed | Voided | Expired | Declined
- `SlotStatus` — Pending | Sealed | Declined | Expired | Skipped
- `AssuranceLevel` — Low | Standard | High | StepUp
- `SignerKind` — User | Person | Guest
- `LinkStatus` — Active | Redeemed | Expired | Revoked
- `SignatureBlobRef` / `FileObjectId`
- `SignedAt`, `ExpiresAt`
- `DeviceSessionMeta` (policy-limited)
- `PolicyId`, `PolicyVersion`
- `VoidReason`, `DeclineReason`

---

## 7. Relationships

```text
SigningPolicy ──governs──► SignaturePackage (by ProcessType/SubjectType)

Subject Module (Documents/Safety/Training/Equipment)
        │ CreateSignaturePackage(subject, slots, policy)
        ▼
SignaturePackage
        ├── SubjectBinding (version/hash pins)
        ├── SignerSlot * ──► User/Person/Guest
        │       ├── CapturedSignature ──► FileObject (stroke/image)
        │       └── IdentityAssuranceRecord
        ├── MagicLink * (per slot or package)
        ├── QrSignSession *
        └── EvidenceCertificate (on complete)

Completed package events ──► subject module callbacks / consumers
                         ──► COR provenance
                         ──► People signature history refs
                         ──► Core Audit
```

### 7.1 Review / Approval Relationship

```text
Documents ApprovalCase / Safety Review
  = business workflow decisioning (subject module + Temporal)
        │ may require
        ▼
Signatures Approval/Review slots
  = cryptographic/assent evidence that a person sealed the decision
```

Do not conflate Documents multi-step approval routing with SignaturePackage slots—routing lives in subject/workflow; seal lives here.

---

## 8. Domain Events

- `SigningPolicyChanged`
- `SignaturePackageCreated`
- `SignaturePackageUpdated`
- `SignerAssigned` / `SignerReassigned`
- `MagicLinkIssued` / `MagicLinkRedeemed` / `MagicLinkRevoked` / `MagicLinkExpired`
- `QrSignSessionOpened` / `QrSignSessionCompleted` / `QrSignSessionExpired`
- `IdentityAssuranceCaptured`
- `SignatureCaptured`
- `SignerDeclined`
- `SignaturePackagePartiallyCompleted`
- `SignaturePackageCompleted`
- `SignaturePackageVoided`
- `SignaturePackageExpired`
- `EvidenceCertificateGenerated`
- `DocumentVersionValidationFailed`
- `SignatureReminderSent` *(or notification-only)*

---

## 9. Business Rules

### 9.1 Package Lifecycle

1. Packages bind to a subject at creation; subject type must be allowed by policy.  
2. Required slots must all reach `Sealed` for `Completed`.  
3. Sequential order: later slots cannot seal before earlier required slots.  
4. Parallel order: any pending slot may seal.  
5. Completed packages are immutable; corrections require void + new package.  
6. Void requires permission + reason; emits events to subject modules.  
7. Expiry auto-transitions Pending/Partial → Expired via workflow.

### 9.2 Assignments

1. Slots assign to User, Person, or Guest identity.  
2. Reassignment voids outstanding magic links for the old assignee.  
3. Duplicate active packages for same subject+process may be prevented by policy (idempotent create).

### 9.3 Document Version Validation

1. When subject is `DocumentVersion`, package stores `DocumentVersionId` + content hash at create.  
2. At each seal (and at completion), Signatures validates via Documents APIs:  
   - version exists  
   - not withdrawn  
   - if policy `RequireEffectiveAtSeal`: version is effective *now*  
   - if policy `PinExactVersion`: version id unchanged and hash matches  
3. Validation failure blocks seal and emits `DocumentVersionValidationFailed`.  
4. Floating “always latest” is **opt-in** and discouraged for audit-grade SWP/policy acks; when enabled, re-bind hash at seal and record both intended and sealed versions.

### 9.4 Guest Signing & Magic Links

1. Guest allowed only if SigningPolicy permits for process type.  
2. Magic links are single-package (preferably single-slot), time-boxed, high-entropy, stored hashed.  
3. Optional single-use redemption.  
4. Guest must capture identity fields required by policy before seal.  
5. Magic links never grant navigation into the full OS.  
6. Revoke on package void/complete/reassign.

### 9.5 QR Signing

1. QR resolves to `QrSignSession` → package/slot.  
2. Session inherits package expiry and policy assurance.  
3. QR should pin subject version for evidence integrity (align with Documents QR guidance).  
4. Rate-limit resolve attempts; unknown QR returns not found without leaking ids.

### 9.6 Identity Verification

| Assurance | Typical checks |
| --- | --- |
| Low | Guest name/company capture |
| Standard | Authenticated User session + person link |
| High | Recent auth / step-up (Core) + explicit consent screen |
| StepUp | Core reauth/OTP before seal |

1. Assurance level recorded on `IdentityAssuranceRecord` per seal.  
2. Signatures does not store passwords; challenges delegated to Core/IdP where applicable.  
3. Shared device scenarios require visible identity chip and re-verify per policy.

### 9.7 Certificate Generation

1. On package completion, generate EvidenceCertificate (async workflow).  
2. Certificate includes: subject refs, version/hash, signer identities, assurance levels, timestamps, signature artifact hashes, policy version, package id, tenant/project context.  
3. Artifact stored via Core Files; immutable.  
4. Regeneration allowed only as identical byte/hash reproduce or explicit “reissue copy” with same manifest—never silent content change.

### 9.8 Reminders

1. Reminder cadence from policy (e.g., T+24h, T+72h, final).  
2. Stop on seal/void/expire/decline.  
3. Implemented via Temporal—not client push only.

---

## 10. Workflow Integration

| Workflow | Purpose |
| --- | --- |
| `SignaturePackageWorkflow` | Track slot completion; complete/expire package |
| `SequentialSigningWorkflow` | Enforce order; unlock next signer |
| `MagicLinkLifecycleWorkflow` | Expiry/revoke |
| `QrSignSessionWorkflow` | Session TTL |
| `SignatureReminderWorkflow` | Reminder schedule → Notifications |
| `EvidenceCertificateWorkflow` | Assemble certificate artifact |
| `VersionValidationWatch` | Optional watch if subject version superseded mid-flight → notify/void per policy |

### 10.1 Typical Sequence (Document Ack)

```text
Documents creates Assignment(purpose=Sign)
  → Signatures.CreateSignaturePackage(subject=DocumentVersion, pin hash)
  → issue MagicLink and/or notify User slots
  → start SignaturePackageWorkflow + ReminderWorkflow
  → signer opens link / app
  → IDV per policy
  → validate document version
  → capture signature → SignatureCaptured
  → on all slots sealed: PackageCompleted
  → EvidenceCertificateGenerated
  → Documents marks acknowledged
  → Core.AuditApi throughout
```

### 10.2 Typical Sequence (Toolbox Multi-Signer)

```text
Safety submits toolbox
  → CreateSignaturePackage(crew slots, parallel)
  → workers seal on mobile (online/offline per policy)
  → PartiallySigned progress events
  → Completed → SafetyActivitySealed
```

---

## 11. Permissions

| Code | Intent |
| --- | --- |
| `signatures.policy.manage` | Configure signing policies |
| `signatures.package.create` | Create packages (often via subject modules’ orchestration) |
| `signatures.package.read` | View package status/evidence metadata |
| `signatures.package.void` | Void packages |
| `signatures.slot.assign` | Assign/reassign signers |
| `signatures.seal.self` | Seal own slot (authenticated) |
| `signatures.guest.issue` | Issue guest/magic links |
| `signatures.qr.issue` | Issue QR sessions |
| `signatures.certificate.generate` | Force regenerate/reissue copy |
| `signatures.audit.read` | Signature audit/history |
| `signatures.reports.read` | Reporting |

Guest seal uses link capability token—not a standing RBAC grant.

Subject modules also need permission to request packages as part of their commands (enforced in subject + Signatures create authz).

---

## 12. Security

### 12.1 Threats & Controls

| Threat | Control |
| --- | --- |
| Link leakage | Short TTL, hash-at-rest, optional single-use, revoke on complete |
| QR replay / farming | Session TTL, rate limits, pin to package, bot protections at edge |
| Signer spoofing | Assurance levels, Core step-up, guest field validation |
| Version bait-and-switch | Pin version id + content hash; validate at seal |
| Evidence tampering | Immutable artifacts, checksums, Core Audit, R2 object controls |
| Privilege abuse (void) | Elevated permission + reason + audit |
| Offline forgery | Server authority on final seal accept; policy may forbid offline seal for high assurance |
| PII leakage in events | Minimal identity in bus payloads; detail via authorized query |

### 12.2 Data Protection

- Signature images/strokes encrypted at rest via platform storage controls.  
- Magic link secrets never logged.  
- Tenant isolation on all package queries.  
- Presigned download of evidence certificates authorized per package ACL.  
- Retention aligned with Documents/Safety evidence retention + legal hold.

### 12.3 Separation of Duties

- Policy admins ≠ necessarily package voiders (configurable).  
- Certificate reissue audited distinctly from original generation.

---

## 13. Legal Considerations

> Not legal advice. Product must support configurable policies reviewed by counsel per region (CA/US/AU/NZ).

### 13.1 Design Implications

| Topic | Product approach |
| --- | --- |
| **Intent to sign** | Clear consent copy before seal; process-type specific |
| **Identity** | Assurance levels matched to risk (SWP vs toolbox) |
| **Integrity** | Hash-bound subject content + immutable evidence certificate |
| **Timestamping** | Trusted server time at seal; record timezone |
| **Attribution** | Bind to User/Person/Guest identity fields |
| **Retention** | Configurable retention; legal hold compatible |
| **Cross-border** | Tenant region settings; data residency roadmap via Core/platform |
| **Wet-ink equivalence** | Policy states where e-sign accepted; do not claim universal legal equivalence in UX copy |
| **Minors / capacity** | Out of default construction worker flows; block unless explicitly configured |
| **Accessibility** | Accessible signing UX (WCAG) for enforceability and inclusion |
| **Disclosure** | Certificate exports suitable for auditors/regulators |

### 13.2 Regional Config

SigningPolicy and tenant legal settings should allow:

- Required assurance by process  
- Guest allowed/disallowed  
- Mandatory typed name vs drawn signature  
- Retention minimums  
- Disclaimer text templates by region  

### 13.3 What Signatures Will Not Claim

- That a seal alone proves hazard controls were effective  
- That guest name entry equals government ID verification unless IDV integrations explicitly added later  
- That all jurisdictions treat all package types as advanced electronic signatures  

---

## 14. Audit Trail

### 14.1 Dual Layer

| Layer | Purpose |
| --- | --- |
| **Core Audit** | Security-significant actions: create, void, policy change, link issue, certificate generate |
| **Signatures evidence trail** | Slot seals, IDV snapshots, validation results—part of EvidenceCertificate |

### 14.2 Must Capture on Seal

- Who (identity + kind)  
- When  
- What subject + version/hash  
- How verified (assurance)  
- Where/device meta per policy  
- Policy version  

---

## 15. Reporting

| Report | Purpose |
| --- | --- |
| Pending signatures aging | Operational follow-up |
| Completion rates by process type | Program health |
| Guest vs authenticated mix | Risk posture |
| Version validation failures | Document control issues |
| Voided packages | Exception review |
| Expired packages | Process friction |
| Certificate issuance log | Evidence ops |
| Assurance level distribution | Policy compliance |

Events feed Analytics for portfolio views; enforcement and evidence remain in Signatures OLTP + artifacts.

---

## 16. Public Interfaces & API (Summary)

### 16.1 Interfaces

| Interface | Purpose |
| --- | --- |
| `SigningPolicyApi` | Get effective policy |
| `SignaturePackageApi` | Create/get/void/list; assign slots |
| `SealApi` | Seal slot (auth or link token) |
| `MagicLinkApi` | Issue/redeem/revoke |
| `QrSignApi` | Issue/resolve/complete session |
| `EvidenceCertificateApi` | Get/download certificate metadata |
| `SignatureQueryApi` | Status for subject modules / My Actions |

### 16.2 HTTP (Illustrative)

Base: `/api/signatures`

- `/policies`
- `/packages`, `/packages/{id}/slots`, `/packages/{id}/void`
- `/packages/{id}/seal`
- `/magic-links`, `/magic-links/redeem`
- `/qr-sessions`, `/qr/{code}`
- `/certificates/{packageId}`
- `/reports/...`

Guest routes are token-scoped and minimal (align with UX Guest Signing).

---

## 17. Offline Support

| Policy option | Behavior |
| --- | --- |
| Offline seal allowed (Standard) | Capture locally; sync idempotent seal command; server re-validates version/auth |
| Offline seal forbidden (High) | Must seal online after IDV |

Conflict: if package expired/voided server-side, offline seal rejected.

---

## 18. Data Ownership

### 18.1 Schema `signatures` Owns

- Policies, packages, slots, captured signature metadata  
- Magic links, QR sessions  
- IDV records  
- Evidence certificates metadata + manifests  
- Reminder counters / workflow correlation ids  

### 18.2 References

| Data | Owner |
| --- | --- |
| Stroke/image bytes | Core Files (refs in Signatures) |
| Certificate PDF bytes | Core Files |
| Document versions | Documents |
| Subject business records | Safety/Training/Equipment/Documents |
| User auth sessions | Core |

---

## 19. Integration With Other Modules

| Module | Interaction |
| --- | --- |
| **Documents** | Ack/sign packages; version validation; guest/QR doc flows |
| **Safety** | Activity/permit/lift seals; multi-signer toolbox |
| **Training** | Evaluation/completion attestations |
| **Equipment** | Inspection/binder sign-off |
| **COR** | Evidence provenance for signed artifacts |
| **People** | Signature history refs |
| **Core** | AuthN step-up, AuthZ, files, platform audit |
| **Notifications** | Reminders and invites |
| **Workflows** | Package/reminder/certificate durability |
| **Web/PWA** | In-app seal + guest/QR surfaces |

---

## 20. Anti-Patterns

1. Storing “signed=true” on subject without SignaturePackageId  
2. Mutating completed evidence  
3. Long-lived magic links without expiry  
4. Sealing floating “latest document” without recording version/hash  
5. Treating Documents approval routing as owned by Signatures  
6. Logging raw magic link secrets  
7. Using Redis as evidence SoR  
8. Claiming universal legal validity in product copy  

---

## 21. Success Criteria

Digital Signatures is correctly designed when:

1. Every audit-grade assent has a package with identity, time, and subject version binding.  
2. Guest/magic link/QR flows are time-boxed, minimal, and evidence-complete.  
3. Document seals cannot silently bind superseded content.  
4. Review/approval signatures are evidence—not a substitute for subject workflows.  
5. Certificates export defensible manifests for COR and external scrutiny.  
6. Reminders and expiry are workflow-durable; security controls match assurance levels.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Principal Architecture | Initial Digital Signature domain architecture |

---

*End of Digital Signature Domain Architecture*
