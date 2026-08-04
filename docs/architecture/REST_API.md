# Proven — REST API Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | REST API Architecture & Endpoint Catalog |
| **Version** | 1.0 (API surface: **v1**) |
| **Status** | Draft |
| **Owner** | API Architecture |
| **Audience** | Backend, Frontend, Integrations, Partners |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Rust Backend](./RUST_BACKEND_ARCHITECTURE.md), [Core Domain](./CORE_DOMAIN.md), module domain docs, [System Architecture](./SYSTEM_ARCHITECTURE.md) |

---

## 1. Purpose

This document defines the **REST API** for Proven’s Construction Compliance Operating System.

It covers conventions (JWT, OAuth-ready auth, versioning, pagination, filtering, sorting, search, rate limits, errors, validation, OpenAPI) and **endpoint documentation** for Companies, Projects, Workers, Equipment, Documents, Training, FLHAs, Inspections, COR, Notifications, Digital Signatures, Reports, and Workflow.

**Conventions implemented** — see [ADR-0013](../adr/0013-rest-api-conventions.md) and
[REST_API_CONVENTIONS.md](../development/REST_API_CONVENTIONS.md) (`proven-shared` wire types +
`proven-platform` middleware). Endpoint catalog below remains the product surface design.

**Base URL (logical):** `https://api.proven.example/api/v1`

---

## 2. Design Principles

1. **Resource-oriented REST** — nouns for resources; HTTP methods for actions; RPC-style only for genuine state transitions (`/activate`, `/seal`).  
2. **Module-aligned paths** — ownership matches bounded contexts.  
3. **Tenant isolation** — tenant inferred from auth context (not client-supplied as authority).  
4. **Additive v1 evolution** — breaking changes require v2.  
5. **Server AuthZ is authoritative** — clients never decide permissions.  
6. **Idempotent writes** where offline/field clients need them (`Idempotency-Key`).  
7. **OpenAPI is the machine-readable source of truth** for HTTP contracts.

---

## 3. Authentication

### 3.1 JWT (Primary Session Token)

| Item | Spec |
| --- | --- |
| Transport | `Authorization: Bearer <access_token>` and/or secure HTTP-only session cookie (web) |
| Claims (logical) | `sub` (principal/user id), `tid` (tenant id), `sid` (session), `exp`, `iat`, `amr`/`acr` (assurance) |
| Lifetime | Short-lived access token; refresh via session/refresh flow |
| Revocation | Server-side session revoke invalidates tokens |

### 3.2 OAuth / OIDC Ready

| Flow | Use |
| --- | --- |
| **Authorization Code + PKCE** | Enterprise SSO (customer IdP) |
| **Client Credentials** | Server-to-server integrations (API clients) |
| **Token exchange / linking** | Map IdP subject → Proven user (Core) |

Endpoints (auth):

| Method | Path | Purpose |
| --- | --- | --- |
| `POST` | `/auth/login` | Password/magic-link start (if enabled) |
| `POST` | `/auth/logout` | Revoke session |
| `POST` | `/auth/refresh` | Rotate access token |
| `GET` | `/auth/oauth/authorize` | OIDC start |
| `GET` | `/auth/oauth/callback` | OIDC callback |
| `POST` | `/auth/token` | OAuth token endpoint (clients) |
| `GET` | `/auth/me` | Current principal profile |

### 3.3 Other Auth Modes

| Mode | Use |
| --- | --- |
| **API keys** | Integration clients (`Authorization: Bearer pk_…` or `X-Api-Key`) |
| **Guest / magic link token** | Scoped signing routes only |
| **Service auth** | Workers → API with constrained credentials |

Unauthenticated: health, OpenAPI (optional), OAuth callbacks, guest redeem.

---

## 4. Authorization

- Permission codes from Core catalog (`projects.project.read`, `safety.activity.create`, …).  
- Resource scopes: tenant, org, project, self.  
- `403` on deny; `401` on missing/invalid auth.  
- Module entitlement checks may return `403` with code `module_disabled`.

---

## 5. Versioning

