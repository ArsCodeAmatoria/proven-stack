# Proven — Go Worker Service Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Go Worker Service Architecture |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Principal Go Engineering / Platform |
| **Audience** | Backend, SRE, DevEx |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [System Architecture](./SYSTEM_ARCHITECTURE.md), [Repository Plan](./REPOSITORY_PLAN.md), [Notifications Domain](./NOTIFICATIONS_DOMAIN.md), [Analytics Domain](./ANALYTICS_DOMAIN.md), [Rust Backend](./RUST_BACKEND_ARCHITECTURE.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document defines the **Go worker service architecture** for Proven.

Go workers execute **I/O-heavy, idempotent, non-domain-authoritative jobs**: Temporal activities for media/PDF/OCR/report rendering, notification channel delivery, analytics ingest, imports/exports, image processing, and scheduled maintenance probes.

**Hard rule:** Go workers **do not own business rules**. Compliance decisions, eligibility, readiness, scoring, and aggregate invariants remain in Rust domain modules. Workers transform, render, deliver, and report status back through APIs/events.

**Architecture only — no implementation.**

---

## 2. Responsibilities

| Responsibility | Worker role | Not worker role |
| --- | --- | --- |
| **Temporal Workers** | Host I/O activities; heartbeat; retry | Domain command decisions (prefer Rust activities for those) |
| **PDF Generation** | Render PDF from approved templates + data snapshots | Decide what content is legally required |
| **OCR** | Extract text from images/PDFs; return structured candidates | Accept OCR as authoritative completion without module validation |
| **Image Processing** | Resize, transcode, thumbnail, AV-scan hooks | Store bypassing Core FileApi |
| **Email** | Provider send for Notification delivery jobs | Choose recipients/priority (Notifications module) |
| **WhatsApp** | WhatsApp Business provider send | Consent/opt-in policy (Notifications/Core) |
| **Report Generation** | Build export artifacts (CSV/XLSX/PDF) from query snapshots | Authorize report scope (Analytics/API) |
| **Imports** | Parse files; batch call module import APIs | Invent domain rows via raw SQL |
| **Exports** | Stream/write artifacts to R2; callback status | Bypass AuthZ filters |
| **Notifications** | Channel delivery attempts | Template/rule/preference SoR |
| **Scheduler** | Cron-like triggers **or** Temporal schedules that enqueue jobs | Replace Temporal for business SLAs |

---

## 3. Service Topology

### 3.1 Deployable Binaries (Recommended Split)

Scale independently on Fly.io:

| Binary | Purpose |
| --- | --- |
| `notify-worker` | Email, Push, Teams, WhatsApp (SMS future) delivery |
| `media-worker` | Image processing, OCR, AV scan hooks, PDF render |
| `analytics-worker` | NATS → ClickHouse ingest |
| `report-worker` | Analytics/COR/module export & report artifacts |
| `import-worker` | Bulk import parsers + API fan-in |
| `temporal-io-worker` | Temporal worker process registering I/O activities |
| `scheduler-worker` *(optional)* | Thin schedule fan-out if not using Temporal Schedules exclusively |

Early stage may combine into fewer binaries; keep **packages separated** so splits remain cheap.

### 3.2 Context Map

```text
NATS (jobs/events) ──► Go workers ──► Providers (email, WhatsApp, …)
Temporal ──► Go I/O activities ──► R2 / PDF / OCR / HTTP
                 │
                 ├──► Proven API (Rust) public endpoints (status callbacks, import commands)
                 ├──► ClickHouse (analytics facts)
                 └──► Object storage (R2)
```

Workers **never** open module Postgres schemas for business writes.

---

## 4. Folder Structure

Aligned with [Repository Plan](./REPOSITORY_PLAN.md):

```text
go/
├── go.mod
├── go.sum
├── cmd/
│   ├── notify-worker/
│   ├── media-worker/
│   ├── analytics-worker/
│   ├── report-worker/
│   ├── import-worker/
│   ├── temporal-io-worker/
│   └── scheduler-worker/          # optional
├── internal/
│   ├── config/                    # typed env config
│   ├── app/                       # DI / wiring per binary
│   ├── platform/
│   │   ├── logging/
│   │   ├── metrics/
│   │   ├── tracing/
│   │   ├── health/
│   │   ├── natsx/                 # consumers, ack, DLQ helpers
│   │   ├── temporalx/             # worker bootstrap, activity register
│   │   ├── httpclient/            # Proven API client (mTLS/API key)
│   │   ├── r2/                    # object storage client
│   │   └── retry/                 # shared backoff policies
│   ├── notify/
│   │   ├── email/
│   │   ├── push/
│   │   ├── teams/
│   │   ├── whatsapp/
│   │   └── sms/                   # future stub
│   ├── media/
│   │   ├── image/
│   │   ├── pdf/
│   │   ├── ocr/
│   │   └── avscan/
│   ├── analytics/
│   │   ├── ingest/
│   │   └── clickhouse/
│   ├── report/
│   │   ├── csv/
│   │   ├── xlsx/
│   │   └── pdfreport/
│   ├── impex/                     # import + export pipelines
│   │   ├── importers/
│   │   └── exporters/
│   └── scheduler/
│       └── jobs/
├── pkg/                           # only if truly reusable across repos (prefer internal/)
└── README.md
```

### 4.1 Layout Rules

- `cmd/*` — `main` only: config, DI, run, graceful shutdown  
- `internal/*` — all business-adjacent worker logic (still **no domain authority**)  
- Provider SDKs isolated behind interfaces  
- No import of Rust; contract via HTTP/NATS/Temporal payloads  

---

## 5. Dependency Injection

### 5.1 Approach

**Manual constructor injection** (idiomatic Go)—no heavy framework.

At process start:

1. Load/validate `Config`  
2. Construct platform clients (NATS, Temporal SDK, R2, HTTP API, ClickHouse, telemetry)  
3. Construct adapter implementations (EmailSender, WhatsAppSender, PdfRenderer, OcrEngine)  
4. Construct application services (DeliveryService, IngestService, …)  
5. Register handlers/activities/consumers  
6. Run until signal  

### 5.2 Interfaces (Ports)

Examples of seams:

| Port | Implementations |
| --- | --- |
| `EmailSender` | SES/SendGrid/… |
| `WhatsAppSender` | Meta Cloud API / BSP |
| `PushSender` | Web push / FCM bridge |
| `TeamsSender` | Incoming webhook / Graph |
| `ObjectStore` | R2/S3 API |
| `ProvenAPI` | HTTP client to Rust |
| `FactWriter` | ClickHouse insert |
| `PdfRenderer` | Chromium/headless or lib-based |
| `OcrEngine` | Cloud OCR / on-box engine |
| `ImageProcessor` | vips/imaging pipeline |
| `Clock` / `IDGen` | test fakes |

### 5.3 Config

Typed config from environment:

- Worker concurrency, queue names, subject filters  
- Provider endpoints & credential refs  
- Proven API base URL + auth  
- Temporal host/namespace/task queues  
- ClickHouse DSN  
- Retry/DLQ parameters  
- Feature flags for channel enablement  

Fail fast on invalid boot config.

---

## 6. Work Intake Models

### 6.1 NATS Job Consumers

Used for notification delivery jobs, analytics facts, media post-process events.

```text
Message received
  → deserialize envelope
  → idempotency check (job id)
  → process
  → ack / nak / term
```

Queue groups per worker type for horizontal scale.

### 6.2 Temporal Activities

Used when durability, heartbeats, and workflow-correlated I/O matter:

- Generate evidence PDF / COR package assembly chunks  
- OCR a file and return text  
- Produce analytics export file  
- Send digests as activity (optional; may be NATS)  

**Activity contracts** live in `contracts/temporal/`; Go and Rust share payload schemas.

Domain-mutating activities remain **Rust-preferred**. Go activities return artifacts/results; workflows then call Rust commands.

### 6.3 Scheduler

Prefer **Temporal Schedules** for:

- Digest triggers  
- Health probe fan-out  
- Periodic rebuild nudges  
- Integration poll kicks  

Optional `scheduler-worker` only emits “tick” jobs to NATS if Temporal Schedules unavailable—still no business rules.

---

## 7. Capability Designs

### 7.1 Notifications Delivery

```text
Notifications module creates DeliveryJob (Postgres)
  → publishes/queues job on NATS
  → notify-worker
      → render provider payload from job DTO (already decided)
      → send via provider
      → callback DeliveryAttemptSucceeded/Failed to Notifications API
```

Rules:

- Respect error classes (transient/permanent/rate-limit)  
- Provider template mapping for WhatsApp  
- No recipient invention  

### 7.2 PDF Generation

Inputs: template id + **immutable data snapshot** + tenant branding refs.  
Output: bytes → R2 via ObjectStore → return `file_object` completion via Proven API / activity result.

Used by: evidence certificates (assist), COR reports, admin exports, training certificates printables.

### 7.3 OCR

Inputs: `file_object_id` / R2 key.  
Output: text + confidence + optional field candidates.  
Downstream: Rust import/verification commands decide acceptance.

### 7.4 Image Processing

Pipeline stages: validate MIME → AV scan hook → resize/thumb → write derivatives → callback Core/Documents/Safety attachment status.

Quarantine path: on AV fail, call File quarantine API—do not publish.

### 7.5 Email / WhatsApp

Adapters only. Include provider message ids for idempotency. Honor quiet hours only if job DTO says already filtered (Notifications owns policy).

### 7.6 Report Generation

Triggered by Analytics `ExportJob` or COR package workflow:

1. Receive authorized query spec / pre-signed data pages / API pull with job token  
2. Materialize artifact  
3. Upload R2  
4. Complete export job via API  

Workers must not run unconstrained SQL against OLTP.

### 7.7 Imports

1. Download source file from R2  
2. Parse (CSV/XLSX) in batches  
3. Call Proven import endpoints per batch with idempotency keys  
4. Aggregate errors → result report artifact  

Validation authority = API/domain.

### 7.8 Exports

Symmetric to reports; may stream large CSV. Heartbeat for Temporal; chunked upload for size limits.

### 7.9 Analytics Ingest

```text
Domain event (NATS)
  → normalize to fact envelope
  → batch insert ClickHouse
  → update checkpoint (API or worker-owned checkpoint store via API)
```

Poison events → DLQ; do not invent metrics.

---

## 8. Retry Strategy

### 8.1 Classification

| Class | Action |
| --- | --- |
| **Transient** | Exponential backoff + jitter; retry |
| **Rate limited** | Honor `Retry-After`; isolate per tenant/connector |
| **Permanent** | No retry; fail callback; DLQ |
| **Payload invalid** | Term; alert; DLQ |
| **Dependency down** | Retry with circuit breaker; degrade non-critical |

### 8.2 Policies by Path

| Path | Retry owner |
| --- | --- |
| NATS consumers | Worker retry + nak; max attempts then DLQ |
| Temporal activities | Temporal retry policy (primary) + activity-level idempotency |
| Provider sends | Bounded attempts then permanent fail to Notifications |

### 8.3 Idempotency

- Every job has stable `job_id` / `activity_id` + `attempt`  
- Side effects keyed by idempotency key to providers when supported  
- Callbacks to Proven API are idempotent  
- At-least-once is expected  

### 8.4 Dead Letter Queue

- NATS DLQ subjects per worker family  
- Temporal failure → workflow see activity error; ops dashboards  
- Replay tools are operational procedures (Admin/ops), audited  

### 8.5 Illustrative Backoff

Attempts at 0s, 30s, 2m, 10m, 30m (configurable per worker); cap by count/time. Critical notification failures alert aggressively.

---

## 9. Logging

### 9.1 Standards

- Structured JSON logs  
- Fields: `service`, `worker`, `job_id`, `tenant_id` (when present), `correlation_id`, `attempt`, `provider`, `error_class`  
- **No** message bodies with PII/PHI, magic links, raw WhatsApp content in info logs  
- Debug sampling only in non-prod  

### 9.2 Correlation

Propagate W3C trace context / correlation ids from NATS headers and Temporal context into logs and outbound API calls.

---

## 10. Monitoring

### 10.1 Metrics (Examples)

| Metric | Purpose |
| --- | --- |
| `worker_jobs_processed` | Throughput |
| `worker_job_duration_seconds` | Latency |
| `worker_retries` | Retry pressure |
| `worker_dlq_depth` | Poison backlog |
| `provider_send_errors` | Channel health |
| `temporal_activity_failures` | Activity health |
| `clickhouse_insert_failures` | Analytics pipeline |
| `r2_upload_failures` | Media/export health |
| `consumer_lag` | NATS lag |

### 10.2 Health Endpoints

Each binary exposes:

- `/healthz` — process up  
- `/readyz` — dependencies reachable (NATS/Temporal/R2 as required)  

Orchestrator uses these for rolling deploys.

### 10.3 Alerts

- DLQ growth  
- Critical notify permanent failure spike  
- Ingest lag beyond freshness SLO  
- Temporal task queue backlog  
- AV quarantine surge  

### 10.4 Tracing

Optional OpenTelemetry spans around job handle → provider call → callback.

---

## 11. Testing

### 11.1 Layers

| Layer | What |
| --- | --- |
| Unit | Pure transforms, retry classifiers, template mappers with fakes |
| Adapter contract | Provider clients against mocks/sandboxes |
| Consumer integration | NATS testcontainer + fake API |
| Temporal activity | Test environment / mock activity env |
| Golden file | PDF/CSV structure smoke (non-flaky fixtures) |

### 11.2 Rules

- Table-driven tests  
- No live prod providers in CI  
- Idempotency tests (double delivery)  
- Timeout/cancelation tests for long PDF/OCR  
- Race tests for concurrent workers on same job id  

---

## 12. Deployment

### 12.1 Runtime

- **Fly.io** machines per worker binary  
- Docker multi-stage builds (`Dockerfile.workers` family or one Dockerfile with targets)  
- Secrets via Fly secrets / platform secret store  
- Horizontal scale by process count / concurrency flags  

### 12.2 Task Queues / Subjects

| Worker | Temporal task queue (ex.) | NATS queue group (ex.) |
| --- | --- | --- |
| temporal-io-worker | `proven-io` | — |
| notify-worker | optional | `notify-delivery` |
| media-worker | `proven-io` activities | `media-jobs` |
| analytics-worker | — | `analytics-ingest` |
| report-worker | `proven-io` | `report-jobs` |
| import-worker | `proven-io` | `import-jobs` |

### 12.3 Release

- GitHub Actions build/test/scan → publish images → Fly deploy  
- Rolling deploy with health checks  
- Compatible activity contract versions during rollout (additive payloads)  
- Feature flags for new channels (WhatsApp)  

### 12.4 Resource Profiles

| Worker | Notes |
| --- | --- |
| media / pdf | Higher CPU/RAM; concurrency low |
| notify | High concurrency; low CPU |
| analytics | Batch memory; CH write throughput |
| report/import | Mixed; timeout headroom |

### 12.5 Security

- Least-privilege API keys for Proven callbacks  
- Egress allowlists where possible  
- No module DB credentials on workers  
- Quarantine & AV before derivatives go public  
- Tenant id always validated on job payload against auth context of API callbacks  

---

## 13. Concurrency & Graceful Shutdown

- Bounded worker pools per binary  
- Context cancel on SIGTERM; finish in-flight within drain timeout  
- NATS unacked messages redeliver  
- Temporal workers shutdown gracefully per SDK  

---

## 14. Error Reporting Back to Platform

Workers report outcomes via:

1. **Proven API callbacks** (delivery attempt, export complete, file processed)  
2. **Temporal activity results/errors**  
3. **NATS result subjects** (optional)  
4. **Metrics/logs** for ops  

Never write directly to `notifications` / `documents` / `safety` tables.

---

## 15. Anti-Patterns

1. Encoding CA overdue rules in notify-worker  
2. Opening Postgres module schemas from Go  
3. Infinite retries without DLQ  
4. Treating OCR text as final Training completion  
5. Logging WhatsApp/email full bodies  
6. Running unconstrained ClickHouse-destroying queries from report workers without AuthZ job tokens  
7. Single god-binary that cannot scale media separately from notify  

---

## 16. Success Criteria

Go workers are correctly designed when:

1. Channels deliver reliably with classified retries and DLQ.  
2. PDF/OCR/image/report jobs are durable via Temporal when needed and leave artifacts in R2.  
3. Analytics ingest keeps ClickHouse fresh without touching OLTP writes.  
4. Imports/exports call versioned Proven APIs with idempotency.  
5. Schedules do not replace Temporal business workflows.  
6. Each worker family scales and deploys independently.  
7. No compliance invariant lives in Go.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Principal Go Engineering | Complete Go worker service architecture (no implementation) |

---

*End of Go Worker Service Architecture*
