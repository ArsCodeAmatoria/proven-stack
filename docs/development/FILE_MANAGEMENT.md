# File Management

Canonical design: [ADR-0010](../adr/0010-file-management.md) and
[R2_STORAGE_ARCHITECTURE.md](../architecture/R2_STORAGE_ARCHITECTURE.md).

Core `FileApi` is the platform's **only** file metadata SoR. Bytes live in Cloudflare R2
(or a local placeholder until R2 is wired). Modules store `FileObjectId` references only.

## Capabilities

| Capability | Status |
| --- | --- |
| Photos / PDFs / Videos / Certificates / Drawings / Attachments | **Implemented** (`FileObjectClass`) |
| Versioning | **Implemented** (child `FileObject` + `parent_file_id` / `content_version`) |
| Metadata | **Implemented** (JSONB bag + update API) |
| Virus scan hook | **Implemented** (port + passthrough / enqueue stubs) |
| Audit trail | **Implemented** (intent, complete, scan, links, delete, metadata) |
| Temporary uploads | **Implemented** (`is_temporary` + `expires_at` + list candidates) |
| Private links | **Implemented** (short-lived presigned GET) |
| Public links | **Implemented** (API share tokens → private presign; **no public R2 ACLs**) |
| Cloudflare R2 SigV4 signer | **Pending** — placeholder URLs today |
| Go media-worker AV / Temporal media workflow | **Pending** |
| Multipart large uploads | **Pending** |

## HTTP (`/api/v1/core/files/*`)

| Method | Path | Purpose |
| --- | --- | --- |
| `POST` | `/upload-intents` | Create intent + PUT URL |
| `GET` | `/{id}` | Metadata |
| `DELETE` | `/{id}` | Soft delete |
| `POST` | `/{id}/complete` | Finish upload → scan hook |
| `GET` | `/{id}/versions` | Version lineage |
| `PUT` | `/{id}/metadata` | Replace metadata bag |
| `POST` | `/{id}/download-link` | Private presigned GET |
| `POST` | `/{id}/share-links` | Create public share token |
| `GET` | `/shares/{token}` | Resolve public share → short GET |
| `POST` | `/{id}/scan-result` | Worker callback (`clean`/`infected`/…) |

## R2 configuration (pending signer)

```bash
R2_ACCOUNT_ID=
R2_BUCKET=
R2_ACCESS_KEY_ID=
R2_SECRET_ACCESS_KEY=
# optional overrides
R2_ENDPOINT=
R2_PUBLIC_BASE_URL=
```

When unset, Core uses `PlaceholderObjectStorage` (`placeholder: true` on URLs). When set but the
signer is not wired, operators must keep using the placeholder or fail closed via
`PendingR2ObjectStorage` — see `infrastructure/object_storage.rs`.

## Virus scan

| Hook | Behavior |
| --- | --- |
| `PassthroughVirusScanHook` | Marks Clean immediately (default in-memory / tests) |
| `EnqueuePendingVirusScanHook` | Leaves `Processing` / `scan_status=pending` for media-worker |

Wire the enqueue hook + Temporal `FileMediaProcessingWorkflow` before production traffic.

## Hard rules

1. No public buckets for tenant data.
2. Server-generated keys only.
3. Available downloads only after clean scan (passthrough counts as clean in dev).
4. Never put secrets or PII in object keys.
