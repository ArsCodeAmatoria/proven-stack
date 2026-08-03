# Proven — Notification Domain Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Notification Domain Architecture (DDD) |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Principal Architecture |
| **Audience** | Engineering, Product, Design, DevEx |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Domain Model](./DOMAIN_MODEL.md), [Core Domain](./CORE_DOMAIN.md), [System Architecture](./SYSTEM_ARCHITECTURE.md), [Signatures Domain](./SIGNATURES_DOMAIN.md), [PRD](../PRD.md) |

---

## 1. Purpose

This document defines the **Notification** bounded context for Proven.

Notifications is a **supporting domain** of the Construction Compliance Operating System. It turns domain events and workflow signals into timely, preference-aware messages across **In-App**, **Push**, **Email**, **Microsoft Teams**, **WhatsApp Business**, and **SMS (future)**—without owning compliance business rules.

Go workers **deliver**; the Notifications module **decides** what is notifiable, to whom, on which channels, at what priority, with templates, queues, digests, escalations, and read state.

**Documentation only — no implementation.**

---

## 2. Bounded Context

### 2.1 Context Card

| Field | Value |
| --- | --- |
| **Name** | Notifications |
| **Module** | `notifications` |
| **Strategic type** | Supporting domain |
| **Product metaphor** | Notify = actionable awareness, not a second work queue SoR |
| **System of record for** | Notification records, templates, delivery rules, preferences, subscriptions, read status, priority classification, escalation policies (notification-side), digest schedules, delivery attempts/queue state, channel provider configs (non-secret metadata) |
| **Not system of record for** | My Actions / domain assignments (subject modules), Temporal business timers (Workflows), AuthZ (Core), provider secrets (platform secrets), compliance outcomes |

### 2.2 Supported Channels

| Channel | Status | Typical use |
| --- | --- | --- |
| **In-App** | Required | Default inbox + bell |
| **Push** | Required (PWA/device) | Field-critical assignments |
| **Email** | Required | Approvals, expiries, digests |
| **Microsoft Teams** | Supported | Supervisor/office alerts via tenant connectors |
| **WhatsApp Business** | Supported | Opt-in field messaging where permitted |
| **SMS** | Future | Opt-in critical escalations; regional compliance gating |

### 2.3 Context Map

```text
All modules ──domain events──► NATS
Workflows ──notify signals──► Notifications API
        │
        ▼
┌────────────────────────────────────────────┐
│              NOTIFICATIONS                 │
│  Rules · Templates · Prefs · Queue · Read  │
└──────────────────┬─────────────────────────┘
                   │ enqueue delivery jobs
                   ▼
            Go Workers ──► Email / Push / Teams / WhatsApp / (SMS)
                   │
                   ▼
            DeliveryAttempt status ──► Notifications
```

### 2.4 Ubiquitous Language

| Term | Meaning |
| --- | --- |
| **Notification** | Persisted message intended for a recipient about a business fact |
| **Template** | Parameterized content per channel/locale |
| **Delivery Rule** | Tenant policy mapping event/process → priority/channels/forced flags |
| **Preference** | User choices within policy ceilings |
| **Subscription** | Opt-in to topic/event classes (beyond defaults) |
| **Queue** | Durable outbound delivery work items |
| **Read Status** | Unread / Read / Dismissed (in-app primarily) |
| **Priority** | Low / Normal / High / Critical |
| **Escalation** | Re-notify or widen audience after non-ack/non-read within SLA |
| **Digest** | Bundled periodic email (or Teams) of lower-priority items |

---

## 3. Responsibilities (Mapped to Ownership)

| Responsibility | Notifications owns? | Clarification |
| --- | --- | --- |
| **Notification Templates** | Yes | Multi-channel, localized |
| **Queues** | Yes (logical) | Postgres/outbox + worker queues; Redis optional for rate limits only |
| **Preferences** | Yes | User channel/quiet hours within tenant policy |
| **Subscriptions** | Yes | Topic subscriptions |
| **Read Status** | Yes | In-app read/dismiss |
| **Priority** | Yes | Classification + routing |
| **Escalations** | Notification-side | Business overdue still owned by subject workflows; Notifications executes notify escalations |
| **Digest Emails** | Yes | Schedule + compose digests |
| **Workflow Integration** | Yes (signals in) | Temporal activities call Notifications; digests/escalations may use Temporal timers |

---

## 4. Aggregate Roots