| Rule | Detail |
| --- | --- |
| URI version | `/api/v1/...` |
| Compatibility | Additive fields OK; renames/removals → v2 |
| Deprecation | `Deprecation` / `Sunset` headers + OpenAPI flags |
| Docs | This catalog + `contracts/openapi/v1.yaml` |

---

## 6. Cross-Cutting Request Conventions

### 6.1 Headers

| Header | Purpose |
| --- | --- |
| `Authorization` | Bearer JWT / API key |
| `Idempotency-Key` | UUID for safe retries (required on many POSTs from field clients) |
| `X-Correlation-Id` | Optional; server generates if absent |
| `Accept-Language` | Locale preference |
| `Content-Type` | `application/json` (except uploads) |

### 6.2 IDs

- Resource ids: UUID strings.  
- External codes (project code, asset tag) as separate fields—not path ids unless documented.

### 6.3 Timestamps

- ISO-8601 UTC (`2026-08-03T21:00:00Z`).

---

## 7. Pagination

**Cursor pagination (default for lists):**

```text
GET /resources?limit=50&cursor=eyJ...
```

Response envelope:

| Field | Meaning |
| --- | --- |
| `data` | Array of resources |
| `pagination.next_cursor` | Opaque cursor or `null` |
| `pagination.has_more` | Boolean |

Rules:

- Default `limit` 25; max 100 (reports export jobs for larger).  
- Stable sort key included in cursor.  
- Offset pagination **not** used for hot field lists.

---

## 8. Filtering

| Style | Example |
| --- | --- |
| Equality | `?status=active` |
| Multi | `?status=active,on_hold` or repeated params |
| Range | `?created_after=...&created_before=...` |
| Scope | `?project_id=<uuid>` |
| Boolean | `?overdue=true` |

Unknown filters → `400` with validation error (strict) **or** ignored if documented as open-search mode—**v1 = strict**.

---

## 9. Sorting

```text
?sort=created_at:desc
?sort=name:asc,created_at:desc
```

Whitelist per resource in OpenAPI. Default documented per collection.

---

## 10. Search

| Param | Use |
| --- | --- |
| `q` | Full-text / fuzzy across allowed fields |

Dedicated search:

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/search` | Global search (projects, workers, equipment, documents) |

AuthZ applied per hit type.

---

## 11. Rate Limits

| Tier | Guidance |
| --- | --- |
| Authenticated user | e.g. 600 req/min baseline |
| API client | Per-client quota from license |
| Auth endpoints | Stricter (e.g. 20/min) |
| Upload intents | Separate budget |

Headers:

- `X-RateLimit-Limit`  
- `X-RateLimit-Remaining`  
- `X-RateLimit-Reset`  

Exceed → `429` with `Retry-After`.

---

## 12. Error Responses

Standard problem envelope:

| Field | Meaning |
| --- | --- |
| `error.code` | Stable machine code (`validation_failed`, `forbidden`, …) |
| `error.message` | Human-readable |
| `error.details` | Optional field errors array |
| `error.correlation_id` | Support tracing |
| `error.doc_url` | Optional |

### 12.1 HTTP Status Map

| Status | Use |
| --- | --- |
| `400` | Malformed JSON / bad query types |
| `401` | Unauthenticated |
| `403` | Forbidden / module disabled |
| `404` | Not found (no cross-tenant leak) |
| `409` | Conflict (version, duplicate membership) |
| `412` | Precondition failed (If-Match) |
| `422` | Validation / domain invariant |
| `429` | Rate limited |
| `500` | Unexpected |
| `503` | Dependency unavailable |

### 12.2 Field Error Object

`{ "field": "due_at", "code": "required", "message": "..." }`

---

## 13. Validation

- Request bodies validated against OpenAPI + server Zod/Rust validators.  
- Domain invariants return `422` with domain codes (`activity_not_submittable`).  
- Optimistic concurrency: `ETag` / `If-Match` on mutable aggregates where documented.  
- Soft-deleted resources behave as `404` for normal reads.

---

## 14. OpenAPI

| Artifact | Location (repo) |
| --- | --- |
| Spec | `contracts/openapi/v1/openapi.yaml` (modular files merged) |
| Runtime | `GET /api/v1/openapi.json` |

Requirements:

- Security schemes: `bearerAuth`, `apiKey`, `oauth2`  
- All documented error responses  
- Examples for primary flows  
- CI breaking-change detection  

---

## 15. Common Response Patterns

**Single resource:** `{ "data": { ... } }`  
**List:** `{ "data": [ ... ], "pagination": { ... } }`  
**Action:** `{ "data": { ... } }` returning updated resource  
**Async job:** `{ "data": { "job_id": "...", "status": "queued" } }` with status poll URL  

---

## 16. Resource: Companies

**Owner module:** Core  
**Path prefix:** `/companies`

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/companies` | List companies (`type`, `q`, `status`) |
| `POST` | `/companies` | Register company |
| `GET` | `/companies/{companyId}` | Get company |
| `PATCH` | `/companies/{companyId}` | Update company |
| `POST` | `/companies/{companyId}/deactivate` | Deactivate |

