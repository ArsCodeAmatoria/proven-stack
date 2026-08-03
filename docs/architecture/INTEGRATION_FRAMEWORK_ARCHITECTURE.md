# Proven — Integration Framework Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Generic Integration Framework Architecture |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Integration / Platform Architecture |
| **Audience** | Backend, SRE, Security, Partner Engineering |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Rust Crate Catalog](./RUST_CRATE_CATALOG.md) (`proven-integrations`), [Administration Domain](./ADMINISTRATION_DOMAIN.md), [Notification Architecture](./NOTIFICATION_ARCHITECTURE.md), [Security Architecture](./SECURITY_ARCHITECTURE.md), [Go Worker Catalog](./GO_WORKER_CATALOG.md), [Temporal Workflows](./TEMPORAL_WORKFLOWS.md), [Event Catalog](./EVENT_CATALOG.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document designs Proven’s **generic integration framework**: pluggable connectors (Microsoft Outlook, Microsoft Teams, WhatsApp Business, BC Crane Safety, WorkSafeBC, and future APIs), **webhooks**, **REST**, authentication, retry, logging, scheduling, and secrets—without letting connectors own compliance domain rules.

**Hard rules**

1. Connectors **adapt**; module public APIs **decide** domain state.  
2. **No raw SQL** into module schemas from integrations.  
3. **Tenant-scoped** connectors; secrets in vault—not in Postgres plaintext.  
4. Inbound traffic is **authenticated, idempotent, and AuthZ-bounded**.  
5. Framework is **provider-agnostic**; specific systems are connector packs.

**Documentation only — no implementation.**

---

## 2. Framework Overview

```text
                    ┌─────────────────────────────────────┐
                    │     proven-integrations (Rust)      │
                    │  Registry · Lifecycle · Idempotency  │
                    │  Mapping · Health · Webhook verify   │
                    └───────────┬─────────────────────────┘
                                │
          ┌─────────────────────┼─────────────────────┐
          ▼                     ▼                     ▼
   Inbound Webhooks      Outbound REST           Schedules
   (signed HTTP)         (client + retry)     (Temporal ticks)
          │                     │                     │
          ▼                     ▼                     ▼
   Module command APIs    Providers / Agencies    Sync jobs
          │                     │
          └──────────┬──────────┘
                     ▼
            Go workers (optional I/O) / notify-worker
```

| Layer | Responsibility |
| --- | --- |
| **Framework core** | Connector registry, config schema, credential refs, sync job state, inbound receipt log, health, mapping DSL hooks |
| **Connector pack** | Provider-specific auth, payload map, webhook signature, API client |
| **Module ports** | People/Projects/Training/Safety/… command & query APIs |
| **Workers** | Heavy transforms, channel delivery (WhatsApp/Teams/email) |
| **Admin UI** | Enable connector, map fields, test connection, view health |

---

## 3. Core Concepts

| Term | Meaning |
| --- | --- |
| **ConnectorType** | Catalog entry (e.g. `ms_teams`, `whatsapp_business`, `worksafe_bc`) |
| **ConnectorInstance** | Tenant-enabled connector with config + secret refs |
| **Capability** | `webhook_inbound` \| `rest_outbound` \| `rest_inbound` \| `sync_pull` \| `sync_push` \| `notify_channel` |
| **SyncJob** | Scheduled or on-demand run with cursor/checkpoint |
| **InboundReceipt** | Idempotent record of an inbound event (`provider_event_id`) |
| **OutboundDelivery** | Attempt log for REST calls (distinct from Notifications delivery when integration-owned) |
| **FieldMapping** | External ↔ Proven field map (versioned) |
| **HealthSnapshot** | Last success/fail, latency, quota |

---

## 4. Connector Catalog (Initial)

### 4.1 Microsoft Outlook

| Aspect | Design |
| --- | --- |
| **Capabilities** | REST outbound (Graph); optional inbound webhook (mail/calendar notifications) |
| **Uses** | Send mail via Graph as alternative/additional to ESP; calendar holds for audits (future); read mail for intake (phase-gated) |
| **Auth** | OAuth2 / Entra app; tenant admin consent; store refresh token ref in vault |
| **Notes** | Prefer Notifications email provider for transactional mail; Outlook connector when customer requires Graph-send or calendar |
| **Domain writes** | None directly—Notifications or scheduling modules via APIs |

### 4.2 Microsoft Teams

| Aspect | Design |
| --- | --- |
| **Capabilities** | `notify_channel` + REST (Graph / incoming webhook) |
| **Uses** | Adaptive cards, user/channel alerts ([Notification Architecture](./NOTIFICATION_ARCHITECTURE.md)) |
| **Auth** | Bot/app registration + tenant connector config |
| **Delivery** | Notifications creates job → Go notify-worker → Teams adapter **or** Integrations REST for non-notify sync |
| **Mapping** | Proven `user_id` ↔ AAD object id |

### 4.3 WhatsApp Business

| Aspect | Design |
| --- | --- |
| **Capabilities** | `notify_channel`; inbound webhook (delivery status, user replies—phase-gated) |
| **Uses** | Opt-in field messaging; template messages only |
| **Auth** | Meta business credentials in vault; webhook verify token + signature |
| **Consent** | Required; enforced by Notifications prefs before send |
| **Inbound replies** | Map to support/ack flows only via explicit module APIs—no free-form command execution |

### 4.4 BC Crane Safety

| Aspect | Design |
| --- | --- |
| **Capabilities** | REST outbound/inbound as agency APIs allow; document upload/status pull |
| **Uses** | Crane-related certification / registry verification assists; submit packs where API exists |
| **Auth** | Agency-issued API keys/OAuth per their program |
| **Domain** | Equipment/People certification **suggestions** or verified flags via Equipment/People APIs—**human confirm** when authoritative registry conflicts |
| **Evolution** | Connector pack version tracks agency API versions |

*Exact agency endpoints are external; framework assumes REST + webhook patterns and versioned mappings.*

### 4.5 WorkSafeBC

| Aspect | Design |
| --- | --- |
| **Capabilities** | REST; form/document submission; status webhooks if offered; report retrieval |
| **Uses** | Incident/claim assist payloads, clearance/letter retrieval where legally integrated |
| **Auth** | Partner credentials per WorkSafeBC integration program |
| **Domain** | Safety incident modules provide data via ports; connector does not auto-close incidents |
| **Compliance** | Feature-flagged; jurisdiction gating; audit all submissions |

### 4.6 Future APIs

| Pattern | Support |
| --- | --- |
| **OpenAPI-described REST** | Generate client stubs; register as connector type |
| **HRIS / ERP** | Sync workers/org → People/Projects via mapping |
| **SSO-only** | Prefer Core/Better Auth OIDC—not Integrations (IdP is AuthN) |
| **Custom webhook source** | Generic inbound verifier + mapping pack |

New connectors = new pack + catalog entry + Admin UI schema—**no framework fork**.

---

## 5. Webhooks

### 5.1 Inbound

```text
Provider → Cloudflare WAF → POST /api/v1/integrations/webhooks/{connector}/{instance}
  → verify signature / timestamp
  → idempotency on provider_event_id (InboundReceipt)
  → map payload → module command(s)
  → 200 quickly; async heavy work via Temporal/NATS
```

| Control | Design |
| --- | --- |
| **Auth** | HMAC/signature, mTLS, or shared secret per provider |
| **Replay** | Timestamp skew window; reject stale |
| **Idempotency** | Unique `(instance_id, provider_event_id)` |
| **AuthZ** | Instance bound to tenant; commands as service principal with narrow grants |
| **Response** | ACK fast; process async |
| **DLQ** | Poison payloads to receipt `failed` + alert |

### 5.2 Outbound (Proven → partner)

| Pattern | Design |
| --- | --- |
| **Partner registers URL** | Tenant Admin; URL allowlist validation; SSRF deny private ranges |
| **Sign** | HMAC with vault secret; include timestamp + event id |
| **Events** | Subset of domain events curated for egress (not raw internal bus) |
| **Retry** | §8 |
| **Disable** | On repeated 4xx/401 or admin off |

---

## 6. REST

### 6.1 Outbound client

| Aspect | Design |
| --- | --- |
| **HTTP** | Shared client in integrations (timeouts, tracing) |
| **Pagination** | Connector-specific cursor/page adapters |
| **Rate limits** | Token bucket per instance; honor `Retry-After` |
| **Pagination checkpoints** | Stored on SyncJob |
| **DTO mapping** | Versioned mappers; unknown fields ignored |

### 6.2 Inbound REST (partner calls Proven)

Prefer **Proven public API** with API keys over bespoke per-agency write APIs. Agency-specific façades allowed as thin translators onto module commands.

---

## 7. Authentication (Connector)

| Method | Used by (examples) |
| --- | --- |
| **OAuth2 auth code / client credentials** | Microsoft Graph (Outlook/Teams) |
| **API key / bearer** | Agency APIs, partner webhooks |
| **HMAC webhook secrets** | WhatsApp, outbound partner hooks |
| **mTLS** | Future high-assurance agencies |

### 7.1 Token lifecycle

- Store **refresh/access refs** in secrets manager; rotate on schedule.  
- Refresh before expiry in sync jobs; on `invalid_grant` → mark connector `needs_reauth` + notify admin.  
- Never log tokens.

Service principal for inbound command execution: role `integration` with least-privilege grants per connector type.

---

## 8. Retry

| Class | Policy |
| --- | --- |
| **Transient** (5xx, timeout, 429) | Exp backoff + jitter; honor Retry-After |
| **Auth failure** | Refresh once; then `needs_reauth`; no tight loop |
| **Permanent 4xx** | Fail job; DLQ; alert |
| **Webhook outbound** | Max N attempts then dead-letter + admin notify |
| **Idempotent keys** | Required on POST side effects |

Temporal for long syncs; NATS for notify deliveries. Align with Go worker retry classifier.

---

## 9. Logging

| Rule | Detail |
| --- | --- |
| **Structured** | `tenant_id`, `connector_type`, `instance_id`, `sync_job_id`, `provider_event_id`, `correlation_id` |
| **Never log** | Secrets, tokens, raw PA payloads with SIN/PII dumps, webhook secrets |
| **Redaction** | Body samples hashed or field-allowlisted |
| **Audit** | Connector enable/disable, mapping publish, successful regulated submissions (WorkSafeBC/BC Crane) via Core Audit |
| **Metrics** | Success rate, latency, retry count, reauth required |

---

## 10. Scheduling

| Mechanism | Use |
| --- | --- |
| **Temporal Schedule / SyncJob workflow** | Pull sync (certs, clearance status), token refresh probe, health poll |
| **Event-driven** | Push on domain events → outbound webhook |
| **Notifications digest** | Not Integrations (Notifications owns) |
| **Admin “Sync now”** | On-demand SyncJob |

Scheduler worker ticks only if Temporal Schedules unavailable—no business SLA logic in cron.

Health: `IntegrationHealthPollWorkflow` pattern ([Temporal](./TEMPORAL_WORKFLOWS.md)).

---

## 11. Secrets

| Secret | Storage |
| --- | --- |
| Client secrets, API keys, webhook HMAC, refresh tokens | Platform secret store / KMS ref |
| Postgres | **Reference id + metadata only** (expiry, scopes)—not plaintext |
| Admin UI | Write-only set/rotate; never read back full secret |
| Rotation | Dual-key window; Temporal expiry workflow alerts |
| CI | Staging secrets separate |

Align with [Security Architecture](./SECURITY_ARCHITECTURE.md).

---

## 12. Mapping & Transforms

| Aspect | Design |
| --- | --- |
| **Versioned maps** | `mapping_version` on instance |
| **Direction** | Inbound / outbound / bidirectional |
| **Validation** | Schema validate before module API call |
| **Conflicts** | Prefer Proven SoR; quarantine row for human resolve on registry mismatch |
| **AI** | Optional mapping suggest ([AI Systems](./AI_SYSTEMS_ARCHITECTURE.md)) with human accept—never silent |

---

## 13. Admin & Lifecycle

| Operation | Effect |
| --- | --- |
| Register/enable instance | Config + secrets + capabilities |
| Test connection | Health probe |
| Pause/disable | Stop schedules & outbound |
| Rotate secret | Vault + bump version |
| View receipts/jobs | Support/debug (redacted) |

Permissions: `admin.integration.manage`, `integrations.instance.manage` (as published in AuthZ catalog).

---

## 14. Folder / Crate Alignment

```text
crates/modules/proven-integrations/
  domain/          # ConnectorInstance, SyncJob, Receipt aggregates
  application/     # Enable, handle webhook, run sync
  connectors/
    ms_outlook/
    ms_teams/
    whatsapp_business/
    bc_crane_safety/
    worksafebc/
    generic_rest/
  infrastructure/  # HTTP, secret refs, sqlx
```

Go: channel adapters under `internal/notify` for Teams/WhatsApp delivery; agency file transfer I/O if needed under workers with API callbacks.

---

## 15. Security Summary

| Topic | Control |
| --- | --- |
| SSRF | Outbound URL allowlist / deny RFC1918 |
| WAF | Cloudflare on webhook paths |
| Tenant isolation | Instance → tenant_id mandatory |
| Least privilege | Per-connector service grants |
| Signing | Verify all inbound; sign outbound |
| Break-glass | Audited |

---

## 16. Testing

| Layer | Focus |
| --- | --- |
| Unit | Signature verify, mappers, retry classifier |
| Contract | Provider sandboxes / recorded fixtures |
| Idempotency | Duplicate webhook → one command |
| Chaos | 429 storms; expired OAuth |
| Security | SSRF attempts; unsigned webhook rejected |

---

## 17. Success Criteria

1. New partners plug in as connector packs without framework rewrites.  
2. Outlook, Teams, WhatsApp, BC Crane Safety, WorkSafeBC fit the same lifecycle model.  
3. Webhooks are verified, idempotent, and fast-ACK.  
4. REST outbound retries safely with checkpoints.  
5. Secrets never rest in plaintext application tables.  
6. Domain modules remain the only writers of compliance state.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Integration Architecture | Generic framework + initial connectors |

---

*End of Integration Framework Architecture*
