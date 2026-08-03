# Proven — Notification Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Notification Architecture |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Notification / Platform Architecture |
| **Audience** | Engineering, Product, Design, SRE |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Notifications Domain](./NOTIFICATIONS_DOMAIN.md), [Go Worker Catalog](./GO_WORKER_CATALOG.md), [Temporal Workflows](./TEMPORAL_WORKFLOWS.md), [Authentication](./AUTHENTICATION_ARCHITECTURE.md), [Frontend Folders](./FRONTEND_FOLDER_STRUCTURE.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document designs Proven’s **notification system**: channels (In-App, Push, Email, WhatsApp Business, Microsoft Teams, future SMS), priority, quiet hours, grouping, digest, escalation, retry, read receipts, and preferences.

**Hard rules**

1. Notifications **decides** recipients, channels, priority, templates, digests, and escalation **notify** steps.  
2. Go workers **deliver** only—no compliance authority.  
3. My Actions / domain assignments remain subject-module SoR; notifications are **awareness**, not a second work queue.  
4. Business overdue SLAs live in **Temporal/domain**; Notifications executes **notification-side** escalations.  
5. No secrets (magic-link plaintext, passwords) in notification payloads or logs.

**Documentation only — no implementation.**

---

## 2. Architecture Overview

```text
Domain events (NATS)  /  Workflow notify calls  /  Direct Notify API
                    │
                    ▼
         ┌──────────────────────┐
         │   NOTIFICATIONS      │
         │ Rules · Prefs · Dedup│
         │ Priority · Grouping  │
         │ Quiet hours · Digest │
         └──────────┬───────────┘
                    │ create Notification + DeliveryJobs
                    ▼
              NATS job queue
                    │
                    ▼
              notify-worker (Go)
                    │
        ┌───────────┼───────────┬──────────┬─────────┐
        ▼           ▼           ▼          ▼         ▼
     In-App*     Push       Email      Teams    WhatsApp
   (*persist +     FCM/APNs   Provider   Graph    Business
     realtime)                          connector API
                    │
                    ▼
         DeliveryAttempt callback → Notifications
                    │
                    ▼
         Escalation / Digest Temporal workflows
```

\*In-App is written to Postgres by the module and pushed to clients via realtime/poll; external providers optional.

---

## 3. Channels

### 3.1 Channel matrix

| Channel | Status | Best for | Consent / setup |
| --- | --- | --- | --- |
| **In-App** | Required | Default inbox, bell, deep links | Always on for authenticated users (prefs can mute categories) |
| **Push** | Required | Field-critical, mobile PWA | Device permission + user pref |
| **Email** | Required | Approvals, expiries, digests, guest links | Verified email; pref |
| **Microsoft Teams** | Supported | Office/supervisor alerts | Tenant connector + channel/user mapping |
| **WhatsApp Business** | Supported | Opt-in field messaging | Explicit opt-in + template approval |
| **SMS** | Future | Critical escalation fallback | Explicit opt-in + regional compliance |

### 3.2 In-App

| Aspect | Design |
| --- | --- |
| **SoR** | `Notification` row per recipient |
| **UX** | Bell + `/notifications` inbox; deep link to subject |
| **Realtime** | Prefer SSE/WebSocket or short poll; offline sees on sync |
| **Read state** | Unread / Read / Dismissed (§12) |
| **Grouping** | Inbox threads/groups (§8) |

### 3.3 Push

| Aspect | Design |
| --- | --- |
| **Targets** | PWA / device push subscriptions stored per user/device |
| **Payload** | Title, body, deep link, notification id—minimize PII |
| **Priority mapping** | High/Critical may use high-priority push where platform allows |
| **Failure** | Invalid token → deactivate subscription |
| **Quiet hours** | Honored unless Critical forced (§7) |

### 3.4 Email

| Aspect | Design |
| --- | --- |
| **Use** | Transactional: assignments, approvals, magic links, digests, exports ready |
| **Templates** | Per locale; tenant branding hooks |
| **Provider** | Go worker adapter (SendGrid/SES/etc.—implementation choice) |
| **Idempotency** | `delivery_attempt_id` → provider message id |
| **Bounce/complaint** | Mark channel unhealthy; audit; stop retry loops |

### 3.5 WhatsApp Business

| Aspect | Design |
| --- | --- |
| **Use** | Opt-in field alerts (signature remind, critical CA) where legally allowed |
| **Templates** | Pre-approved WA templates only; variable map from Notification data |
| **Consent** | Required; stored on preference/subscription; revocable |
| **Connector** | Tenant WhatsApp Business account metadata in Notifications; secrets in vault |
| **Not for** | Unsolicited marketing |

### 3.6 Microsoft Teams

| Aspect | Design |
| --- | --- |
| **Use** | Supervisor/office: adaptive cards or message to user/chat/channel |
| **Connector** | Tenant Teams app / webhook / Graph config |
| **Mapping** | Proven user → Teams AAD id when linked |
| **Digests** | Optional Teams digest card |

### 3.7 Future SMS

| Aspect | Design |
| --- | --- |
| **Use** | Last-resort Critical escalation when other channels fail/unregistered |
| **Gating** | Feature flag + regional/legal allowlist + explicit opt-in |
| **Content** | Short; deep link; no secrets |
| **Provider** | Go adapter when enabled—same delivery attempt model |

---

## 4. Priority

| Level | Meaning | Routing defaults |
| --- | --- | --- |
| **Low** | FYI | In-app; often digest-only for email |
| **Normal** | Action soon | In-app + email/push per prefs |
| **High** | Action needed | In-app + push + email (prefs ceiling) |
| **Critical** | Safety/compliance urgent | Force channels per DeliveryRule (may bypass quiet hours) |

### 4.1 Classification

- **DeliveryRule** maps event/process → default priority.  
- Emitters may suggest priority; module **clamps** to rule max/min.  
- Critical examples: missing critical FLHA, OOS on active lift, break-glass authz, severe incident open.

Priority stored on `Notification` and inherited by `DeliveryJob`s.

---

## 5. Quiet Hours

| Aspect | Design |
| --- | --- |
| **Definition** | Per-user window (timezone-aware) when non-Critical external channels are deferred |
| **Tenant ceiling** | Tenant may set max quiet span / disallow quiet for High |
| **Behavior** | Hold DeliveryJobs until window ends; In-App still written |
| **Critical** | Bypass quiet hours when DeliveryRule `force_critical` |
| **Digest** | Quiet hours do not block digest schedule (digest is itself batching) |
| **Guest links** | Signing invites may use “time-sensitive” rule—policy may bypass quiet for guest email |

Deferred jobs show reason `quiet_hours` until released.

---

## 6. Preferences

| Preference | Scope |
| --- | --- |
| Channel enable (email/push/Teams/WA/SMS) | User within tenant policy |
| Category mute (training reminders, marketing-like digests) | User |
| Quiet hours schedule + timezone | User |
| Digest cadence (daily/weekly/off) | User |
| Per-project mute (optional) | User |
| WhatsApp/SMS opt-in | User (explicit) |

### 6.1 Policy ceilings

Tenant **DeliveryRules** can:

- Force Critical onto email/push regardless of mute (documented).  
- Disallow muting safety-critical categories.  
- Require verified email for certain roles.

### 6.2 Effective channel selection

```text
channels = rule.default_channels
  ∩ user.preferences
  ∪ rule.forced_channels(priority)
  − unhealthy_channels
  then apply quiet-hours deferral
```

---

## 7. Grouping

| Mechanism | Purpose |
| --- | --- |
| **Dedupe key** | Same `(recipient, subject_ref, action_class)` within window → update/collapse instead of spam |
| **Thread id** | Group CA updates under one in-app thread |
| **Batch key** | Multiple training expiries → one grouped notification |
| **Inbox UI sections** | By project, priority, or category |

Grouping is **UX + write-path collapse**; each material state change may still audit in domain. Collapsed notifications keep latest body + count.

---

## 8. Digest

| Aspect | Design |
| --- | --- |
| **Purpose** | Bundle Low/Normal items into periodic email/Teams |
| **Schedule** | `DigestSchedule` + `DigestScheduleWorkflow` (Temporal) |
| **Inclusion** | Unread or undelivered-external items marked `digestible` |
| **Exclusion** | Critical/High already pushed; items with `force_immediate` |
| **Compose** | Create `DigestBatch` → one DeliveryJob email/Teams |
| **Prefs** | User cadence; off = no digest (immediates still flow) |
| **Idempotency** | Schedule tick + period key |

Digest does not remove in-app notifications; it reduces email noise.

---

## 9. Escalation

### 9.1 Two layers

| Layer | Owner | Example |
| --- | --- | --- |
| **Business escalation** | Domain + Temporal (CA overdue → reassign) | Subject status changes |
| **Notification escalation** | Notifications + `NotificationEscalationWorkflow` | Unread Critical after 15m → push + email manager |

### 9.2 Notification escalation policy

| Step | Example |
| --- | --- |
| 1 | Wait T1; if unread/unresolved → resend push |
| 2 | Wait T2; add email if not used |
| 3 | Wait T3; widen to supervisor / Teams channel |
| Stop | Read receipt, subject resolved signal, cancel, or max steps |

### 9.3 Inputs

- Prefer **subject resolved** signals (CA closed, slot sealed) over read-only.  
- Read alone may stop soft escalations; Critical business workflows continue independently.

### 9.4 Events

`EscalationTriggered` (workflow/notifications) for analytics/audit as appropriate.

---

## 10. Retry

| Stage | Policy |
| --- | --- |
| **Provider transient** | Exp backoff + jitter; honor `Retry-After` ([Go Worker Catalog](./GO_WORKER_CATALOG.md)) |
| **Provider permanent** | Fail attempt; mark; no infinite retry (bounce, invalid WA template) |
| **Quiet deferral** | Not a failure—release later |
| **Rate limit** | Per tenant/channel; queue delay |
| **Max attempts** | Per channel class (e.g. 8 for email) then dead-letter + optional escalate path |
| **Idempotency** | Same `delivery_attempt_id` must not double-send when provider supports keys |

Callbacks: `DeliveryAttemptSucceeded` / `Failed` update Notification delivery state.

---

## 11. Read Receipts

| Aspect | Design |
| --- | --- |
| **Primary** | In-App: `Unread` → `Read` (and optional `Dismissed`) |
| **API** | Mark one / mark all / optimistic UI |
| **Push/Email** | No reliable native read; optional open-pixel **not** used for compliance (privacy). Deep-link open may mark related in-app read |
| **Escalation** | Soft escalations stop on in-app Read when policy says |
| **Audit** | Mark-read generally **not** Core audit (noise); security-sensitive exceptions rare |
| **Multi-device** | Server SoR; sync across sessions |

“Read” ≠ “done.” Completing work is domain action.

---

## 12. Templates

| Aspect | Design |
| --- | --- |
| **Keys** | Stable `template_code` per event class |
| **Variants** | Channel + locale |
| **Variables** | Allowlisted; validated before send |
| **Branding** | Tenant branding for email/Teams |
| **WA/SMS** | Separate approved template ids |

Subject modules do not embed raw HTML email—they emit events/data.

---

## 13. Intake & Dedup Pipeline

```text
1. Receive event / Notify command
2. Authorize emitter (service or user)
3. Resolve recipients (explicit, role expand, project membership—via Core queries)
4. Apply DeliveryRule → priority + channels
5. Apply preferences + consent
6. Dedupe/group
7. Persist Notification (in-app)
8. Enqueue DeliveryJobs (external channels) or mark digestible
9. Start escalation workflow if policy attached
```

Recipient expansion never bypasses AuthZ visibility rules for deep-link targets (user may be notified only if they could access subject—or receive limited teaser per policy).

---

## 14. Guest & External Recipients

| Case | Design |
| --- | --- |
| **Guest sign email** | Recipient email on slot; no user prefs; time-sensitive rule |
| **External Teams channel** | Connector binding; not a Proven user inbox |
| **No in-app** | External-only delivery attempts |

---

## 15. Preferences UX & API (Logical)

| Capability | Surface |
| --- | --- |
| List/update prefs | Settings + `/notifications/preferences` |
| Device push register | Mobile/PWA |
| Inbox list/mark read | `/notifications` |
| Admin delivery rules | Admin console |
| Connector setup | Admin integrations |

Permissions: `notifications.prefer.self`; admin manage rules; users cannot disable forced Critical if policy forbids.

---

## 16. Observability

| Metric | Use |
| --- | --- |
| Enqueue → deliver latency | SLO |
| Success/fail by channel | Provider health |
| Quiet deferred count | UX |
| Digest batch size | Noise control |
| Escalation step rate | Policy tuning |
| Unsubscribe/opt-out | Consent health |

---

## 17. Security & Privacy

- Minimize PII in push/email bodies.  
- Magic links in email only; never in push payload logs.  
- WhatsApp/SMS opt-in recorded and auditable.  
- Tenant isolation on all notification rows.  
- Rate-limit notify APIs to prevent spam abuse.

---

## 18. Mapping Requirements → Design

| Requirement | Section |
| --- | --- |
| Channels | §3 |
| In-App / Push / Email / WA / Teams / SMS | §3.2–3.7 |
| Priority | §4 |
| Quiet Hours | §5 |
| Grouping | §7 |
| Digest | §8 |
| Escalation | §9 |
| Retry | §10 |
| Read Receipts | §11 |
| Preferences | §6, §15 |

---

## 19. Success Criteria

1. Every channel has a clear delivery path and consent model.  
2. Priority + quiet hours + prefs compose without dropping Critical safety alerts incorrectly.  
3. Grouping and digests reduce noise without hiding High/Critical immediates.  
4. Escalations stop on resolve/read per policy and do not replace domain SLAs.  
5. Retries are idempotent; permanent failures surface cleanly.  
6. Read state is server-authoritative for in-app; preferences are user-controllable within ceilings.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Notification Architecture | Channels, priority, digest, escalation |

---

*End of Notification Architecture*