**Filters:** `type` (prime|subcontractor|…), `status`, `q`  
**Sort:** `name`, `created_at`  
**AuthZ:** `core.company.*`

---

## 17. Resource: Projects

**Owner module:** Projects (+ Core membership endpoints)  
**Path prefix:** `/projects`

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/projects` | List projects |
| `POST` | `/projects` | Create project |
| `GET` | `/projects/{projectId}` | Project detail |
| `PATCH` | `/projects/{projectId}` | Update details |
| `POST` | `/projects/{projectId}/activate` | Activate |
| `POST` | `/projects/{projectId}/hold` | Put on hold |
| `POST` | `/projects/{projectId}/resume` | Resume |
| `POST` | `/projects/{projectId}/close` | Close |
| `POST` | `/projects/{projectId}/archive` | Archive |
| `GET` | `/projects/{projectId}/participants` | List prime/sub/client |
| `POST` | `/projects/{projectId}/participants` | Add participant |
| `PATCH` | `/projects/{projectId}/participants/{participantId}` | Update |
| `DELETE` | `/projects/{projectId}/participants/{participantId}` | Remove |
| `GET/PUT` | `/projects/{projectId}/location` | Location |
| `GET/POST` | `/projects/{projectId}/areas` | Areas |
| `GET/PUT` | `/projects/{projectId}/settings` | Settings |
| `GET/POST` | `/projects/{projectId}/controls` | Required controls |
| `GET` | `/projects/{projectId}/dashboard` | Place dashboard |
| `GET/POST` | `/projects/{projectId}/memberships` | Orchestrate Core membership |
| `DELETE` | `/projects/{projectId}/memberships/{personId}` | Unassign worker |
| `GET/POST` | `/project-templates` | Templates |

**Filters:** `status`, `q`, `region_code`  
**Sort:** `name`, `updated_at`, `status`  
**AuthZ:** `projects.*` + membership visibility

---

## 18. Resource: Workers (People)

**Owner module:** People  
**Path prefix:** `/workers` (alias of people; OpenAPI title **Workers**)

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/workers` | Directory search |
| `POST` | `/workers` | Register person |
| `GET` | `/workers/{personId}` | Profile |
| `PATCH` | `/workers/{personId}` | Update profile |
| `POST` | `/workers/{personId}/deactivate` | Deactivate |
| `GET/POST` | `/workers/{personId}/roles` | Workforce roles |
| `GET/POST` | `/workers/{personId}/trades` | Trades |
| `GET/POST` | `/workers/{personId}/emergency-contacts` | Emergency contacts |
| `GET/POST` | `/workers/{personId}/medical-restrictions` | Medical (sensitive) |
| `GET` | `/workers/{personId}/competency` | Competency view (`project_id` query) |
| `GET` | `/workers/{personId}/assignments` | Project assignment views |
| `GET` | `/workers/{personId}/availability` | Availability range |
| `PUT` | `/workers/{personId}/availability` | Set availability |
| `GET/POST` | `/workers/{personId}/attendance` | Attendance |
| `GET` | `/workers/{personId}/history` | Timeline |

**Filters:** `q`, `trade`, `status`, `project_id`, `workforce_role`  
**Sort:** `name`, `created_at`  
**AuthZ:** `people.*` (medical separate)

