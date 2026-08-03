# Proven — Frontend Folder Structure

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Frontend Folder Structure Design |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Lead Frontend Engineering |
| **Audience** | Frontend Engineering, Design, Mobile/PWA |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Frontend Architecture](./FRONTEND_ARCHITECTURE.md), [UX Architecture](../ux/UX_ARCHITECTURE.md), [Design System](../design/DESIGN_SYSTEM.md), [Offline Sync](./OFFLINE_SYNC_ARCHITECTURE.md), [Repository Plan](./REPOSITORY_PLAN.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document **designs and documents every frontend folder** for Proven’s Next.js App Router client: shared vs feature components, hooks, stores, utilities, services, layouts, pages, authentication, forms, tables, charts, offline sync, notifications, file upload, error handling, loading, and accessibility.

**Hard rules**

1. **`app/` is routing only** — thin pages; logic lives in `features/` and `lib/`.  
2. **No business invariants in React** — Zod/RHF are UX shape only.  
3. **Primitives in `packages/ui`** — product composites in `features/` or `components/`.  
4. **Offline primitives in `packages/pwa-sync`** — feature wiring in `features/offline`.  

**No application code — folder documentation only.**

---

## 2. Package Placement

| Path | Role |
| --- | --- |
| `apps/web` | Next.js application (this document’s focus) |
| `packages/ui` | Design-system primitives (Button, Input, Dialog, …) |
| `packages/api-client` | Typed `/api/v1` HTTP client |
| `packages/pwa-sync` | Outbox, drafts store, media queue, sync engine |
| `packages/typescript-config` | Shared TS config |
| `packages/eslint-config` | Shared ESLint config |

---

## 3. Complete Tree (`apps/web`)

```text
apps/web/
├── app/                          # App Router (routes + layouts only)
├── components/                   # App-wide shared composites (not domain features)
├── features/                     # Domain/feature modules
├── hooks/                        # App-wide hooks
├── stores/                       # Client UI stores (non-server state)
├── lib/                          # Utilities, services, auth, forms helpers
├── styles/                       # Global CSS / tokens entry
├── public/                       # Static + PWA assets
├── messages/                     # i18n message catalogs (optional)
├── tests/                        # Web-unit/component colocated or here
├── next.config.ts
├── middleware.ts                 # Auth gate / headers (edge)
├── package.json
└── tsconfig.json
```

Each top-level folder is documented below.

---

## 4. `app/` — App Router

**Purpose:** URL structure, nested layouts, loading/error boundaries, and metadata. Pages import feature screens; they do not own domain logic.

```text
app/
├── layout.tsx                    # Root: html, fonts, theme provider shell
├── template.tsx                  # Optional transition wrapper
├── not-found.tsx                 # Global 404
├── error.tsx                     # Global error boundary UI
├── global-error.tsx              # Root crash UI
├── loading.tsx                   # Optional root loading (sparingly)
├── (marketing)/                  # Public marketing (optional)
│   ├── layout.tsx
│   └── page.tsx
├── (auth)/                       # Unauthenticated / guest flows
│   ├── layout.tsx                # Minimal chrome
│   ├── login/
│   │   ├── page.tsx
│   │   └── loading.tsx
│   ├── oauth/callback/
│   │   └── page.tsx
│   ├── logout/
│   │   └── page.tsx
│   └── guest/
│       └── sign/
│           └── [token]/
│               ├── page.tsx
│               ├── loading.tsx
│               └── error.tsx
└── (app)/                        # Authenticated product shell
    ├── layout.tsx                # AppShell: rail / mobile tabs / providers
    ├── loading.tsx               # Shell-level route loading
    ├── error.tsx                 # Authenticated segment errors
    ├── page.tsx                  # Role-aware home redirect
    ├── actions/                  # My Actions (mobile home)
    │   ├── page.tsx
    │   └── loading.tsx
    ├── command-center/           # Desktop home
    │   └── page.tsx
    ├── find/                     # Mobile search hub
    │   └── page.tsx
    ├── notifications/
    │   ├── page.tsx
    │   └── loading.tsx
    ├── activity/                 # Cross-project activity feed
    │   └── page.tsx
    ├── projects/
    │   ├── page.tsx              # Project list
    │   ├── loading.tsx
    │   └── [projectId]/
    │       ├── layout.tsx        # Place header + subnav
    │       ├── loading.tsx
    │       ├── page.tsx          # Place overview
    │       ├── actions/
    │       ├── people/
    │       ├── safety/
    │       │   ├── page.tsx
    │       │   ├── flha/
    │       │   │   ├── new/page.tsx
    │       │   │   └── [activityId]/page.tsx
    │       │   └── …
    │       ├── equipment/
    │       ├── documents/
    │       ├── training/
    │       └── activity/
    ├── people/
    ├── safety/                   # Tenant-scoped safety hubs
    ├── equipment/
    ├── documents/
    ├── training/
    ├── signatures/
    ├── analytics/
    ├── cor/
    ├── admin/                    # Desktop admin console
    │   ├── layout.tsx            # Admin chrome (warn on small viewports)
    │   └── …/
    ├── settings/
    │   ├── page.tsx
    │   ├── profile/
    │   ├── appearance/
    │   ├── notifications/
    │   └── security/             # MFA, sessions
    └── sync/                     # Sync Center (offline queue UI)
        └── page.tsx
```

### 4.1 Folder notes (`app/`)

| Folder / file | Owns | Must not |
| --- | --- | --- |
| **Route groups `(auth)` `(app)`** | Layout chrome split without URL segment | Business rules |
| **`layout.tsx`** | Shell composition, provider nesting | Data fetching of entire product |
| **`page.tsx`** | Compose feature screen + params | Fat logic, SQL metaphors |
| **`loading.tsx`** | Suspense fallback for segment | Fake “sealed” success states |
| **`error.tsx`** | Segment error UI + reset | Swallow auth errors silently |
| **`[projectId]/`** | Place-scoped IA | Client-trusted AuthZ |

---

## 5. `components/` — Shared App Composites

**Purpose:** Reusable UI used across features but **not** design-system primitives (those live in `packages/ui`) and **not** domain-specific widgets (those live in `features/*/components`).

```text
components/
├── layout/                       # Shell chrome
│   ├── AppShell
│   ├── DesktopRail
│   ├── MobileTabBar
│   ├── PlaceHeader
│   ├── PlaceSubnav
│   ├── PageHeader
│   ├── CommandPalette
│   └── AdminShell
├── navigation/
│   ├── NavItem
│   ├── ProjectSwitcher
│   └── Breadcrumbs
├── feedback/
│   ├── ToastViewport
│   ├── SyncPill
│   ├── EmptyState
│   ├── InlineAlert
│   └── ConfirmDialog
├── provenance/                   # Proof language
│   ├── ProofSeal
│   ├── EvidencePanel
│   └── StatusChip
├── forms/                        # Shared form chrome (not domain fields)
│   ├── Form
│   ├── FormField
│   ├── FormSection
│   ├── WizardShell
│   └── SubmitBar
├── tables/
│   ├── DataTable
│   ├── DataTableToolbar
│   ├── DataTablePagination
│   └── ColumnHeader
├── charts/
│   ├── ChartContainer
│   ├── KpiTile
│   ├── TrendLine
│   └── Sparkline
├── files/
│   ├── FileDropzone
│   ├── UploadProgress
│   └── AttachmentThumb
├── loading/
│   ├── PageSkeleton
│   ├── TableSkeleton
│   ├── CardSkeleton
│   └── Spinner
├── errors/
│   ├── ErrorState
│   ├── RouteError
│   └── CorrelationDetails
└── a11y/
    ├── SkipLink
    ├── LiveRegion
    └── VisuallyHidden
```

### 5.1 Folder documentation

| Folder | Purpose |
| --- | --- |
| **`layout/`** | App chrome: rail, tabs, Place header—matches UX shells |
| **`navigation/`** | Cross-route navigation controls |
| **`feedback/`** | Toasts, Sync Pill, empties, confirms |
| **`provenance/`** | Seal/proof visual language (never invent sealed state client-side) |
| **`forms/`** | RHF-friendly field chrome and wizard frame |
| **`tables/`** | Shared DataTable pattern (TanStack Table) |
| **`charts/`** | Analytics-facing chart wrappers (theme-aware); no ad-hoc KPI math |
| **`files/`** | Upload UX primitives wired to upload services |
| **`loading/`** | Skeletons and spinners for Suspense/query pending |
| **`errors/`** | Consistent error panels with optional correlation id |
| **`a11y/`** | Skip links, live regions for sync/toasts, SR-only text |

---

## 6. `features/` — Feature Modules

**Purpose:** Product capabilities by domain. Each feature owns screens, feature components, feature hooks, schemas, and API query/mutation wiring.

```text
features/
├── actions/                      # My Actions queue UX
├── projects/
├── people/
├── safety/
├── equipment/
├── documents/
├── training/
├── signatures/
├── notifications/
├── analytics/
├── cor/
├── admin/
├── search/                       # Find / command palette results
├── settings/
├── auth/                         # Login, SSO, guest sign UX
└── offline/                      # Sync Center, draft banners, queue UX
```

### 6.1 Canonical feature interior

```text
features/<name>/
├── api/                          # Query options, mutation hooks (TanStack Query)
├── components/                   # Feature-only composites
├── hooks/                        # Feature-local hooks
├── schemas/                      # Zod UX schemas
├── screens/                      # Full screens imported by app/*/page.tsx
├── model/                        # View-model mappers (display only)
├── constants.ts
└── index.ts                      # Public exports for app routes
```

### 6.2 Feature folder documentation

| Feature | Owns |
| --- | --- |
| **`actions/`** | Unified action queue items, filters, deep links |
| **`projects/`** | Project list, Place overview tiles (composition only) |
| **`people/`** | Worker directory, profile panels |
| **`safety/`** | FLHA/toolbox/inspection wizards, CA lists, incident UX |
| **`equipment/`** | Asset list, pre-use flows, readiness display |
| **`documents/`** | Doc library, version viewer, ack flows |
| **`training/`** | Assignments, completions, gaps UI |
| **`signatures/`** | Package progress, seal pad (authenticated) |
| **`notifications/`** | Inbox list, preference screens wiring |
| **`analytics/`** | Dashboard widgets using shared `components/charts` |
| **`cor/`** | Readiness, gaps, engagement UI |
| **`admin/`** | Branding, API keys, builders, admin tables |
| **`search/`** | Global/mobile Find result groups |
| **`settings/`** | Profile, theme, notification prefs, security UI |
| **`auth/`** | Login form, SSO buttons, guest sign wizard |
| **`offline/`** | Sync Center, conflict compare, pending banners |

**Must not:** call Temporal/NATS; embed AuthZ decisions; store sealed proof without server ACK.

---

## 7. `hooks/` — App-Wide Hooks

**Purpose:** Hooks reused across multiple features. Feature-specific hooks stay under `features/*/hooks`.

```text
hooks/
├── useMediaQuery.ts              # Breakpoints / mobile detection
├── useDebouncedValue.ts
├── useDisclosure.ts              # Dialog/sheet open state
├── usePagination.ts
├── useProjectContext.ts          # Active Place from route/layout
├── usePermissionUX.ts            # Hide/disable controls (non-authoritative)
├── useStickyParams.ts            # URL filter sync helpers
├── useOnlineStatus.ts
├── useAnnounce.ts                # Live region announcements
└── useTheme.ts
```

| Hook area | Notes |
| --- | --- |
| **Permissions UX** | Reflects server-provided capability flags; never sole security |
| **Online status** | Feeds Sync Pill / offline banners |
| **Announce** | Accessibility for sync/toast outcomes |

---

## 8. `stores/` — Client UI Stores

**Purpose:** Ephemeral or cross-route **UI** state that is not server cache. Prefer URL + TanStack Query; stores are the exception.

```text
stores/
├── ui-store.ts                   # Rail collapsed, command palette open
├── sync-ui-store.ts              # Sync Pill expanded, last conflict focus
├── wizard-ui-store.ts            # Optional cross-mount wizard chrome only
└── README.md                     # When to use a store vs Query vs URL
```

| Allowed | Disallowed |
| --- | --- |
| Chrome toggles, draft **UI** step index if not in URL | AuthZ grants, tenant config as SoR, compliance entity truth |
| Sync Center filter prefs (optional) | Duplicate of React Query cache |

Server state → **TanStack Query**. Offline mutations → **`packages/pwa-sync`**.

---

## 9. `lib/` — Utilities, Services, Cross-Cutting

**Purpose:** Non-React helpers, service wrappers, and infrastructure for the web app.

```text
lib/
├── auth/
│   ├── session.ts                # Read session/cookie helpers
│   ├── guards.ts                 # Redirect helpers for RSC/middleware collab
│   ├── guest-token.ts            # Guest route token handling
│   └── mfa.ts                    # Step-up UX helpers
├── api/
│   ├── client.ts                 # Wraps packages/api-client with credentials
│   ├── query-client.ts           # TanStack Query client factory
│   ├── query-keys.ts             # Global key factory
│   └── errors.ts                 # Problem-details → UI error model
├── services/
│   ├── upload-service.ts         # Intent → presign → PUT → complete
│   ├── notification-service.ts   # Mark read, preference updates
│   ├── search-service.ts         # /search wrappers
│   └── analytics-service.ts      # Dashboard query wrappers
├── forms/
│   ├── createForm.ts             # RHF + Zod resolver helpers
│   └── validators.ts             # Shared UX validators (email format, …)
├── offline/
│   ├── bridge.ts                 # Adapts pwa-sync ↔ feature mutations
│   └── allowlist.ts              # Mirrors server allowlist for UX gating
├── files/
│   ├── checksum.ts
│   └── accept.ts                 # Client accept maps (UX only)
├── charts/
│   └── formatters.ts             # Number/date format for tiles
├── a11y/
│   ├── focus.ts
│   └── keys.ts
├── theme/
│   └── themes.ts                 # Day / Dark / Site HC
├── i18n/
│   └── index.ts
├── utils/
│   ├── cn.ts                     # className merge
│   ├── date.ts
│   ├── id.ts
│   └── url.ts
└── constants/
    ├── routes.ts
    └── storage-keys.ts
```

### 9.1 Folder documentation

| Folder | Purpose |
| --- | --- |
| **`auth/`** | Session, guest, MFA **UX/helpers**; server AuthZ remains authoritative |
| **`api/`** | HTTP + React Query plumbing |
| **`services/`** | Use-case-shaped client services (orchestration of API calls only) |
| **`forms/`** | Shared form factory utilities |
| **`offline/`** | Bridge to `packages/pwa-sync`; allowlist for disabling online-only CTAs |
| **`files/`** | Checksum/accept helpers for upload UX |
| **`charts/`** | Display formatters for analytics |
| **`a11y/`** | Focus trap helpers, keyboard utils |
| **`theme/`** | Theme tokens wiring |
| **`i18n/`** | Locale loading |
| **`utils/`** | Pure helpers |
| **`constants/`** | Route paths, storage key names |

**Services vs features/api:** `lib/services` for cross-feature shared call sequences; `features/*/api` for domain hooks.

---

## 10. Layouts (Where They Live)

| Layout | Location | Responsibility |
| --- | --- | --- |
| Root | `app/layout.tsx` | Fonts, theme, global providers |
| Auth | `app/(auth)/layout.tsx` | Minimal branded chrome |
| App shell | `app/(app)/layout.tsx` | Rail / tabs, Sync Pill host, toasts |
| Place | `app/(app)/projects/[projectId]/layout.tsx` | Place header + subnav |
| Admin | `app/(app)/admin/layout.tsx` | Admin navigation; mobile warning |
| Chrome pieces | `components/layout/*` | Presentational shell parts |

Providers (QueryClient, theme, sync engine) nest in app layouts—not in every page.

---

## 11. Pages

| Concern | Rule |
| --- | --- |
| **Definition** | `app/**/page.tsx` files |
| **Thickness** | Parse params → render `features/.../screens/X` |
| **Data** | Prefer screen-level queries; RSC for static shells where auth allows |
| **Naming** | Mirror UX IA: actions, command-center, Place tabs |

Documented route map: [Frontend Architecture §6](./FRONTEND_ARCHITECTURE.md) + tree in §4 above.

---

## 12. Authentication Folders

| Path | Role |
| --- | --- |
| `app/(auth)/**` | Login, OAuth callback, logout, guest sign routes |
| `features/auth/**` | Login form, SSO controls, guest seal UI |
| `lib/auth/**` | Session helpers, redirects |
| `middleware.ts` | Cookie/session gate for `(app)` routes; guest exception paths |
| `app/(app)/settings/security/**` | MFA/sessions management UI |

Guest sign stays isolated from full app privileges.

---

## 13. Forms

| Path | Role |
| --- | --- |
| `components/forms/**` | Shared FormField, WizardShell, SubmitBar |
| `features/*/schemas/**` | Zod schemas per feature |
| `features/*/components/**` | Domain fields (hazard picker, checklist) |
| `lib/forms/**` | RHF helpers |
| Draft persistence | Via `packages/pwa-sync` + `features/offline` |

Wizards (FLHA, inspection) live under `features/safety` / `features/equipment` screens—not under `app/`.

---

## 14. Tables

| Path | Role |
| --- | --- |
| `components/tables/**` | DataTable shell, toolbar, pagination |
| `features/*/components/**` | Column defs, row actions per domain |
| Filters | Sync to URL (`hooks/useStickyParams`) |

No domain filtering AuthZ in the table—server list endpoints scope results.

---

## 15. Charts

| Path | Role |
| --- | --- |
| `components/charts/**` | Themed containers, KPI tile, trend line |
| `features/analytics/**` | Dashboard screens composing charts + query hooks |
| `lib/charts/**` | Formatters only |

KPI math stays on Analytics API / warehouse—not recomputed as SoR in the client.

---

## 16. Offline Sync

| Path | Role |
| --- | --- |
| `packages/pwa-sync` | Outbox, drafts, media, sync engine (package) |
| `features/offline/**` | Sync Center screen, conflict UI, banners |
| `lib/offline/**` | Bridge + UX allowlist |
| `components/feedback/SyncPill` | Global status chrome |
| `app/(app)/sync/page.tsx` | Route entry for Sync Center |
| `hooks/useOnlineStatus` | Connectivity |
| SW / manifest | `public/` + Next PWA config (see Frontend Architecture) |

Align with [OFFLINE_SYNC_ARCHITECTURE.md](./OFFLINE_SYNC_ARCHITECTURE.md).

---

## 17. Notifications (UI)

| Path | Role |
| --- | --- |
| `app/(app)/notifications/**` | Inbox route |
| `features/notifications/**` | List, row, mark-read mutations, prefs |
| `lib/services/notification-service.ts` | Shared API helpers |
| `app/(app)/settings/notifications/**` | Preference pages |
| Toasts | `components/feedback` + feature triggers |

Optimistic mark-read allowed; server remains authoritative.

---

## 18. File Upload

| Path | Role |
| --- | --- |
| `components/files/**` | Dropzone, progress, thumbs |
| `lib/services/upload-service.ts` | Intent → R2 PUT → complete |
| `lib/files/**` | Accept lists, checksum UX |
| Feature usage | `features/*/components` attach uploads to activities/docs |
| Offline photos | `packages/pwa-sync` media store + offline bridge |

Client type/size checks are UX only; AV quarantine is server-side.

---

## 19. Error Handling

| Path | Role |
| --- | --- |
| `app/error.tsx`, `app/global-error.tsx` | Route boundaries |
| `app/(app)/error.tsx`, segment `error.tsx` | Scoped recovery |
| `components/errors/**` | ErrorState, correlation details |
| `lib/api/errors.ts` | Map problem+json → UI model |
| Feature empty/error | Inline in screens |

Never show stack traces or secrets. Sync failures actionable (Retry) via Sync Center.

---

## 20. Loading

| Path | Role |
| --- | --- |
| `app/**/loading.tsx` | Segment Suspense fallbacks |
| `components/loading/**` | Skeletons (page, table, card), spinner |
| Query `isPending` / `isFetching` | Feature screens choose skeleton vs inline |

Deterministic progress for uploads/sync when count known (Design System motion).

---

## 21. Accessibility

| Path | Role |
| --- | --- |
| `components/a11y/**` | SkipLink, LiveRegion, VisuallyHidden |
| `lib/a11y/**` | Focus management helpers |
| `hooks/useAnnounce` | Assertive/polite announcements (sync, toasts) |
| Feature forms/tables | Label association, keyboard ops, ≥44px targets |

Target **WCAG 2.2 AA**. Themes include Site High Contrast. Honor `prefers-reduced-motion`.

---

## 22. `styles/` · `public/` · Other Roots

### 22.1 `styles/`

```text
styles/
└── globals.css                   # Tailwind layers + CSS variables (brand, status, surfaces)
```

Design tokens per [DESIGN_SYSTEM.md](../design/DESIGN_SYSTEM.md); admin branding overrides constrained variables only.

### 22.2 `public/`

```text
public/
├── icons/                        # PWA icons
├── manifest.webmanifest
└── robots.txt                    # as needed
```

### 22.3 `messages/` (optional)

i18n catalogs keyed by locale; loaded via `lib/i18n`.

### 22.4 `middleware.ts`

Edge auth routing: protect `(app)`; allow `(auth)` and guest sign; security headers collaboration.

### 22.5 `tests/`

Component/feature tests if not colocated; Playwright e2e lives primarily in repo `tests/e2e` (monorepo), not necessarily under `apps/web`.

---

## 23. `packages/ui` (Shared Components — Primitives)

Documented here for boundary clarity:

```text
packages/ui/
├── src/
│   ├── button/
│   ├── input/
│   ├── select/
│   ├── dialog/
│   ├── sheet/
│   ├── tabs/
│   ├── checkbox/
│   ├── tooltip/
│   └── …
└── package.json                  # @proven/ui
```

**Owns:** accessible primitives + tokens consumption.  
**Must not:** feature screens, API calls, offline outbox, AuthZ.

---

## 24. Import Direction

```text
app/*  →  features/*  →  components/*  →  packages/ui
                ↓              ↓
            lib/* , hooks/* , stores/*
                ↓
         packages/api-client , packages/pwa-sync
```

**Forbidden:** `packages/ui` → `features`; `components` → `app`; features importing each other’s internals (prefer public `features/x` exports or shared `components`).

---

## 25. Ownership

| Area | Owners |
| --- | --- |
| `app/`, shell `components/layout` | Frontend platform |
| `features/safety`, `equipment`, field wizards | Frontend + domain UX |
| `features/offline`, `packages/pwa-sync` | Frontend mobile/offline |
| `features/analytics`, `components/charts` | Frontend + analytics |
| `features/auth`, `lib/auth` | Frontend + security review |
| `packages/ui` | Design system / frontend |

Align with CODEOWNERS `/apps/web/`, `/packages/`.

---

## 26. Success Criteria

1. Every requirement area maps to a documented folder.  
2. New routes add thin `page.tsx` + feature screen—not new logic in `app/`.  
3. Shared vs feature vs primitive boundaries are obvious.  
4. Offline, upload, notifications, charts, tables, forms each have a clear home.  
5. Accessibility and loading/error patterns are shared, not one-off.  
6. No folder encourages putting domain invariants in the client.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Lead Frontend Engineering | Complete folder structure documentation |

---

*End of Frontend Folder Structure*
