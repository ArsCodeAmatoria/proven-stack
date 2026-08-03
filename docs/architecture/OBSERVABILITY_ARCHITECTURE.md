# Proven — Observability & SRE Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Observability / SRE Architecture |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | SRE Architecture |
| **Audience** | SRE, Backend, Frontend, Security, On-call |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Deployment Architecture](./DEPLOYMENT_ARCHITECTURE.md), [System Architecture](./SYSTEM_ARCHITECTURE.md), [Testing Strategy](./TESTING_STRATEGY.md), [Go Worker Catalog](./GO_WORKER_CATALOG.md), [Rust Backend](./RUST_BACKEND_ARCHITECTURE.md), [Security Architecture](./SECURITY_ARCHITECTURE.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document designs Proven’s **observability stack**: metrics, logging, tracing, alerts, **Grafana**, **Prometheus**, **Loki**, **OpenTelemetry**, health checks, dashboards, and incident response.

**Hard rules**

1. **Golden signals first** — latency, traffic, errors, saturation per critical service.  
2. **Correlate with `correlation_id` / trace context** across web → API → workers → Temporal.  
3. **No secrets or raw PII** in logs/metrics labels (cardinality + privacy).  
4. **SLOs drive alerts** — not raw CPU noise.  
5. Documentation only — no implementation.

---

## 2. Observability Goals

| Goal | Meaning |
| --- | --- |
| **Detect** | Know when users are hurt within minutes |
| **Diagnose** | Jump from alert → dashboard → trace → logs |
| **Recover** | Runbooks + clear owners |
| **Learn** | Post-incident improvements to SLOs/tests |

---

## 3. Logical Architecture

```text
Apps (Next.js*, API, Go workers, Temporal activities)
        │  OTel SDK / instrumentation
        ▼
┌───────────────────┐
│ OpenTelemetry     │  traces + metrics (+ logs bridge)
│ Collector         │
└─────────┬─────────┘
          │
    ┌─────┴──────┬────────────┐
    ▼            ▼            ▼
Prometheus    Tempo/        Loki
(metrics)     OTLP traces   (logs)
    │            │            │
    └────────────┼────────────┘
                 ▼
              Grafana
         dashboards · alerts · explore
```

\*Browser RUM is sampled and scrubbed; primary SLIs are API and worker side.

| Component | Role |
| --- | --- |
| **OpenTelemetry** | Unified instrumentation & context propagation |
| **Prometheus** | Metrics TSDB / scrape or remote-write from Collector |
| **Loki** | Log aggregation |
| **Grafana** | Dashboards, alert rules UI, Explore |
| **Trace backend** | Tempo/Jaeger/etc. via OTLP (Grafana-linked) |

Managed cloud equivalents acceptable if they speak OTLP and Grafana; names above are the **reference architecture**.

---

## 4. OpenTelemetry

### 4.1 Propagation

| Hop | Context |
| --- | --- |
| Browser → API | W3C `traceparent` where enabled |
| API → NATS | Trace headers in message metadata |
| API → Temporal | Context in workflow/activity headers |
| API → HTTP (R2/providers) | Outgoing W3C |
| Workers → API callbacks | Propagate correlation + trace |

Always set application **`correlation_id`** (UUID) even when trace sampling drops spans.

### 4.2 Instrumentation scope

| Service | Traces | Metrics | Logs |
| --- | --- | --- | --- |
| Rust API | HTTP, DB, NATS publish, Temporal start | RED + business | Structured slog/tracing |
| Go workers | Activity, HTTP, provider calls | Job RED, queue | Structured |
| Next.js | Server routes optional; client sample | Web Vitals export | Edge/platform logs |
| Collector | Receive/process/export | Collector health | Collector logs |

### 4.3 Sampling

- Head/tail sampling: higher keep rate for errors and slow traces.  
- Staging higher sample rate than prod.  
- Never sample based on tenant id alone in a way that leaks identity in metrics.

---

## 5. Metrics (Prometheus)

### 5.1 Golden signals (per service)

| Signal | Examples |
| --- | --- |
| **Latency** | `http_server_request_duration_seconds` histogram |
| **Traffic** | Request/job rate |
| **Errors** | 5xx rate, callback failures, activity failures |
| **Saturation** | CPU, memory, DB pool in-use, queue depth, Temporal backlog |

### 5.2 Proven domain-aware metrics (low cardinality)

| Metric intent | Labels (careful) |
| --- | --- |
| API requests | `service`, `route_group`, `method`, `status_class` — **not** raw path with ids |
| Auth | `auth.login_failures`, `authz_deny_total` by `reason_class` |
| Outbox/NATS lag | `outbox_publish_lag_seconds` |
| Worker jobs | `worker`, `job_type`, `result` |
| Notify | `channel`, `result` |
| R2 | `operation`, `result` |
| Sync (product) | Client telemetry aggregated carefully |
| CH ingest | `lag_seconds`, `rows_inserted` |

**Forbidden high-cardinality labels:** `tenant_id`, `user_id`, `email`, full URLs, `project_id` on hot metrics (use logs/traces instead).

### 5.3 SLIs / SLOs (initial targets)

| SLI | SLO (illustrative) |
| --- | --- |
| API availability (non-5xx) | 99.9% monthly |
| API p95 latency (read) | < 300–500ms (excl. heavy reports) |
| API p95 latency (write field) | < 1s |
| Notify delivery success | 99% within 5m (excl. provider outage) |
| Worker activity success | 99% after retries |
| Staging freshness analytics | Per warehouse SLO |

Burn-rate alerts on SLO—not single spike panics.

---

## 6. Logging (Loki)

### 6.1 Format

Structured JSON:

| Field | Required |
| --- | --- |
| `timestamp` | Yes |
| `level` | Yes |
| `service` | Yes |
| `version` / `git_sha` | Yes |
| `correlation_id` | Yes when request-scoped |
| `trace_id` | When present |
| `tenant_id` | When authorized context (access-controlled) |
| `message` | Yes |

### 6.2 Levels & volume

- Info: request completed, job succeeded.  
- Warn: retries, degraded deps.  
- Error: failures needing action.  
- Debug: staging only or flag-gated.

Drop/sample health-check access logs.

### 6.3 Redaction

Never log: passwords, tokens, magic-link secrets, TOTP, stroke payloads, full auth headers, raw medical notes.

### 6.4 Loki labels

Keep labels low-cardinality: `service`, `env`, `level`. Put `tenant_id` in JSON body, not Loki index labels at scale.

---

## 7. Tracing

| Practice | Design |
| --- | --- |
| **Span names** | `HTTP GET /api/v1/projects`, `sqlx.query`, `nats.publish`, `temporal.StartWorkflow`, `notify.send` |
| **Attributes** | `http.status_code`, `rpc.system`, `messaging.destination`, `feature` — scrub PII |
| **Errors** | Mark span error + exception summary |
| **Link** | Grafana: metrics → exemplars → traces → logs (correlation) |
| **Long workers** | Activity heartbeats visible; parent workflow id attribute |

---

## 8. Health Checks

| Endpoint | Meaning |
| --- | --- |
| **`/healthz` (liveness)** | Process up; no dependency checks |
| **`/readyz` (readiness)** | Can serve: Postgres, NATS, Redis, Temporal, R2 as required by that binary |
| **Worker ready** | Can consume (NATS/Temporal reachable) |
| **Vercel** | Platform health; app `/api/health` optional lightweight |

### 8.1 Orchestration use

- Fly rolling deploy waits on readiness.  
- Fail readiness on sustained DB outage; keep liveness up for restart semantics.  
- Health endpoints **unauthenticated** but not informative about internals (no connection strings).

Synthetic uptime checks from outside Cloudflare probe `https://api.../readyz` and web homepage.

---

## 9. Grafana

| Use | Design |
| --- | --- |
| **Dashboards** | As-code (jsonnet/Model) in repo `deploy/observability/` (future) |
| **Explore** | Logs (Loki), metrics (Prometheus), traces |
| **Alerting** | Grafana Alerting or Prometheus rules → Pager/Ops channel |
| **Access** | SSO; prod viewer vs editor roles; audit admin changes |
| **Folders** | `API`, `Workers`, `Data`, `Edge`, `SLO`, `Security` |

---

## 10. Prometheus

| Aspect | Design |
| --- | --- |
| **Ingest** | OTel Collector → Prometheus remote-write **or** scrape `/metrics` |
| **Retention** | Short local + longer remote storage (ADR) |
| **Recording rules** | Precompute SLO burn, error rates |
| **Federation** | Optional per-env Prometheus |

Expose `/metrics` only on private network or with auth—not public internet.

---

## 11. Loki

| Aspect | Design |
| --- | --- |
| **Ship** | Collector / Fluent Bit / Grafana Agent → Loki |
| **Retention** | Env-based (e.g. staging 7–14d; prod 30–90d by class) |
| **Auth** | Same Grafana RBAC; prod logs restricted |
| **Query** | By `correlation_id`, `trace_id`, `service` |

---

## 12. Dashboards (Minimum Set)

| Dashboard | Contents |
| --- | --- |
| **API Overview** | RPS, p95/p99, 5xx, pool saturation, auth fail rate |
| **Workers** | Per binary job rates, failures, DLQ, lag |
| **Temporal** | Schedule lag, workflow fail, activity timeouts |
| **Data plane** | Postgres connections/CPU, Redis, NATS, R2 errors, CH ingest lag |
| **Edge** | Cloudflare WAF/blocks, cache (marketing only) |
| **SLO / Error budget** | Burn rates for API & notify |
| **Security** | Login failures, AuthZ denies, quarantine spikes |
| **Release** | Version markers, deploy annotations |

Annotate deploys on graphs (git sha).

---

## 13. Alerts

### 13.1 Principles

- Alert on **symptoms** (SLO burn, error rate, lag)—not every CPU blip.  
- Page only when **human action** needed soon.  
- Staging alerts → Slack; prod pages on-call.  
- Every alert links **dashboard + runbook**.

### 13.2 Severity

| Severity | Response |
| --- | --- |
| **SEV1** | Customer-wide outage / data risk — page immediately |
| **SEV2** | Major degradation — page / urgent |
| **SEV3** | Partial / workaround — ticket business hours |
| **SEV4** | Info / capacity planning |

### 13.3 Example alert classes

| Alert | Condition (illustrative) |
| --- | --- |
| API error budget burn | Fast burn 2% / 1h |
| API readiness failing | Ready check down > 2m |
| Worker DLQ growth | Depth rising > N |
| Notify failure rate | > X% for 15m |
| Postgres connections | > 85% pool |
| Outbox lag | > threshold |
| Certificate/job poison | Spike in permanent failures |
| Security | Login failure storm |

Tune after baseline; avoid flappy alerts.

---

## 14. Incident Response

### 14.1 Lifecycle

```text
Detect (alert/user) → Triage SEV → Communicate
  → Mitigate (rollback/flag/scale)
  → Diagnose (Grafana: metrics → traces → logs)
  → Resolve → Monitor
  → Post-incident review (blameless)
```

### 14.2 Roles

| Role | Duty |
| --- | --- |
| **Incident commander** | Decisions, comms cadence |
| **Tech lead** | Diagnosis/mitigation |
| **Comms** | Status to stakeholders |
| **Scribe** | Timeline |

### 14.3 Comms

- Internal channel `#incident-YYYYMMDD`  
- Customer status for SEV1/2 per policy  
- Correlate deploy version in first 5 minutes  

### 14.4 Runbooks (minimum)

| Runbook | Trigger |
| --- | --- |
| API 5xx spike | Rollback Fly tag; check DB |
| Auth outage | IdP/Better Auth; session store |
| Worker DLQ | Poison inspect; pause consumer |
| R2 upload fail | Creds/CORS/AV |
| Notify provider down | Degrade channel; status |
| Migration stuck | SRE + DB owners |

Store under `docs/runbooks/` (as authored).

### 14.5 Post-incident

- Timeline, root cause, contributing factors  
- Action items with owners (tests, alerts, docs)  
- Update SLO/alert thresholds if noisy/missed  

---

## 15. Environment Separation

| Env | Observability |
| --- | --- |
| **Dev** | Optional local Collector; console logs |
| **CI** | Test logs as artifacts; no prod shipping |
| **Staging** | Full stack; noisier sampling; Slack alerts |
| **Production** | Strict RBAC; paging; longer retention |

Separate Grafana orgs/datasources per env or clear `env` labels.

---

## 16. Security of the Observability Plane

- SSO to Grafana; MFA.  
- Prod log access audited / least privilege.  
- Metric endpoints not public.  
- Scrubbers in Collector for known secret patterns.  
- Retention aligned with privacy policy.

---

## 17. Ownership

| Area | Owner |
| --- | --- |
| Collector / Prometheus / Loki / Grafana | SRE |
| App instrumentation | Service owners (API, workers, web) |
| SLO definitions | SRE + Product/Eng leads |
| On-call rotation | SRE |
| Runbooks | Service owners + SRE |

---

## 18. Success Criteria

1. On-call can go from page → dashboard → trace → logs in one workflow.  
2. SLOs exist for API and critical workers with burn-rate alerts.  
3. OTel context crosses API, NATS, Temporal, and workers.  
4. Health checks correctly drive Fly rolling deploys.  
5. Incidents follow a documented response and leave durable improvements.  
6. Cardinality and PII rules keep the platform affordable and safe.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | SRE Architecture | OTel, Prometheus, Loki, Grafana, IR |

---

*End of Observability & SRE Architecture*