Also: `GET /workers/me` → current linked person.

---

## 19. Resource: Equipment

**Owner module:** Equipment  
**Path prefix:** `/equipment`

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/equipment/assets` | List/filter fleet |
| `POST` | `/equipment/assets` | Register asset |
| `GET` | `/equipment/assets/{assetId}` | Asset detail |
| `PATCH` | `/equipment/assets/{assetId}` | Update |
| `POST` | `/equipment/assets/{assetId}/assign` | Assign project/person |
| `POST` | `/equipment/assets/{assetId}/unassign` | Unassign |
| `POST` | `/equipment/assets/{assetId}/hold` | Manual hold |
| `POST` | `/equipment/assets/{assetId}/retire` | Retire |
| `GET` | `/equipment/assets/{assetId}/readiness` | Readiness decision |
| `POST` | `/equipment/assets/{assetId}/qr` | Bind QR |
| `GET` | `/equipment/qr/{code}` | Resolve QR |
| `GET/POST` | `/equipment/assets/{assetId}/photos` | Photos |
| `GET/POST` | `/equipment/inspections` | List/create inspections |
| `GET` | `/equipment/inspections/{inspectionId}` | Get |
| `POST` | `/equipment/inspections/{inspectionId}/submit` | Submit |
| `GET/POST` | `/equipment/deficiencies` | Deficiencies |
| `POST` | `/equipment/deficiencies/{id}/clear` | Clear |
| `GET/POST` | `/equipment/maintenance-orders` | Maintenance |
| `GET/POST` | `/equipment/certifications` | Cert records |
| `GET/POST` | `/equipment/binders` | Crane binders |
| `GET` | `/equipment/binder-templates` | Templates |

**Filters:** `class`, `status`, `project_id`, `readiness`, `q`  
**Sort:** `asset_tag`, `updated_at`  
**AuthZ:** `equipment.*`

**Inspection kinds:** `pre_use` | `periodic` | … via body/query.

---

## 20. Resource: Documents

**Owner module:** Documents  
**Path prefix:** `/documents`

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/documents` | Library list |
| `POST` | `/documents` | Create document |
| `GET` | `/documents/{documentId}` | Metadata |
| `PATCH` | `/documents/{documentId}` | Update metadata |
| `POST` | `/documents/{documentId}/archive` | Archive |
| `GET` | `/documents/{documentId}/versions` | List versions |
| `POST` | `/documents/{documentId}/versions` | Create draft version |
| `GET` | `/documents/versions/{versionId}` | Version detail |
| `POST` | `/documents/versions/{versionId}/submit-review` | Review |
| `POST` | `/documents/versions/{versionId}/submit-approval` | Approval |
| `POST` | `/documents/versions/{versionId}/approve` | Approve |
| `POST` | `/documents/versions/{versionId}/reject` | Reject |
| `POST` | `/documents/versions/{versionId}/publish` | Publish |
| `GET` | `/documents/{documentId}/effective` | Effective version (`at` query) |
| `POST` | `/documents/assignments` | Create assignment/ack campaign |
| `GET` | `/documents/acknowledgements` | List pending/complete |
| `POST` | `/documents/acknowledgements/{id}/complete` | Complete ack |
| `POST` | `/documents/guest-sign` | Issue guest sign |
| `POST` | `/documents/qr-targets` | Issue QR sign target |
| `GET` | `/documents/search` | Document search (`q`) |
| `GET/POST` | `/documents/templates` | Templates |
| `GET/POST` | `/documents/retention-policies` | Retention |
| `POST` | `/documents/{documentId}/legal-holds` | Legal hold |

**Filters:** `category`, `status`, `project_id`, `q`  
**AuthZ:** `documents.*`

**Upload:** `POST /files/upload-intents` (Core files) then complete; version content references `file_object_id`.

---

## 21. Resource: Training

**Owner module:** Training  
**Path prefix:** `/training`

