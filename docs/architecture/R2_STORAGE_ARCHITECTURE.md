# Proven — Cloudflare R2 Storage Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Cloud Object Storage Architecture (Cloudflare R2) |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Cloud Storage / Platform Architecture |
| **Audience** | Backend, SRE, Security, Frontend |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [System Architecture](./SYSTEM_ARCHITECTURE.md), [Core Domain](./CORE_DOMAIN.md), [Security Architecture](./SECURITY_ARCHITECTURE.md), [PostgreSQL](./POSTGRESQL_ARCHITECTURE.md), [Go Worker Catalog](./GO_WORKER_CATALOG.md), [Digital Signatures](./DIGITAL_SIGNATURES_ARCHITECTURE.md), [Database Migration Strategy](./DATABASE_MIGRATION_STRATEGY.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document designs **Cloudflare R2** usage for Proven: what is stored (photos, PDFs, videos, certificates, drawings, attachments, OCR results), **naming**, **folders**, **metadata**, **lifecycle**, **retention**, **security**, and **permissions**.

**Hard rules**

1. R2 holds **bytes only** — business meaning and AuthZ live in Postgres modules + Core `FileObject`.  
2. **No public buckets** for tenant data — access via short-lived presigned URLs or authenticated API redirect.  
3. **Server-generated keys only** — clients never choose object paths.  
4. Upload path: **Authorize → Intent → Presign → PUT → Complete (checksum) → AV/process → Available|Quarantine**.  
5. Secrets and customer data never in key names beyond opaque ids.

**Documentation + Core FileApi implementation** — see [ADR-0010](../adr/0010-file-management.md)
and [FILE_MANAGEMENT.md](../development/FILE_MANAGEMENT.md). Cloudflare SigV4 signer, Go
media-worker AV, Temporal media workflow, and multipart remain pending integrations.

---

## 2. Role of R2 in the Stack

| Store | Role |
| --- | --- |
| **Cloudflare R2** | Object bytes (blobs) |
| **Postgres `core.file_objects`** | Object identity, checksum, class, status, retention class, tenant, uploader |
| **Module tables** | Attach blob to FLHA, document version, signature capture, etc. |
| **ClickHouse** | Not for binaries |
| **Redis** | Not for binaries |

```text
Client/Worker
    │ presigned PUT/GET (time-boxed)
    ▼
Cloudflare R2  ◄── metadata/ACL decisions from Core FileApi
    │
    ▼
Go media workers (AV, OCR, derivatives) → callback Core
```

---

## 3. What We Store

| Content type | Examples | Object class |
| --- | --- | --- |
| **Photos** | FLHA/inspection/deficiency images; thumbs | `media/image` |
| **PDFs** | Controlled doc renders, reports, certificates, COR packages | `document/pdf` |
| **Videos** | Optional field video evidence (phase-gated) | `media/video` |
| **Certificates** | Signature evidence certificates; training printable certs | `certificate` |
| **Drawings** | Site drawings, lift plans, CAD exports (pdf/png/dwg policy) | `drawing` |
| **Attachments** | Generic file attachments on activities/cases | `attachment` |
| **OCR results** | Extracted text/JSON candidates (not authoritative until accepted) | `derived/ocr` |
| **Derivatives** | Thumbnails, transcodes | `derived/image`, `derived/video` |
| **Exports** | Analytics/audit CSV/XLSX/PDF | `export` |
| **Quarantine** | Suspected malware / type mismatch | `quarantine` (prefix or bucket) |

Binary SoR for “file exists” is Core FileObject + R2; OCR acceptance is Documents/module SoR.

---

## 4. Bucket Strategy

| Approach | Design |
| --- | --- |
| **Environments** | Separate buckets (or accounts): `proven-dev`, `proven-staging`, `proven-prod` |
| **Prod layout** | Prefer **one private prod bucket** with strict prefixes **or** split `proven-prod-data` + `proven-prod-exports` for lifecycle clarity |
| **Quarantine** | Dedicated prefix `quarantine/` or separate bucket with tighter IAM |
| **Public** | Marketing assets not in tenant data bucket (CDN elsewhere) |

No anonymous list/read on tenant buckets.

---

## 5. Naming

### 5.1 Object key pattern

```text
{env}/{tenant_id}/{class}/{yyyy}/{mm}/{file_object_id}/{filename_safe}
```

| Segment | Rule |
| --- | --- |
| `env` | `prod` \| `staging` \| … (optional if bucket implies env) |
| `tenant_id` | UUID — isolation & lifecycle |
| `class` | `images` \| `pdfs` \| `videos` \| `certs` \| `drawings` \| `attachments` \| `ocr` \| `derived` \| `exports` \| `quarantine` |
| `yyyy/mm` | Server receipt time — lifecycle/partitions |
| `file_object_id` | Core UUID — stable identity |
| `filename_safe` | Sanitized original or `original`, `thumb.webp`, `ocr.v1.json` |

### 5.2 Derivative naming

```text
.../{file_object_id}/thumb.webp
.../{file_object_id}/ocr.v1.json
.../{file_object_id}/transcode.mp4
```

Same parent `file_object_id` or child FileObjects linked via `parent_file_object_id` in Postgres—**choose one convention** (prefer child FileObjects for AuthZ clarity).

### 5.3 Forbidden in keys

- Emails, person names, raw document titles with PII  
- Secrets, tokens  
- Client-supplied path traversal (`../`)  

---

## 6. Folders (Prefixes)

Logical folder tree (prefixes, not filesystem):

```text
{tenant_id}/
  images/
  pdfs/
  videos/
  certs/
  drawings/
  attachments/
  ocr/
  derived/
  exports/
  quarantine/
```

| Prefix | Contents |
| --- | --- |
| `images/` | Photos + optionally co-located originals |
| `pdfs/` | PDF binaries |
| `videos/` | Video originals |
| `certs/` | Evidence/training certificates |
| `drawings/` | Drawing packs |
| `attachments/` | Unclassified attachments |
| `ocr/` | OCR JSON/text artifacts |
| `derived/` | Thumbs/transcodes (or nest under parent id) |
| `exports/` | Time-boxed export artifacts |
| `quarantine/` | Blocked objects pending review |

Module meaning (which FLHA) is **not** encoded as deep folders—Postgres links `file_object_id`.

---

## 7. Metadata

### 7.1 R2 object metadata (HTTP/user metadata)

| Key | Purpose |
| --- | --- |
| `tenant-id` | Defense in depth |
| `file-object-id` | Correlate to Core |
| `content-sha256` | Checksum declared/verified |
| `object-class` | Class enum |
| `sensitivity` | `standard` \| `restricted` |
| `source` | `upload` \| `worker` \| `export` |

Keep metadata small; authoritative fields remain in Postgres.

### 7.2 Core FileObject metadata (SoR)

| Field | Purpose |
| --- | --- |
| Status | `PendingUpload` \| `Available` \| `Quarantined` \| `Deleted` |
| MIME / size / checksum | Integrity |
| Retention class | Lifecycle policy binding |
| Access class | AuthZ hints |
| Uploader / created_at | Provenance |
| Parent / derivative links | Graph |

### 7.3 Content types

Server allowlists per feature (images, `application/pdf`, video types, etc.). Magic-byte verification after upload; mismatch → quarantine.

---

## 8. Lifecycle

| Stage | Behavior |
| --- | --- |
| **Intent** | Core creates FileObject `PendingUpload` + presign |
| **Upload** | Client/worker PUT to R2 |
| **Complete** | Checksum verify → enqueue media workflow |
| **Process** | AV → derivatives/OCR as needed |
| **Available** | Downloadable per AuthZ |
| **Quarantine** | Not available to normal download |
| **Soft delete** | FileObject deleted; object lifecycle abort/delete per policy |
| **Archive/expire** | Prefix lifecycle rules by class/retention |

### 8.1 Incomplete uploads

Abandon `PendingUpload` after TTL (e.g. 24h): abort multipart, delete orphan keys via sweeper job.

### 8.2 Multipart

Large videos/PDFs/exports use multipart upload; workers heartbeat on Temporal activities.

---

## 9. Retention

| Class | Typical content | Retention posture |
| --- | --- | --- |
| **Evidence** | Signature captures, sealed activity photos, certificates | Long (align legal/compliance; often 7–10+ years) |
| **Controlled documents** | Published PDF bytes | Per document retention + legal hold |
| **Operational attachments** | Working drafts | Shorter or follow parent entity |
| **OCR derived** | Regenerable | Medium; can purge if original kept |
| **Thumbs/derived** | Regenerable | Shorter; rebuild on demand |
| **Exports** | Report downloads | Short (days–weeks) then delete |
| **Quarantine** | Malware suspects | Short review window then destroy |
| **Legal hold** | Any | Suppress lifecycle delete |

R2 lifecycle rules approximate class via prefix; **legal hold** is enforced in Core (do not delete FileObject/key while held).

OLTP evidence rows and R2 bytes should share retention intent; deleting bytes without updating FileObject is forbidden.

---

## 10. Security

| Control | Design |
| --- | --- |
| **Private buckets** | Block all public ACLs |
| **Encryption** | R2 server-side encryption at rest; TLS in transit |
| **Presign** | Method, key, max size, short TTL (minutes); content-type constraints |
| **CORS** | Only Proven web origins for browser PUT |
| **No list** | Clients cannot list bucket prefixes |
| **AV** | Mandatory before Available ([Security](./SECURITY_ARCHITECTURE.md)) |
| **Quarantine isolation** | Separate prefix/IAM; admin review only |
| **Access logs** | Enable R2/access logging to SIEM where available |
| **Credentials** | Scoped API tokens in secret store; rotate; prefer temporary creds |
| **Worker egress** | Workers use service credentials; tenant validated on callback |

---

## 11. Permissions

### 11.1 Application AuthZ (authoritative)

| Action | Gate |
| --- | --- |
| Create upload intent | `core.file.upload` + module context permission |
| Download | `core.file.read` + `AuthorizeFileAccess` / module ACL |
| Delete | `core.file.delete` + policy |
| Quarantine review | Security/admin permission |

UI never grants R2 keys permanently.

### 11.2 Cloud IAM (platform)

| Principal | Access |
| --- | --- |
| API (presign issuer) | Sign limited PUT/GET |
| Go media/report workers | Read/write needed prefixes |
| Humans | No direct prod bucket access (break-glass audited) |
| CI | Staging only |

### 11.3 Presigned GET

- Short TTL  
- Optional `Content-Disposition` safe filename  
- Do not issue GET for Quarantined/Deleted  

---

## 12. Per-Content Guidelines

### 12.1 Photos

- Cap resolution client-side; store original + `thumb.webp`  
- Optional GPS in EXIF stripped or retained per tenant privacy policy  
- Bind to Safety/Equipment via module attachment rows  

### 12.2 PDFs

- Controlled docs: versioned FileObjects; publish does not overwrite key—new version id  
- Certificates/reports: immutable once Available  

### 12.3 Videos

- Feature-flagged; size/duration caps; async transcode derivative  
- Higher storage cost → stricter retention review  

### 12.4 Certificates

- Write-once; verify hash with Signatures package  
- Long retention; exportable with AuthZ  

### 12.5 Drawings

- Type allowlist; large file multipart; preview derivative when possible  

### 12.6 Attachments

- Generic class; inherit parent entity retention when possible  

### 12.7 OCR results

- Stored as JSON/text under `ocr/`; **candidates only**  
- Documents module must accept before search/index authority  
- Regenerable when OCR model changes  

---

## 13. Integration Flows

### 13.1 Browser upload

```text
AuthZ → CreateFileUploadIntent
  → Presigned PUT
  → CompleteFileUpload
  → FileMediaProcessingWorkflow (AV, thumb, OCR optional)
  → Available | Quarantined
  → Module binds file_object_id
```

### 13.2 Worker-produced artifacts

```text
Temporal activity renders PDF/export
  → Upload with service credentials / presign
  → Complete via API
  → Attach to export job / certificate / COR package
```

### 13.3 Download

```text
AuthorizeFileAccess → short GET presign or streamed redirect
```

---

## 14. Observability & Operations

| Signal | Use |
| --- | --- |
| Pending upload age | Orphan sweeper |
| Quarantine count | Security |
| Storage by prefix/tenant | Cost |
| AV fail rate | Pipeline health |
| Lifecycle delete errors | Retention bugs |

Runbooks: credential rotation, ransomware/isolation, bulk legal hold, restore from versioning if enabled.

### 14.1 Versioning

Enable **object versioning** on prod evidence prefixes where cost allows—supports overwrite mistakes and forensics. App still prefers new `file_object_id` over mutate-in-place.

---

## 15. Multi-Environment & Future Expansion

| Topic | Design |
| --- | --- |
| **Local dev** | MinIO/R2-compatible stub in Compose |
| **Region** | Choose R2 jurisdiction aligned with tenant residency roadmap |
| **Replication** | Future cross-region copy for DR—not day-one |
| **Inventory** | Periodic reconcile FileObject vs R2 keys (orphan report) |

---

## 16. Success Criteria

1. Every stored byte has a Core `FileObject` and server-generated key.  
2. Photos, PDFs, videos, certificates, drawings, attachments, and OCR outputs have clear classes and prefixes.  
3. Lifecycle and retention match evidence vs export vs regenerable derived data.  
4. No public tenant data; presign + AuthZ enforce access.  
5. AV quarantine prevents malware from becoming Available.  
6. Workers and browsers never bypass FileApi for business attachments.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Cloud Storage Architecture | Cloudflare R2 design |

---

*End of Cloudflare R2 Storage Architecture*
