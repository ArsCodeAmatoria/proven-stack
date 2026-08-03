# Proven — Go Worker Catalog

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Go Worker Design Catalog |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Lead Go Engineering |
| **Audience** | Go/Backend Engineering, SRE, DevEx |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Go Workers Architecture](./GO_WORKERS_ARCHITECTURE.md), [Temporal Workflows](./TEMPORAL_WORKFLOWS.md), [Notifications](./NOTIFICATIONS_DOMAIN.md), [Data Warehouse](./DATA_WAREHOUSE_ARCHITECTURE.md), [Security](./SECURITY_ARCHITECTURE.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document designs **every Go worker** for Proven: Temporal I/O, notifications, PDF, OCR, reporting, import, export, image processing, scheduler—plus cross-cutting **monitoring, logging, testing, retry policies**, and **folder structure**.

**Hard rules**

1. Workers are **I/O-only** — no compliance/domain authority ([AGENTS.md](../../AGENTS.md)).  
2. All business writes go through **Proven API** public endpoints (service auth).  
3. Jobs are **idempotent** (job id / activity id / delivery attempt id).  
4. Secrets via platform injection—never logged.

**Documentation only — no implementation.**

---

## 2. Worker Inventory

| Binary | Primary intake | Role |
| --- | --- | --- |
| **`temporal-io-worker`** | Temporal task queue `proven-io` | Hosts PDF/OCR/image/AV/report/export activities |
| **`notify-worker`** | NATS delivery jobs (+ optional Temporal activities) | Email, Push, Teams, WhatsApp delivery |
| **`media-worker`** | NATS media jobs and/or shared with Temporal activities | Image, PDF, OCR, AV (may merge with temporal-io early) |
| **`report-worker`** | NATS / Temporal | Report & export artifact render |
| **`import-worker`** | NATS / Temporal | Bulk import parse → API commands |
| **`export-worker`** | NATS / Temporal | *(optional split)* large exports; else `report-worker` |
| **`analytics-worker`** | NATS domain events | ClickHouse fact ingest |
| **`scheduler-worker`** | Cron / ticks *(optional)* | Fan-out schedule ticks—not business SLAs |

Early stage may combine `media-worker` + report activities into `temporal-io-worker`; keep `internal/` packages separate.

---

## 3. Folder Structure

```text
go/
├── go.mod
├── go.sum
├── README.md
├── cmd/
│   ├── temporal-io-worker/
│   │   └── main.go
│   ├── notify-worker/
│   │   └── main.go
│   ├── media-worker/
│   │   └── main.go
│   ├── report-worker/
│   │   └── main.go
│   ├── import-worker/
│   │   └── main.go
│   ├── export-worker/              # optional; else report-worker
│   │   └── main.go
│   ├── analytics-worker/
│   │   └── main.go
│   └── scheduler-worker/           # optional
│       └── main.go
├── internal/
│   ├── config/                     # typed env per binary
│   ├── app/                        # wiring: NewNotifyApp, NewTemporalIOApp, …
│   ├── platform/
│   │   ├── logging/                # structured slog/zap wrapper
│   │   ├── metrics/                # Prometheus/OTel meters
│   │   ├── tracing/                # OTel traces; propagate W3C
│   │   ├── health/                 # /healthz /readyz
│   │   ├── natsx/                  # consume, ack, nak, DLQ
│   │   ├── temporalx/              # worker bootstrap, register activities
│   │   ├── httpclient/             # Proven API client (service auth)
│   │   ├── r2/                     # object storage
│   │   ├── clickhouse/             # analytics inserts
│   │   └── retry/                  # shared backoff classifiers
│   ├── notify/
│   │   ├── dispatcher/             # route attempt → channel
│   │   ├── email/
│   │   ├── push/
│   │   ├── teams/
│   │   ├── whatsapp/
│   │   ├── templates/              # render from approved template keys + data
│   │   └── sms/                    # future stub
│   ├── media/
│   │   ├── image/                  # resize, transcode, thumbnail
│   │   ├── pdf/                    # PDF generation from templates
│   │   ├── ocr/                    # text extraction candidates
│   │   └── avscan/                 # malware scan adapters
│   ├── report/
│   │   ├── csv/
│   │   ├── xlsx/
│   │   └── pdfreport/
│   ├── impex/
│   │   ├── importers/              # CSV/XLSX parsers per import type
│   │   └── exporters/              # stream writers to R2
│   ├── analytics/
│   │   ├── ingest/                 # event → fact row mapper
│   │   └── transform/              # dim upsert helpers (ids only)
│   └── scheduler/
│       └── jobs/                   # tick emitters only
├── pkg/                            # avoid; prefer internal/
└── scripts/                        # optional local run helpers
```

### 3.1 Layout rules

| Path | Rule |
| --- | --- |
| `cmd/*` | `main` only: config, construct app, run, graceful shutdown |
| `internal/platform` | Shared infra—no provider business content |
| `internal/notify|media|report|…` | Capability packages; interfaces for providers |
| **Forbidden** | SQL against module schemas; importing Rust; deciding AuthZ |

---

## 4. Temporal Worker (`temporal-io-worker`)

| Aspect | Design |
| --- | --- |
| **Purpose** | Register and execute **I/O activities** on queue `proven-io` with heartbeats for long jobs. |
| **Activities (examples)** | `RenderPdf`, `RunOcr`, `ProcessImage`, `AvScanFile`, `RenderReportArtifact`, `UploadExportChunk`, `GenerateEvidenceCertificatePdf` |
| **Not hosted here** | Domain commands (Rust activities on `proven-domain`) |
| **Inputs** | Workflow activity payloads: `tenant_id`, `file_object_id` / snapshot refs, template key, job id—**no secrets** |
| **Outputs** | Artifact location / checksum / OCR candidate DTO / scan result enum |
| **Callbacks** | HTTP to Core FileApi / Signatures / Analytics export job status |
| **Heartbeats** | Required for PDF/OCR/export > few seconds |
| **Retries** | Temporal retry policy (§12); activity idempotent on `activity_id` + input hash |
| **Config** | Temporal host, namespace, `proven-io` concurrency, R2, API base URL |
| **Ownership** | Go platform + media/report owners |

Align activity names with [TEMPORAL_WORKFLOWS.md](./TEMPORAL_WORKFLOWS.md) (FileMediaProcessing, EvidenceCertificate, ExportReport, EvidencePackage render).

---

## 5. Notifications Worker (`notify-worker`)

| Aspect | Design |
| --- | --- |
| **Purpose** | Execute **channel delivery attempts** created by Notifications module (recipients/priority/template already decided). |
| **Intake** | NATS `proven.jobs.notify.delivery` (or equivalent); optional Temporal activity `DeliverNotification` if workflow awaits |
| **Channels** | Email, Push, Teams, WhatsApp (SMS future stub) |
| **Flow** | Receive attempt → render template with provided data → provider send → POST delivery result to API |
| **Idempotency** | `delivery_attempt_id`; provider message id stored on success |
| **Respect** | Quiet hours / consent already filtered by module; worker does not re-decide policy (may no-op if API says cancelled) |
| **Retries** | Transient provider errors → retry; permanent (bounce, invalid) → fail callback, no infinite loop |
| **Config** | Provider credentials, rate limits, webhook signing secrets |
| **Ownership** | Notifications + Go |

---

## 6. PDF Worker (capability: `internal/media/pdf` + activities)

| Aspect | Design |
| --- | --- |
| **Purpose** | Render PDFs from **approved templates** + immutable data snapshots (FLHA proof sheets, evidence certificates, COR report PDFs, export PDFs). |
| **Intake** | Temporal activity or NATS `jobs.media.pdf` |
| **Inputs** | `template_key`, `template_version`, JSON snapshot ref or inline bounded payload, `output_file_intent` |
| **Process** | Fetch snapshot if needed → render → upload via presign/API → return `file_object_id` / checksum |
| **Must not** | Choose legal content; invent missing compliance fields |
| **Retries** | Transient R2/API; permanent template-missing → fail |
| **Heartbeats** | Page loops / large merges |
| **Ownership** | Media/report Go owners |

---

## 7. OCR Worker (capability: `internal/media/ocr`)

| Aspect | Design |
| --- | --- |
| **Purpose** | Extract text/candidates from images/PDFs for Documents (or other modules) to **validate before accept**. |
| **Intake** | Temporal `RunOcr` / NATS media job |
| **Outputs** | Plain text + optional structured candidates + confidence; never auto-publish documents |
| **Retries** | Provider throttling; permanent corrupt file → fail → quarantine path via API |
| **Ownership** | Media Go owners |

---

## 8. Image Processing Worker (capability: `internal/media/image` + `avscan`)

| Aspect | Design |
| --- | --- |
| **Purpose** | Thumbnails, resize, transcode, EXIF strip policy, AV scan hooks post-upload. |
| **Intake** | Part of `FileMediaProcessingWorkflow` activities / NATS |
| **Flow** | Download object → AV scan → process derivatives → upload derivatives → `MarkAvailable` or `Quarantine` via Core API |
| **Must not** | Bypass FileApi; serve quarantined as available |
| **Retries** | Scanner transient; malware hit → permanent quarantine |
| **Ownership** | Media + security review on AV adapter |

---

## 9. Reporting Worker (`report-worker`)

| Aspect | Design |
| --- | --- |
| **Purpose** | Build analytics/COR/module **report artifacts** (CSV/XLSX/PDF) from **pre-authorized query snapshots** or paginated API pulls with job token. |
| **Intake** | Temporal `ExportReportWorkflow` activities / NATS `jobs.report.render` |
| **Flow** | Authorize job still open (API) → fetch pages → render → R2 complete → `ExportJobCompleted` via API |
| **Formats** | `internal/report/csv`, `xlsx`, `pdfreport` |
| **AuthZ** | Job token / service call already scoped; worker must not widen filters |
| **Heartbeats** | Per page / per chunk |
| **Ownership** | Analytics + Go |

---

## 10. Import Worker (`import-worker`)

| Aspect | Design |
| --- | --- |
| **Purpose** | Parse bulk files (workers, assets, training rosters); emit **batched module import API** commands. |
| **Intake** | NATS / Temporal import job |
| **Flow** | Download file → parse → validate shape → batch `POST` import endpoints with idempotency keys → progress callback |
| **Must not** | Insert via raw SQL; invent tenant data; skip AuthZ job context |
| **Error handling** | Row-level error report file; job fail if critical threshold exceeded (policy from API) |
| **Ownership** | Integrations/admin + Go |

---

## 11. Export Worker (`export-worker` or `report-worker`)

| Aspect | Design |
| --- | --- |
| **Purpose** | Stream large exports to R2; symmetric to reporting with emphasis on **chunked upload** and memory bounds. |
| **Intake** | Same family as reports; split binary if CPU/memory profile differs |
| **Flow** | Query pages → write stream → multipart R2 → complete export job |
| **Retries** | Chunk re-upload idempotent; resume with cursor |
| **Ownership** | Analytics + Go |

---

## 12. Analytics Ingest Worker (`analytics-worker`)

| Aspect | Design |
| --- | --- |
| **Purpose** | Consume domain events → transform → **ClickHouse** inserts; dim upserts as designed in warehouse doc. |
| **Intake** | NATS queue group `ANLY` |
| **Must not** | Become SoR; enforce readiness; block OLTP |
| **Idempotency** | `fact_id` = `event_id` when 1:1 |
| **Retries** | CH transient; poison → error table / DLQ |
| **Checkpoints** | Via API or documented store—not ad-hoc local disk as SoR |
| **Ownership** | Analytics + Go |

---

## 13. Scheduler Worker (`scheduler-worker`, optional)

| Aspect | Design |
| --- | --- |
| **Purpose** | Emit **tick** jobs to NATS when Temporal Schedules are unavailable; health probe fan-out. |
| **Must not** | Implement CA due logic, training expiry, or other business SLAs (those are Temporal workflows / Rust). |
| **Prefer** | Temporal Schedules for digests, health polls, reindex ticks |
| **Ownership** | SRE / platform Go |

---

## 14. Cross-Cutting: Monitoring

| Signal | Where |
| --- | --- |
| **Process** | `/healthz` liveness; `/readyz` (NATS/Temporal/R2/CH as required) |
| **Metrics** | Job success/fail, latency histograms, retry counts, DLQ depth, provider error rates, CH insert lag, activity heartbeat timeouts |
| **Alerts** | DLQ growth, ready failing, provider 5xx spike, ingest lag vs SLO |
| **Dashboards** | Per binary + Temporal task queue backlog |

Metrics in `internal/platform/metrics`; labels: `worker`, `job_type`, `tenant_id` **sparingly** (cardinality).

---

## 15. Cross-Cutting: Logging

| Rule | Detail |
| --- | --- |
| **Format** | Structured JSON (slog/zap) |
| **Fields** | `correlation_id`, `tenant_id`, `job_id` / `activity_id`, `attempt`, `worker` |
| **Never log** | Provider API keys, magic-link secrets, raw signature strokes, full PII payloads, auth headers |
| **Levels** | Info for start/success; Warn transient; Error permanent/poison |
| **Tracing** | Propagate W3C trace context from NATS headers / Temporal context |

Implementation home: `internal/platform/logging` + `tracing`.

---

## 16. Cross-Cutting: Testing

| Layer | Focus |
| --- | --- |
| **Unit** | Parsers, template render with fixtures, retry classifier, fact mappers |
| **Contract** | Provider fakes; API client mocks |
| **Integration** | Testcontainers NATS; R2 stub; CH stub; Temporal test environment for activities |
| **Idempotency** | Same job id twice → one side effect |
| **Chaos** | Kill mid-heartbeat; provider 429; CH timeout |
| **Forbidden tests** | Asserting domain compliance outcomes (those are Rust) |

Layout: `internal/.../*_test.go`; optional `go/test/integration/`.

---

## 17. Retry Policies

### 17.1 Error classes

| Class | Action |
| --- | --- |
| **Transient** | Network, 5xx, 429, CH busy → exponential backoff + jitter |
| **Retry-after** | Honor provider/`Retry-After` |
| **Permanent** | 4xx validation, malware, missing template → no retry; fail callback |
| **Poison** | Unparseable payload after N attempts → DLQ + alert |

### 17.2 By intake

| Intake | Policy |
| --- | --- |
| **Temporal activities** | Primary: Temporal retry (intervals, max attempts, non-retryable error types). Activity code still idempotent. |
| **NATS consumers** | Nak + redelivery; max deliver → DLQ subject `….dlq` |
| **Provider sends** | Channel-specific caps; circuit breaker optional |
| **API callbacks** | Retry status POST; eventually alert if stuck |

### 17.3 Shared helper

`internal/platform/retry` classifies errors and supplies backoff—used by NATS loops and provider clients. Temporal policies configured at activity registration (do not double-sleep heavily inside activity unless necessary).

### 17.4 Illustrative defaults (tune per job)

| Job family | Max attempts | Backoff |
| --- | --- | --- |
| Notify delivery | 8 | exp, cap ~15m |
| PDF/OCR | 5 | exp; long start-to-close timeout |
| Image/AV | 5 | exp |
| Report/export | 5 | exp; heartbeat |
| Import batch page | 5 | exp |
| Analytics insert | 10 | short exp |
| Scheduler tick | 3 | short |

---

## 18. Per-Worker Summary Matrix

| Worker | Temporal | NATS | R2 | CH | Providers | API callbacks |
| --- | --- | --- | --- | --- | --- | --- |
| temporal-io | ✓ | optional | ✓ | — | OCR/AV | ✓ |
| notify | optional | ✓ | — | — | email/push/Teams/WA | ✓ |
| media | via temporal or ✓ | ✓ | ✓ | — | OCR/AV | ✓ |
| report | ✓ | ✓ | ✓ | read via API | — | ✓ |
| import | ✓ | ✓ | ✓ | — | — | ✓ |
| export | ✓ | ✓ | ✓ | — | — | ✓ |
| analytics | — | ✓ | — | ✓ | — | checkpoint ✓ |
| scheduler | — | ✓ out | — | — | — | optional |

---

## 19. Configuration (All Workers)

Common env (names illustrative):

| Key | Purpose |
| --- | --- |
| `PROVEN_API_BASE_URL` | Rust API |
| `PROVEN_SERVICE_CREDENTIAL` | Service auth |
| `NATS_URL` | Bus |
| `TEMPORAL_HOST` / `NAMESPACE` | Workflows |
| `R2_*` | Object storage |
| `CLICKHOUSE_DSN` | Analytics worker |
| `LOG_LEVEL` / `OTEL_*` | Telemetry |
| Provider-specific | Per notify/media binary |

Config structs in `internal/config`; validate at startup; fail closed if required deps missing.

---

## 20. Security Notes

- Service credentials scoped least privilege.  
- Tenant id on every job validated against auth context on API callbacks.  
- Egress allowlists for OCR/provider HTTP.  
- No user impersonation without audited API support.  
- AV quarantine path mandatory before Available.

---

## 21. Ownership

| Area | Owners |
| --- | --- |
| `cmd/*`, `internal/platform` | Go platform / SRE |
| `internal/notify` | Notifications + Go |
| `internal/media` | Media Go + security (AV) |
| `internal/report`, `impex` | Analytics / admin + Go |
| `internal/analytics` | Analytics + Go |
| `internal/scheduler` | SRE |

CODEOWNERS: `/go/` → `@org/proven-workers`.

---

## 22. Success Criteria

1. Every I/O capability has a documented worker/package home.  
2. Temporal I/O activities are isolated from domain decision activities.  
3. Retries are classified; DLQ/poison is visible.  
4. Logging/metrics/tracing are uniform across binaries.  
5. Tests prove idempotency without asserting domain invariants.  
6. Folder structure allows splitting binaries without rewriting packages.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Lead Go Engineering | Complete worker catalog |

---

*End of Go Worker Catalog*