| Method | Path | Purpose |
| --- | --- | --- |
| `GET/POST` | `/training/courses` | Courses/orientations |
| `GET/PATCH` | `/training/courses/{courseId}` | Course |
| `GET/POST` | `/training/competencies` | Competency definitions |
| `GET/POST` | `/training/evaluations/definitions` | Eval templates |
| `POST` | `/training/evaluations/attempts` | Start attempt |
| `POST` | `/training/evaluations/attempts/{id}/submit` | Submit |
| `GET/POST` | `/training/requirements` | Requirements |
| `GET/POST` | `/training/assignments` | Assignments |
| `POST` | `/training/assignments/{id}/complete` | Complete (evidence) |
| `GET/POST` | `/training/completions` | Record/list completions |
| `POST` | `/training/completions/{id}/revoke` | Revoke |
| `GET/POST` | `/training/waivers` | Waivers |
| `GET/POST` | `/training/renewals` | Renewal cases |
| `GET` | `/training/matrix` | Matrix (`project_id` optional) |
| `GET` | `/training/competency` | `GetPersonCompetency` (`person_id`, `project_id`) |
| `GET/POST` | `/training/toolbox-library` | Toolbox content library |
| `GET` | `/training/reports/{reportKey}` | Training reports |

**Filters:** `status`, `person_id`, `project_id`, `course_id`, `overdue`  
**AuthZ:** `training.*`

---

## 22. Resource: FLHAs

**Owner module:** Safety  
**Path prefix:** `/safety/flhas`  
*(Also available generically under `/safety/activities?type=flha`.)*

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/safety/flhas` | List FLHAs |
| `POST` | `/safety/flhas` | Start FLHA |
| `GET` | `/safety/flhas/{activityId}` | Detail |
| `PATCH` | `/safety/flhas/{activityId}` | Update draft/in progress |
| `POST` | `/safety/flhas/{activityId}/submit` | Submit |
| `POST` | `/safety/flhas/{activityId}/review` | Review |
| `POST` | `/safety/flhas/{activityId}/close` | Close |
| `POST` | `/safety/flhas/{activityId}/void` | Void |
| `POST` | `/safety/flhas/{activityId}/hazards` | Add hazard entry |
| `POST` | `/safety/flhas/{activityId}/controls` | Add control |
| `POST` | `/safety/flhas/{activityId}/request-signatures` | Start seal package |
| `GET` | `/safety/activity-types` | Type catalog including FLHA |

**Filters:** `project_id`, `status`, `created_after`, `q`  
**Idempotency-Key:** required for POST/PATCH from offline clients  
**AuthZ:** `safety.activity.*`

Related safety collections (same module): `/safety/toolbox-talks`, `/safety/corrective-actions`, `/safety/incidents`, `/safety/bulletins`, `/safety/permits`, `/safety/lift-plans`, `/safety/daily-logs`, `/safety/libraries/hazards`, `/safety/libraries/controls`.

---

## 23. Resource: Inspections

Inspections span **Equipment pre-use/periodic** and **Safety site inspections**.

### 23.1 Equipment Inspections

Prefix: `/equipment/inspections` (see §19)

| Method | Path | Purpose |
| --- | --- | --- |
| `POST` | `/equipment/inspections` | Start pre-use/periodic |
| `POST` | `/equipment/inspections/{id}/submit` | Submit pass/fail |

### 23.2 Safety Site Inspections

Prefix: `/safety/inspections`

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/safety/inspections` | List site inspections |
| `POST` | `/safety/inspections` | Start site inspection activity |
| `GET` | `/safety/inspections/{activityId}` | Detail |
| `PATCH` | `/safety/inspections/{activityId}` | Update |
| `POST` | `/safety/inspections/{activityId}/submit` | Submit |
| `POST` | `/safety/inspections/{activityId}/close` | Close |

**Discriminator:** OpenAPI tags `EquipmentInspections` vs `SafetyInspections`.  
**AuthZ:** respective module permissions.

---

## 24. Resource: COR