| Aggregate | Responsibility |
| --- | --- |
| **NotificationTemplate** | Template code, channel variants, locale bodies, required variables |
| **DeliveryRule** | Event/process mapping to priority, channels, force-critical, escalation policy ref |
| **NotificationPreference** | Per-user (and optional per-tenant default) channel & quiet-hour settings |
| **Subscription** | User/team subscription to a topic |
| **Notification** | Single notification instance to a recipient |
| **DeliveryJob** | Queue item for a channel send |
| **EscalationPolicy** | Steps: wait → rechannel / widen recipients |
| **DigestSchedule** | Cadence and inclusion rules |
| **DigestBatch** | Materialized digest instance |
| **ChannelConnector** | Tenant Teams/WhatsApp connector config metadata |
| **NotificationReportingProjection** | Delivery/read metrics |

---

## 5. Entities

### 5.1 Templates & Rules

| Entity | Parent | Description |
| --- | --- | --- |
| **TemplateVariant** | NotificationTemplate | Channel + locale body/subject |
| **TemplateVariable** | NotificationTemplate | Declared placeholders |
| **RuleMatch** | DeliveryRule | Event type / process type / module filters |
| **ForcedChannel** | DeliveryRule | Channels that preferences cannot fully mute |

### 5.2 Preferences & Subscriptions

| Entity | Description |
| --- | --- |
| **ChannelPreference** | Enable/disable per channel |
| **QuietHours** | Local-time mute window |
| **TopicSubscription** | Subscribe/unsubscribe to topic codes |
| **ContactEndpoint** | Resolved email/push token/Teams webhook/WhatsApp number refs (tokens via secure store ids) |

### 5.3 Notification & Delivery

| Entity | Description |
| --- | --- |
| **NotificationTarget** | Recipient principal/person/guest endpoint |
| **NotificationPayload** | Rendered or structured data |
| **ReadState** | Read/dismiss timestamps |
| **DeliveryAttempt** | Try number, provider response, error class |
| **DedupRecord** | DedupKey retention for at-least-once events |
| **EscalationStepExecution** | Which escalation step fired |
| **DigestItem** | Notification ids included in a digest |

---

## 6. Value Objects

- `NotificationId`, `TemplateCode`, `DeliveryJobId`, `DigestBatchId`
- `Channel` — InApp | Push | Email | Teams | WhatsApp | SMS
- `Priority` — Low | Normal | High | Critical
- `NotificationStatus` — Pending | Queued | PartiallyDelivered | Delivered | Failed | Cancelled | Superseded
- `DeliveryStatus` — Pending | Sending | Sent | Failed | DeadLetter | Skipped
- `ReadStatus` — Unread | Read | Dismissed
- `RecipientRef` — UserId / PersonId / GuestRef / TeamId (expand)
- `DedupKey`
- `EventType`, `CorrelationId`, `TenantId`, `ProjectId?`
- `ProviderRef`, `ProviderMessageId`
- `QuietHours`, `LocaleCode`
- `EscalationStep` — delay + action
- `ErrorClass` — Transient | Permanent | RateLimited | OptOut | Config

---

## 7. Relationships

```text
Domain Event / Workflow Signal
        │
        ▼
DeliveryRule ──selects──► TemplateCode + Priority + Channels + EscalationPolicy
        │
        ▼
NotificationPreference + Subscription ──filter──► allowed channels
        │
        ▼
Notification (InApp always for actionable? per rule)
        ├── ReadState
        └── DeliveryJob * (per channel)
                └── DeliveryAttempt *
                        └── Go worker provider I/O

DigestSchedule ──buckets──► Low/Normal notifications ──► DigestBatch ──► Email/Teams job

EscalationPolicy ──on unread/unacked──► new Notification / widen audience
        ▲
        └── may be triggered by Temporal timer OR Notifications scheduler
```

### 7.1 Relationship to My Actions

```text
My Actions (UX queue) = domain assignments (Safety/Training/Signatures/…)
Notifications         = alerts that something needs attention
```

Deep links in notifications should open the owning module’s action—not duplicate SoR.

---

## 8. Domain Events

### 8.1 Lifecycle

- `NotificationCreated`
- `NotificationQueued`
- `NotificationDispatched`
- `NotificationDelivered`
- `NotificationPartiallyDelivered`
- `NotificationFailed`
- `NotificationCancelled`
- `NotificationRead`
- `NotificationDismissed`

### 8.2 Configuration

- `NotificationTemplatePublished`
- `DeliveryRuleChanged`
- `PreferenceUpdated`
- `SubscriptionChanged`
- `EscalationPolicyChanged`
- `DigestScheduleChanged`
- `ChannelConnectorConfigured`

### 8.3 Delivery & Escalation

- `DeliveryAttemptStarted`
- `DeliveryAttemptSucceeded`
- `DeliveryAttemptFailed`
- `DeliveryDeadLetter`
- `NotificationEscalated`
- `DigestBatchCreated`
- `DigestBatchSent`

