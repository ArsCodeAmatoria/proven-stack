# Proven — Progressive Web App (PWA) Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Progressive Web App Architecture |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | PWA / Frontend Architecture |
| **Audience** | Frontend, Mobile, Security, Product |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Frontend Architecture](./FRONTEND_ARCHITECTURE.md), [Frontend Folders](./FRONTEND_FOLDER_STRUCTURE.md), [Offline Sync](./OFFLINE_SYNC_ARCHITECTURE.md), [Digital Signatures](./DIGITAL_SIGNATURES_ARCHITECTURE.md), [Notification Architecture](./NOTIFICATION_ARCHITECTURE.md), [Authentication](./AUTHENTICATION_ARCHITECTURE.md), [UX Architecture](../ux/UX_ARCHITECTURE.md), [Design System](../design/DESIGN_SYSTEM.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document designs the **Proven Progressive Web App**: installability, offline field work, background sync, push notifications, camera/photo upload, GPS, QR scanning, digital signatures, caching, conflict resolution, updates, and performance.

**Hard rules**

1. PWA is the **worker-first** client (My Actions); desktop Command Center shares the Next.js app but field capabilities prioritize mobile install.  
2. **Server is SoR** — offline queues are durable intent ([Offline Sync](./OFFLINE_SYNC_ARCHITECTURE.md)).  
3. **No business invariants in the client** — Zod/UI only.  
4. **Never show “sealed/Proven”** until server ACK.  
5. Capability use is **permission + HTTPS + user gesture** constrained; secrets not in IndexedDB beyond auth policy.

**Documentation only — no implementation.**

---

## 2. Product Positioning

| Surface | PWA emphasis |
| --- | --- |
| **Mobile worker** | Installable, offline FLHA/inspection, camera, GPS stamp, QR sign, push |
| **Supervisor phone** | Review queues, push, light Place views |
| **Desktop** | Same origin app; install optional; offline allowlist still applies sparingly |

Stack: Next.js App Router + service worker (Serwist/Workbox or equivalent) + `packages/pwa-sync` + Web APIs (Camera, Geolocation, BarcodeDetector/getUserMedia, Push).

---

## 3. Installable

### 3.1 Requirements

| Item | Design |
| --- | --- |
| **Web app manifest** | `name`, `short_name`, icons (maskable), `start_url` (`/actions` or role-aware), `display: standalone`, theme/background colors per Design System |
| **HTTPS** | Required (Vercel/Cloudflare) |
| **Engagement** | Optional soft prompt after meaningful use—not blocking first paint |
| **iOS** | Add to Home Screen guidance; note SW/push limitations vs Android |
| **Icons** | `public/icons/` multi-size + monochrome where needed |

### 3.2 Standalone UX

- Hide browser chrome expectations; use in-app nav (MobileTabBar).  
- Safe-area insets; status bar theme.  
- Deep links from push/email open installed app when possible (`start_url` + path).

---

## 4. Offline

### 4.1 Allowlist (field)

| Allow offline | Online-only (initial) |
| --- | --- |
| Safety drafts/submit (typed) | Admin console |
| Pre-use inspections | COR package generation |
| Photo queue | Role/grant changes |
| Authenticated offline seal (policy) | Guest magic-link redeem |
| Cached reference libs (hazards, checklists) | Global search corpus |

### 4.2 UX states

| State | UI |
| --- | --- |
| Offline | Banner + Sync Pill “Offline” |
| Pending N | Sync Pill count |
| Syncing | Deterministic progress when known |
| Conflict | Sync Center + activity banner |
| Auth expired | Pause drain; prompt re-login |

Language: “Saved on this device — not yet proven on server.”

Full protocol: [Offline Sync Architecture](./OFFLINE_SYNC_ARCHITECTURE.md).

---

## 5. Background Sync

| Aspect | Design |
| --- | --- |
| **API** | Background Sync (where supported) registers tag e.g. `proven-outbox` |
| **Role** | Best-effort drain when connectivity returns in background |
| **Reliability** | **Foreground drain** remains source of truth (online event, focus, manual Retry) |
| **Periodic Background Sync** | Optional later for reference cache refresh—not for compliance submits |
| **Constraints** | Browser/OS may defer or deny; never require BG Sync for correctness |
| **SW duties** | On sync event → message client or run sync engine bridge to flush outbox |

Outbox lives in IndexedDB via `packages/pwa-sync`; SW must not own domain rules.

---

## 6. Push Notifications

| Aspect | Design |
| --- | --- |
| **Permission** | Request after context (e.g. first Critical assignment)—not on first load |
| **Subscription** | PushSubscription → Notifications module device registry |
| **Payload** | Title, body, deep link, `notification_id`; minimize PII |
| **SW handler** | `push` → show notification; `notificationclick` → focus/open route |
| **Priority** | High/Critical may use urgent presentation where platform allows |
| **Prefs** | User channel prefs + quiet hours ([Notification Architecture](./NOTIFICATION_ARCHITECTURE.md)) |
| **Auth** | Click opens app; session refresh if needed before private routes |

Push is complementary to In-App; not a work SoR.

---

## 7. Camera

| Aspect | Design |
| --- | --- |
| **API** | `getUserMedia` / file input `capture="environment"` fallback |
| **Use cases** | FLHA photos, inspection evidence, deficiency pics, signature stroke alternative (image) |
| **Permission** | Transient; explain purpose; degrade to gallery picker |
| **UX** | Full-bleed capture in field flows; confirm before queue |
| **Security** | No camera streams uploaded without user intent; AV scan server-side after upload |
| **Performance** | Prefer reasonable resolution caps before enqueue |

---

## 8. Photo Upload

```text
Capture → persist blob in IndexedDB media store (before leaving camera UI)
  → enqueue upload intent chain (CreateFileUploadIntent → R2 PUT → Complete → Bind)
  → thumbnails local immediate
  → server AV → Available | Quarantined
```

| Rule | Detail |
| --- | --- |
| Crash safety | Blob on disk/IDB before navigate away |
| Offline | Queue until online; Sync Pill includes media |
| Limits | Client UX size/type; server authoritative |
| Quarantine | Show attachment error; allow retake |
| EXIF | Strip/policy for GPS in image per privacy settings when required |

---

## 9. GPS

| Aspect | Design |
| --- | --- |
| **API** | Geolocation (`getCurrentPosition` / watch sparingly) |
| **Use** | Optional stamp on FLHA/inspection/incident (“where captured”); not primary AuthZ |
| **Permission** | Explicit; deny → continue without coords + label “location unavailable” |
| **Accuracy** | Record accuracy meters; never fake precision |
| **Privacy** | Tenant/policy: retain coords with evidence retention class; minimize continuous tracking—**no background GPS stalking** |
| **Offline** | Cache last fix only if fresh enough; else omit |
| **Maps** | Deep-link out optional; in-app map not required for MVP |

GPS is evidence context, not a time clock SoR unless product later defines attendance integration.

---

## 10. QR Scanner

| Aspect | Design |
| --- | --- |
| **API** | `BarcodeDetector` where available; else camera frame + wasm/js fallback; manual code entry fallback |
| **Use** | Equipment asset tags, document/sign QR sessions, Place check-in targets |
| **Flow** | Scan → resolve session/API → navigate to guest sign or asset pre-use |
| **Offline** | Resolve only if session/package pre-cached; else queue “pending scan” or require online |
| **Security** | HTTPS only; validate payload against Proven-signed/known prefixes; no arbitrary URL open without allowlist |

Align with [Digital Signatures](./DIGITAL_SIGNATURES_ARCHITECTURE.md) QR sessions.

---

## 11. Digital Signatures (PWA)

| Mode | PWA behavior |
| --- | --- |
| **Authenticated seal** | Canvas/stylus pad in wizard; optional offline per policy |
| **Guest seal** | Separate guest route; generally **online** |
| **QR sign** | Scanner → guest/auth sign UI |
| **Pending seal** | Local capture queued; UI not “Proven” until ACK |
| **Reminders** | Push + in-app for pending slots |

Stroke/image → media store → seal API. Details: Digital Signatures + Offline Sync architectures.

---

## 12. Caching

### 12.1 Layers

| Layer | Content | Strategy |
| --- | --- | --- |
| **App shell** | JS/CSS/fonts/chrome | Precache on SW install; versioned |
| **Static assets** | Icons, manifest | Cache-first |
| **API JSON** | Allowlisted GETs (checklists, hazard libs, open drafts meta) | Network-first or stale-while-revalidate with **staleness labels** |
| **Authenticated JSON** | Most private data | Prefer network; soft cache with short TTL; `Cache-Control` respect |
| **Media** | Local IDB blobs pending upload | Not SW Cache API as SoR |
| **Pages** | App Router | Shell offline; data from IDB drafts/outbox |

### 12.2 Rules

- Do not precache entire API corpus.  
- Never cache secrets.  
- AuthZ not inferred from cache.  
- Search/admin: online-first.

---

## 13. Conflict Resolution

| Situation | PWA handling |
| --- | --- |
| Server sealed/voided/closed | Server wins; local → conflict; Compare UI |
| Dual-device draft | Choose side / manual merge (v1) |
| Version/etag mismatch | Pause aggregate outbox |
| Attachment bind missing parent | Wait dependency / dead-letter |
| Seal after void | Reject; surface error |

Sync Center: Retry, Resolve, Discard local (confirmed). No silent overwrite of sealed evidence.

---

## 14. Updates

| Mechanism | Design |
| --- | --- |
| **SW update** | On `waiting` worker → prompt “Update Proven” |
| **Apply** | `skipWaiting` + reload when user accepts; try drain outbox first |
| **Manifest** | Bump when icons/name change |
| **App Router** | Deploy immutable hashed assets; avoid breaking old SW mid-flight |
| **Migrations** | `packages/pwa-sync` schema version in IDB; migrate on load |
| **Forced update** | Rare; for critical security—block app until refresh after message |

Never wipe outbox on routine update without migration.

---

## 15. Performance

### 15.1 Targets (architectural)

| Metric | Intent |
| --- | --- |
| **LCP** | Fast shell on 4G; defer heavy wizards |
| **INP** | Snappy tap on My Actions |
| **Offline start** | Shell + drafts usable without network |
| **Upload** | Non-blocking; background-friendly |

### 15.2 Techniques

- Split field wizards via dynamic import  
- Virtualize long queues  
- Image downscale before IDB/upload  
- Precache shell only  
- Avoid giant client Zod on first paint  
- Prefer RSC for chrome where auth allows; hydrate field flows  
- Measure field vs admin Web Vitals separately  

### 15.3 Device constraints

- Storage quota warnings before capture  
- Battery: avoid continuous GPS/watch  
- Camera: single stream; release on blur  

---

## 16. Service Worker Responsibilities

| Event | Behavior |
| --- | --- |
| `install` | Precache shell |
| `activate` | Claim clients; delete old caches |
| `fetch` | Routing strategies per §12 |
| `sync` | Background outbox flush trigger |
| `push` / `notificationclick` | Show/open |
| `message` | Client ↔ SW commands (skip waiting, sync now) |

SW **must not**: call Temporal, invent AuthZ, write module business tables.

---

## 17. Security & Privacy

| Topic | Control |
| --- | --- |
| Origin | HTTPS only |
| Storage | Clear on logout per policy |
| Camera/GPS/mic | Permission + purpose strings |
| Guest tokens | Isolated routes; short TTL |
| CSP | As frontend security architecture |
| Push endpoints | User-bound; revoke on logout |

---

## 18. Capability Detection & Progressive Enhancement

```text
If feature missing → fallback
  Camera → file picker
  BarcodeDetector → manual entry / secondary lib
  Background Sync → foreground-only drain
  Push → in-app + email only
  Geolocation → skip stamp
```

Never hard-fail install on missing BG Sync.

---

## 19. Folder / Package Touchpoints

| Area | Location |
| --- | --- |
| Manifest / icons | `apps/web/public` |
| SW registration | `apps/web` PWA config |
| Sync engine | `packages/pwa-sync` |
| Sync UI | `features/offline`, Sync Pill |
| Capture UI | `features/safety|equipment`, `components/files` |
| QR | `features/signatures` / equipment |
| Push register | `features/notifications` + SW |

See [Frontend Folder Structure](./FRONTEND_FOLDER_STRUCTURE.md).

---

## 20. Testing

| Layer | Focus |
| --- | --- |
| Unit | Outbox, cache routing mocks |
| E2E | Airplane mode FLHA + photo + sync |
| Manual | Install Android/iOS; push click; QR; GPS deny path |
| Chaos | Kill mid-upload; SW update with pending outbox |
| A11y | Capture flows, Sync Pill live regions |

---

## 21. Rollout Phases

| Phase | Scope |
| --- | --- |
| **P0** | Manifest install, shell SW, offline drafts/outbox, camera upload queue, Sync Pill |
| **P1** | Background Sync hooks, push, GPS stamps, QR scan |
| **P2** | Offline authenticated signatures |
| **P3** | Periodic sync, richer install UX, advanced cache tuning |

---

## 22. Success Criteria

1. Workers can install Proven and complete allowlisted field flows offline with photos.  
2. Background Sync helps but is not required for correctness.  
3. Push deep-links into the right action without leaking PII.  
4. Camera, GPS, and QR degrade gracefully when denied/unavailable.  
5. Signatures never lie about sealed state offline.  
6. Caching and updates preserve outbox integrity.  
7. Performance keeps My Actions usable on mid-tier mobile networks.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | PWA Architecture | Install, offline, device APIs, sync |

---

*End of Progressive Web App Architecture*