**Owner module:** COR  
**Path prefix:** `/cor`

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/cor/frameworks` | List BCCSA COR / SECOR / packs |
| `GET` | `/cor/frameworks/{frameworkId}` | Framework detail |
| `GET` | `/cor/frameworks/{frameworkId}/elements` | Elements |
| `GET` | `/cor/readiness` | Readiness profile (`subject_type`, `subject_id`, `framework_id`) |
| `POST` | `/cor/readiness/mappings` | Link evidence |
| `DELETE` | `/cor/readiness/mappings/{mappingId}` | Unlink |
| `GET` | `/cor/gaps` | Gap register |
| `PATCH` | `/cor/gaps/{gapId}` | Assign/update gap |
| `GET/POST` | `/cor/plans` | Audit plans |
| `GET/POST` | `/cor/engagements` | Internal / external prep audits |
| `GET` | `/cor/engagements/{id}` | Engagement detail |
| `POST` | `/cor/engagements/{id}/open` | Start fieldwork |
| `POST` | `/cor/engagements/{id}/close` | Close |
| `GET/POST` | `/cor/engagements/{id}/interviews` | Interviews |
| `GET/POST` | `/cor/engagements/{id}/observations` | Observations |
| `GET/POST` | `/cor/engagements/{id}/findings` | Findings |
| `POST` | `/cor/findings/{id}/close` | Close finding |
| `GET/POST` | `/cor/corrective-actions` | Audit CAs |
| `POST` | `/cor/engagements/{id}/score` | Calculate scorecard |
| `POST` | `/cor/packages` | Request evidence package |
| `GET` | `/cor/packages/{packageId}` | Package status |
| `GET` | `/cor/packages/{packageId}/download` | Authorized download |
| `POST` | `/cor/reports` | Generate report |
| `GET` | `/cor/reports/{reportId}` | Report metadata |
| `GET` | `/cor/history` | Historical audits |
| `GET` | `/cor/dashboard` | COR dashboard |

**Filters:** `framework_id`, `status`, `subject_id`  
**AuthZ:** `cor.*`

---

## 25. Resource: Notifications

**Owner module:** Notifications  
**Path prefix:** `/notifications`

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/notifications/inbox` | List inbox |
| `POST` | `/notifications/inbox/{id}/read` | Mark read |
| `POST` | `/notifications/inbox/read-all` | Mark all read |
| `POST` | `/notifications/inbox/{id}/dismiss` | Dismiss |
| `GET/PUT` | `/notifications/preferences` | Channel prefs |
| `GET/PUT` | `/notifications/subscriptions` | Topic subscriptions |
| `GET/POST` | `/notifications/templates` | Admin templates |
| `GET/POST` | `/notifications/rules` | Delivery rules |
| `GET/POST` | `/notifications/escalations` | Escalation policies |
| `GET/POST` | `/notifications/digests` | Digest schedules |
| `GET/POST` | `/notifications/connectors` | Teams/WhatsApp connectors |
| `GET` | `/notifications/reports/delivery` | Delivery stats |

**Internal:** workers call `POST /notifications/internal/delivery-attempts` (service auth)—not public partner API.

**Filters:** `read_status`, `priority`, `created_after`  
**AuthZ:** `notifications.*`

---

## 26. Resource: Digital Signatures

**Owner module:** Signatures  
**Path prefix:** `/signatures`

| Method | Path | Purpose |
| --- | --- | --- |
| `GET/POST` | `/signatures/policies` | Signing policies |
| `POST` | `/signatures/packages` | Create package |
| `GET` | `/signatures/packages/{packageId}` | Status/detail |
| `POST` | `/signatures/packages/{packageId}/void` | Void |
| `POST` | `/signatures/packages/{packageId}/slots` | Assign/reassign slots |
| `POST` | `/signatures/packages/{packageId}/seal` | Seal own slot (auth) |
| `POST` | `/signatures/magic-links` | Issue magic link |
| `POST` | `/signatures/magic-links/redeem` | Redeem (guest) |
| `POST` | `/signatures/qr-sessions` | Issue QR session |
| `GET` | `/signatures/qr/{code}` | Resolve QR |
| `POST` | `/signatures/qr-sessions/{id}/seal` | Seal via QR session |
| `GET` | `/signatures/certificates/{packageId}` | Evidence certificate metadata |
| `GET` | `/signatures/certificates/{packageId}/download` | Download artifact |
| `GET` | `/signatures/packages` | List (`subject_type`, `subject_id`, `status`) |