Inbound integration (consumed, not owned):

- Subject-module domain events (`*Assigned`, `*Expiring`, `SignaturePackageCreated`, `GapOpened`, …)
- Workflow notify activities

---

## 9. Business Rules

### 9.1 Creation

1. Notifications are created from **events/signals**, not by workers inventing outcomes.  
2. Content rendered from templates + event payload DTOs; missing required variables → fail closed to DLQ/alert.  
3. `DedupKey` prevents duplicate notifications on at-least-once bus delivery.  
4. Tenant isolation mandatory.

### 9.2 Preferences vs Policy

1. Users may disable non-forced channels.  
2. **Critical** / forced channels (tenant DeliveryRule) cannot be fully muted.  
3. Quiet hours suppress Low/Normal; High may delay; Critical bypasses quiet hours.  
4. WhatsApp/SMS require explicit opt-in subscription + regional consent records.

### 9.3 Priority

| Priority | Routing tendency |
| --- | --- |
| Low | Digest candidates; in-app only optional |
| Normal | In-app + email/push per prefs |
| High | Immediate multi-channel |
| Critical | Forced channels + escalation eligible |

### 9.4 Read Status

1. Read status applies primarily to In-App.  
2. Marking read does not complete domain work.  
3. Escalations may key off unread **or** subject-module uncompleted state (prefer subject completion signals when available).

### 9.5 Digests

1. Digest includes only eligible Low/Normal items not already escalated/critical.  
2. Items notified immediately are excluded from next digest (or marked summarized—policy).  
3. Digest send failure retries like email jobs.

### 9.6 Escalations

1. Notification escalations widen audience or change channel; they do not change Safety CA due dates (Workflows/Safety own that).  
2. Steps are ordered and audited.  
3. Stop when notification cancelled, subject completion event received, or max step reached.

### 9.7 Teams / WhatsApp

1. Tenant connector must be configured and healthy.  
2. Teams posts to approved webhooks/channels mapping.  
3. WhatsApp uses Business-approved templates where providers require; map Proven templates → provider template ids.  
4. Failures surface as delivery failures—not silent drops.

### 9.8 SMS (Future)

1. Same preference/opt-in model.  
2. Extra compliance: consent, sender registration, quiet hours by jurisdiction.  
3. Channel enum reserved; rules can reference SMS before enablement (no-op until flag on).

---

## 10. Workflow Integration

| Integration | Pattern |
| --- | --- |
| **Inbound from Temporal** | Activity `NotifyUser` / `NotifyRole` / `ScheduleDigest` calls Notifications API |
| **Outbound timers** | DigestSchedule and EscalationPolicy delays via Temporal **or** Notifications internal scheduler; prefer Temporal for durability consistency |
| **Subject workflows** | CA overdue workflow emits event → Notifications rule → Critical notify + escalate |
| **Signature reminders** | Signatures reminder workflow → Notifications template `signatures.reminder` |

### 10.1 Sequence

```text
Safety CorrectiveActionOverdue event
  → DeliveryRule match (Critical)
  → create Notification (in-app)
  → enqueue Push + Email + Teams (forced)
  → start EscalationPolicy timers
  → if still open (subject signal absent) → notify manager
```

Workers never decide overdue; they only send.

---

## 11. Retry Strategy

### 11.1 Attempt Classes

| Error class | Behavior |
| --- | --- |
| **Transient** | Exponential backoff + jitter; max attempts N |
| **RateLimited** | Respect provider retry-after; isolate per connector |
| **Permanent** | Do not retry; mark Failed; alert if Critical |
| **OptOut / Unsubscribed** | Skip channel; try fallback if rule allows |
| **Config** | Skip; page ops for connector repair |

### 11.2 Backoff (Illustrative Policy)

- Attempts: 1 immediate, then 1m, 5m, 30m, 2h (configurable per channel)  
- Cap: e.g., 8 attempts or 24h window  
- Then **DeadLetter** with operator visibility  
- Critical dead-letters page on-call / admin inbox  

### 11.3 Idempotency

1. DeliveryJob id + attempt number unique.  
2. Provider idempotency keys where supported.  
3. At-least-once workers → DedupKey on Notification create; provider-side de-dupe best effort.  

### 11.4 Poison Messages

- DLQ retention with replay tooling  
- Replay reuses same Notification id where safe; otherwise supersede  

### 11.5 Partial Success

- Multi-channel: In-App success + Email fail ⇒ `PartiallyDelivered`  
- Escalation may retry failed channels only  

---

## 12. Permissions

