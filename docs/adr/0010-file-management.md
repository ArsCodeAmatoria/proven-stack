# ADR-0010: File Management in Core

| Field | Value |
| --- | --- |
| Status | Accepted |
| Date | 2026-08-03 |
| Deciders | Lead Software Engineering |

## Context

Proven stores photos, PDFs, videos, certificates, drawings, and generic attachments in
**Cloudflare R2**, with identity and AuthZ in Postgres ([R2_STORAGE_ARCHITECTURE.md](../architecture/R2_STORAGE_ARCHITECTURE.md),
[CORE_DOMAIN.md](../architecture/CORE_DOMAIN.md) §17). Core already exposes a thin
`FileApi` (upload intent → complete) with `core.file_objects` metadata.

Product needs a complete **File Management** surface: object classes, versioning, metadata,
virus-scan hooks, audit trail, temporary uploads, and public/private download links — without
a second files module (ADR-0001).

## Decision

1. Expand Core `FileApi` / `FileService` as the platform's only file metadata SoR.
2. Bytes remain in R2 (or a local placeholder store until R2 credentials are wired). Clients
   never choose object keys; Core generates keys by class prefix.
3. Object classes: `photo`, `pdf`, `video`, `certificate`, `drawing`, `attachment`.
4. Lifecycle: Intent → (presigned PUT) → Complete → **VirusScanHook** → Available | Quarantined.
5. **Private links** = short-lived presigned GET (bucket stays private).
6. **Public links** = share tokens resolved through the API (never public R2 ACLs / anonymous
   bucket read).
7. Versioning via child `FileObject` rows (`parent_file_id` + `content_version`).
8. Temporary uploads carry `is_temporary` + `expires_at`; sweeper lists purge candidates.
9. Where Cloudflare SDK, Go media-worker AV, Temporal media workflow, or multipart are not yet
   wired: ship ports + placeholder adapters and document the pending integration.

## Consequences

- Modules continue to store only `FileObjectId` references.
- Real R2 signing requires `R2_*` config; without it, Core issues placeholder URL descriptors.
- AV: default hook is pass-through Clean (dev/tests); production wires enqueue-to-worker hook.
- Arch gates unchanged — files stay inside `proven-core`.