**Guest:** limited routes with magic-link token; no full API access.  
**AuthZ:** `signatures.*` + link tokens  
**Idempotency-Key:** seal retries

---

## 27. Resource: Reports

Reports are produced by **Analytics**, **COR**, and domain modules. Unified entry:

**Path prefix:** `/reports`

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/reports/catalog` | Available report keys by module |
| `POST` | `/reports/runs` | Start report/export job |
| `GET` | `/reports/runs/{jobId}` | Job status |
| `GET` | `/reports/runs/{jobId}/download` | Download artifact |
| `GET/POST` | `/reports/definitions` | Saved Analytics definitions |
| `GET/POST` | `/reports/subscriptions` | Scheduled delivery |

**Body (run):** `{ "report_key": "safety.ca_aging", "filters": {...}, "format": "csv"|"xlsx"|"pdf" }`

**AuthZ:** `analytics.export.*` / module `*.reports.read` as applicable  
**Async:** large runs return `202`-style job pattern (`200` with `status=queued` acceptable if documented).

Also module-native: `GET /training/reports/...`, `GET /cor/reports/...`, `GET /analytics/dashboards/...`.

---

## 28. Resource: Workflow

**Owner module:** Workflows (tracking) + Temporal runtime  
**Path prefix:** `/workflows`

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/workflows/definitions` | List definitions |
| `POST` | `/workflows/definitions` | Register/publish metadata |
| `GET` | `/workflows/definitions/{definitionId}` | Detail |
| `GET` | `/workflows/instances` | List instances |
| `GET` | `/workflows/instances/{instanceId}` | Status / milestones |
| `POST` | `/workflows/instances` | Start (authorized orchestration) |
| `POST` | `/workflows/instances/{instanceId}/signal` | Signal |
| `POST` | `/workflows/instances/{instanceId}/cancel` | Cancel |

**Filters:** `status`, `subject_type`, `subject_id`, `definition_id`  
**AuthZ:** admin/orchestration permissions; most starts happen internally from domain commands—public start limited.

**Note:** Clients generally do **not** drive Temporal directly; domain APIs start workflows. These endpoints expose visibility and controlled ops.

---

## 29. Supporting Endpoints

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/health` | Liveness |
| `GET` | `/ready` | Readiness |
| `GET` | `/search` | Global search |
| `POST` | `/files/upload-intents` | Presign upload (Core) |
| `POST` | `/files/{fileId}/complete` | Complete upload |
| `GET` | `/files/{fileId}` | Metadata / download auth |
| `GET` | `/admin/dashboard` | Admin composition |
| `GET/PUT` | `/admin/branding` | Branding |
| `GET/POST` | `/admin/api-keys` | API keys |
| `GET` | `/openapi.json` | OpenAPI document |

---

## 30. Idempotency & Concurrency

| Mechanism | Use |
| --- | --- |
| `Idempotency-Key` | POST/PATCH from PWA offline queues |
| `ETag` / `If-Match` | Patch project, activity, document metadata |
| `row_version` in body | Optional alternate |

Replay with same key returns original result (`200`/`201` as first response).

---

## 31. Webhooks (Future Partner Surface)

Not required for v1 core product UI, but OAuth-ready API anticipates:

- Partner registers webhook via Admin Integrations  
- Signed callbacks on subscribed event types  
- Documented separately when enabled  

---

## 32. Security Notes

- TLS everywhere  
- Tenant never taken from untrusted body for AuthZ root  
- Guest tokens scoped + short TTL  
- Rate limit auth and seal endpoints  
- Audit export and package download heavily permissioned  
- PII/PHI routes (medical) extra permission + audit  

---

## 33. Success Criteria

The REST API succeeds when:

1. Web/PWA and integrations share one versioned contract.  
2. Field clients can safely retry with idempotency.  
3. OpenAPI matches runtime and fails CI on breaks.  
4. Resources map cleanly to modules without cross-module god endpoints.  
5. JWT + OAuth/OIDC cover human and machine access.  
6. Errors are stable, correlatable, and safe across tenants.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | API Architecture | Complete REST API architecture & endpoint catalog (no implementation) |

---

*End of REST API Architecture*