| Code | Intent |
| --- | --- |
| `notifications.inbox.read` | Read own notifications |
| `notifications.inbox.manage` | Mark read/dismiss own |
| `notifications.preference.manage_self` | Own preferences |
| `notifications.subscription.manage_self` | Own subscriptions |
| `notifications.template.manage` | Templates admin |
| `notifications.rule.manage` | Delivery rules / forced channels |
| `notifications.escalation.manage` | Escalation policies |
| `notifications.digest.manage` | Digest schedules |
| `notifications.connector.manage` | Teams/WhatsApp connectors |
| `notifications.send.impersonate` | Ops/system send on behalf (rare, audited) |
| `notifications.dlq.manage` | Replay dead letters |
| `notifications.reports.read` | Delivery reporting |

Guests do not get full inbox; guest flows use direct magic-link channels from Signatures/Documents.

---

## 13. Public Interfaces & API (Summary)

### 13.1 Interfaces

| Interface | Purpose |
| --- | --- |
| `NotifyApi` | Create notification from trusted module/workflow callers |
| `InboxApi` | List/read/dismiss |
| `PreferenceApi` | Get/update preferences & subscriptions |
| `TemplateAdminApi` | Manage templates |
| `RuleAdminApi` | Delivery rules / escalations / digests |
| `ConnectorAdminApi` | Teams/WhatsApp configuration |
| `DeliveryOpsApi` | DLQ replay, job status |

### 13.2 HTTP (Illustrative)

Base: `/api/notifications`

- `/inbox`, `/inbox/{id}/read`
- `/preferences`, `/subscriptions`
- `/templates`, `/rules`, `/escalations`, `/digests`
- `/connectors/teams`, `/connectors/whatsapp`
- `/ops/dlq`
- `/reports/...`

Internal event consumers are not public HTTP.

---

## 14. Reporting

| Report | Purpose |
| --- | --- |
| Delivery success rate by channel | Reliability |
| Latency p95 create→delivered | Performance |
| Unread aging | Engagement |
| Critical failure/DLQ volume | Ops health |
| Opt-in rates WhatsApp/SMS | Adoption/compliance |
| Digest open rates (email provider) | Effectiveness |
| Escalation fire rate | Process pressure |
| Preference mute distribution | Policy tuning |

Metrics feed ops dashboards; optional Analytics events for product insights. No PHI in notification analytics payloads.

---

## 15. Security & Privacy

1. Templates must not embed secrets.  
2. Payload minimization—ids + short summaries; fetch detail in-app after authz.  
3. Connector secrets in platform secret store—not in Notifications DB plaintext.  
4. Tenant isolation on inbox queries.  
5. WhatsApp/SMS consent records retained for audit.  
6. Admin sends impersonating users are Core-audited.  
7. Push tokens treated as sensitive endpoints.

---

## 16. Data Ownership

### 16.1 Schema `notifications` Owns

- Templates, rules, preferences, subscriptions  
- Notifications, read state  
- Delivery jobs/attempts, dedup  
- Escalation policies/executions  
- Digest schedules/batches  
- Connector metadata  
- Reporting projections  

### 16.2 Not Owned

| Concern | Owner |
| --- | --- |
| Business overdue truth | Subject modules + Workflows |
| User contact master email | Core / People (resolved at send) |
| Provider transport | Go workers + vendors |
| AuthZ to view linked resource | Core + subject module on deep link |

---

## 17. Integration With Other Modules

| Module | Interaction |
| --- | --- |
| **All domains** | Emit events → notification rules |
| **Workflows** | Explicit notify activities; shared timers for digest/escalation |
| **Core** | Resolve users/endpoints; AuthZ; audit admin actions |
| **People** | Optional contact channels; no preference SoR fork |
| **Signatures** | Reminder/invite templates |
| **Web/PWA** | Inbox UI + push subscription |
| **Go workers** | Channel I/O only |
| **Analytics** | Optional delivery/product metrics |

---

## 18. Anti-Patterns

1. Encoding Safety/Training invariants in Go workers  
2. Using Notifications as the assignment SoR  
3. Infinite retries without DLQ  
4. Letting users mute Critical forced channels  
5. Putting Redis as permanent notification store  
6. Sending full medical/PII bodies in email/WhatsApp  
7. Dual preference databases in People and Notifications  

---

## 19. Success Criteria

Notifications is correctly designed when:

1. Field and office users get the right channel for the right priority.  
2. Preferences work without defeating critical safety escalations.  
3. Teams/WhatsApp/Email/Push share one rule/template model; SMS can join later.  
4. Retries are bounded, classified, and operable via DLQ.  
5. Digests reduce noise without hiding Critical/High items.  
6. Workflows and domain events integrate cleanly; workers remain delivery-only.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Principal Architecture | Initial Notification domain architecture (multi-channel) |

---

*End of Notification Domain Architecture*
