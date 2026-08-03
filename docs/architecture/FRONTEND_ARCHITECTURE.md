# Proven — Frontend Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Frontend Architecture |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Principal Frontend Architecture |
| **Audience** | Frontend Engineering, Design, Mobile/PWA |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [UX Architecture](../ux/UX_ARCHITECTURE.md), [Repository Plan](./REPOSITORY_PLAN.md), [System Architecture](./SYSTEM_ARCHITECTURE.md), [Rust Backend](./RUST_BACKEND_ARCHITECTURE.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document defines the **complete frontend architecture** for Proven’s web/PWA client.

Stack: **Next.js App Router**, **TypeScript**, **Tailwind CSS**, **shadcn/ui**, **TanStack Query**, **React Hook Form**, **Zod**, **PWA** with **offline support**.

**Documentation only — no application code.**

### 1.1 Non-Negotiable Rules

1. **No business invariants in React** — UI validates shape/UX only; server is authoritative ([AGENTS.md](../../AGENTS.md)).  
2. Mobile-first for workers; desktop-first for supervisors/admins ([UX](../ux/UX_ARCHITECTURE.md)).  
3. Proof/seal states are first-class in UI language.  
4. Offline is a normal path for allowlisted field flows.  
5. Prefer calm composition over dashboard card grids.

---

## 2. Application Placement

| Path | Role |
| --- | --- |
| `apps/web` | Next.js application (Vercel) |
| `packages/ui` | Shared design-system primitives (shadcn-based) |
| `packages/api-client` | Typed HTTP client for `/api/v1` |
| `packages/pwa-sync` | Offline mutation queue primitives |
| `packages/typescript-config` / `eslint-config` | Shared tooling |

---

## 3. Folder Structure

```text
apps/web/
├── app/                              # Next.js App Router
│   ├── (marketing)/                  # optional public marketing
│   ├── (auth)/                       # login, SSO callback, guest sign
│   │   ├── login/
│   │   ├── guest/sign/[token]/
│   │   └── layout.tsx
│   ├── (app)/                        # authenticated shell
│   │   ├── layout.tsx                # rail/tabs, providers
│   │   ├── page.tsx                  # role-aware home redirect
│   │   ├── actions/                  # My Actions
│   │   ├── command-center/           # desktop home
│   │   ├── activity/
│   │   ├── projects/
│   │   │   ├── page.tsx
│   │   │   └── [projectId]/
│   │   │       ├── layout.tsx        # Place header + subnav
│   │   │       ├── page.tsx          # Overview
│   │   │       ├── actions/
│   │   │       ├── people/
│   │   │       ├── safety/
│   │   │       ├── equipment/
│   │   │       ├── documents/
│   │   │       ├── training/
│   │   │       └── activity/
│   │   ├── people/
│   │   ├── safety/
│   │   ├── equipment/
│   │   ├── documents/
│   │   ├── training/
│   │   ├── signatures/               # pending packages (optional hub)
│   │   ├── analytics/
│   │   ├── cor/
│   │   ├── notifications/
│   │   ├── admin/                    # desktop admin console
│   │   ├── settings/                 # profile, theme, prefs
│   │   └── find/                     # mobile search hub
│   ├── layout.tsx                    # root html, fonts, theme
│   ├── not-found.tsx
│   └── error.tsx
├── components/                       # app-specific composites
│   ├── layout/                       # AppShell, Rail, MobileTabBar, PlaceHeader
│   ├── navigation/
│   ├── feedback/                     # toasts, sync pill, empty states
│   └── provenance/                   # proof seal, evidence panel
├── features/                         # feature modules by domain UX
│   ├── actions/
│   ├── projects/
│   ├── people/
│   ├── safety/
│   ├── equipment/
│   ├── documents/
│   ├── training/
│   ├── signatures/
│   ├── notifications/
│   ├── analytics/
│   ├── cor/
│   ├── admin/
│   └── offline/
├── lib/
│   ├── auth/                         # session helpers, guards
│   ├── api/                          # query keys, fetch wrappers
│   ├── forms/                        # RHF + Zod helpers
│   ├── i18n/                         # readiness
│   ├── theme/
│   └── utils/
├── hooks/
├── styles/
│   └── globals.css                   # Tailwind + CSS variables
├── public/
│   ├── icons/                        # PWA icons
│   ├── manifest.webmanifest
│   └── sw.js                         # or Serwist/Workbox entry
├── tests/
├── next.config.ts
├── tailwind.config.ts
├── tsconfig.json
└── package.json
```

### 3.1 Organization Rules

- **`app/`** — routing and layouts only; keep page files thin.  
- **`features/*`** — screens, feature hooks, feature-local components.  
- **`components/`** — shared app chrome and cross-feature UI.  
- **`packages/ui`** — primitives (Button, Input, Dialog)—not product features.  
- Colocate feature tests next to features when practical.

---

## 4. Components

### 4.1 Layers

| Layer | Location | Examples |
| --- | --- | --- |
| **Primitives** | `packages/ui` | Button, Input, Select, Dialog, Sheet, Tabs, Checkbox |
| **Patterns** | `apps/web/components` + `packages/ui` | DataTable shell, FormField, PageHeader, StatusChip |
| **Product composites** | `features/*` | SafetyActivityWizard, EquipmentReadinessCard, QueueItem |
| **Chrome** | `components/layout` | Rail, TabBar, PlaceHeader, CommandPalette |

### 4.2 Proven-Specific Patterns ([UX](../ux/UX_ARCHITECTURE.md))

| Pattern | Purpose |
| --- | --- |
| **Queue Item** | My Actions / Needs you |
| **Proof Seal** | Sealed / partial / pending signature state |
| **Sync Pill** | Offline pending/syncing/failed |
| **Eligibility Pill** | Ready / Gap / Blocked (display only) |
| **Evidence Panel** | Who/when/version |
| **Place Header** | Project context + subnav |
| **Review Drawer** | Desktop side panel for review/sign |
| **Empty State** | Human copy + one CTA |

### 4.3 Card Policy

Default **no decorative cards**. Use card chrome only when the surface is an interactive work item (queue item) or required by dense admin tables as row containers—not for marketing-like hero blocks inside the app.

### 4.4 Composition Rules

- Prefer server components for static shells where auth allows.  
- Interactive islands (`use client`) at feature boundaries (forms, tables, wizards).  
- Do not sprinkle `"use client"` at the root layout unnecessarily.

---

## 5. Layouts

### 5.1 Root Layout

- Fonts (distinctive pair per UX—not Inter/Roboto defaults)  
- Theme provider (Day / Dark / Site High Contrast)  
- Global CSS variables  
- Skip link  

### 5.2 Auth Layout

- Minimal chrome  
- Guest signing: single-purpose, no OS nav  

### 5.3 App Layout (Authenticated)

**Desktop**

- Left rail (Home, Actions, Projects, modules, Admin)  
- Top bar: search, project switcher, notifications, account  
- Main canvas  

**Mobile**

- Bottom tabs: Today | Project | Find | Activity | Menu  
- Sticky project context chip  
- No hamburger-as-home  

### 5.4 Place Layout

Nested under `/projects/[projectId]/*`:

- Place header (name, status, prime)  
- Horizontal subnav: Overview · Actions · People · Safety · Equipment · Docs · Training · Activity  

### 5.5 Admin Layout

Desktop-only primary; denser nav from Administration IA; not shown as mobile worker tabs.

---

## 6. Pages & Routing

### 6.1 Routing Model

App Router file-based routes with **route groups** for shells:

| Group | Prefix | Audience |
| --- | --- | --- |
| `(auth)` | `/login`, `/guest/...` | Public/limited |
| `(app)` | `/actions`, `/projects`, … | Authenticated |
| `(app)/admin` | `/admin/...` | Admin permission |

### 6.2 Key Routes (Logical)

| Route | Surface |
| --- | --- |
| `/` | Redirect → `/actions` (mobile/worker) or `/command-center` (desktop directing roles) |
| `/actions` | My Actions |
| `/command-center` | Command Center |
| `/activity` | Activity Feed |
| `/projects` | Project list |
| `/projects/[id]` | Place overview |
| `/projects/[id]/safety/...` | Project-scoped safety |
| `/people/[id]` | Person profile tabs |
| `/equipment/[id]` | Asset readiness |
| `/documents/...` | Library / viewer / ack |
| `/training/...` | Assignments / matrix |
| `/cor/...` | Readiness / engagements |
| `/analytics/...` | Dashboards |
| `/notifications` | Inbox |
| `/admin/...` | Administration console |
| `/guest/sign/[token]` | Guest signing |
| `/find` | Mobile search |

### 6.3 Routing Rules

- Deep links preserve back context (“Back to My Actions”).  
- Project switcher updates Place context without losing module when possible.  
- Permission-gated routes use server-side session checks + client fallback UI.  
- Parallel routes/intercepting routes optional for desktop drawers (review panels)—use sparingly.

### 6.4 Middleware

Edge middleware responsibilities:

- Session cookie presence / redirect to login  
- Basic route protection  
- **Not** full AuthZ (server still enforces)  

---

## 7. State Management

### 7.1 Categories

| State | Tool | Examples |
| --- | --- | --- |
| **Server state** | TanStack Query | Projects, activities, inbox, readiness |
| **Form state** | React Hook Form | Wizards, admin forms |
| **URL state** | Search params | Filters, tabs, selected ids |
| **Ephemeral UI** | React state / `useEffectEvent` where appropriate | Drawer open, step index |
| **Offline queue** | `packages/pwa-sync` + IndexedDB | Pending mutations |
| **Auth session** | Cookie session + light client cache | Principal, tenant |
| **Theme** | Prefer CSS + small client store | Day/Dark/Site HC |

### 7.2 Explicitly Avoid

- Global Redux-style stores for server entities  
- Duplicating AuthZ grants as long-lived client truth  
- Caching eligibility decisions as enforcement  

### 7.3 TanStack Query Conventions

- Query key factory per feature: `['safety', 'activity', id]`  
- Tenant id in keys implicitly via session (never mix tenants)  
- Mutations invalidate targeted keys; optimistic updates only for allowlisted UX (e.g., mark notification read)  
- `staleTime` tuned per surface (lists short; reference libraries longer)  

---

## 8. Authentication

### 8.1 Flows

| Flow | UI |
| --- | --- |
| Password / magic link | `(auth)/login` |
| SSO/OIDC | Redirect + callback |
| Session refresh | Silent; on 401 re-auth |
| Guest magic link | `/guest/sign/[token]` |
| Logout | Clear client caches + redirect |

### 8.2 Client Responsibilities

- Attach cookies automatically (same-site)  
- On 401: pause queries, redirect to login with return URL  
- Show identity chip on shared devices  
- Guest shell never loads app rail  

### 8.3 Server Responsibilities (Next)

- Optional RSC prefetch of session bootstrap  
- No storage of refresh tokens in `localStorage`  

---

## 9. Forms

### 9.1 Stack

React Hook Form + Zod resolver for **input shaping**.

### 9.2 Rules

1. Zod schemas mirror request DTOs—not domain aggregates.  
2. Server error codes map to field/form errors.  
3. Multi-step wizards (FLHA, inspections) keep draft local + offline queue.  
4. Destructive submits use confirm patterns per UX.  
5. Signature capture is a specialized control calling Signatures APIs—not a fake checkbox.  

### 9.3 Accessibility

- Label association, `aria-invalid`, error text ids  
- Keyboard submit; don’t trap focus incorrectly in sheets  

---

## 10. Tables

### 10.1 Usage

Desktop directories and admin lists (People, Equipment, Documents, Audit viewer).

### 10.2 Design

- Virtualize large lists when needed  
- Column visibility / density for admin  
- Row click → detail or drawer  
- Server-driven pagination/cursor (never load unbounded)  
- Filters sync to URL  

Mobile: prefer **queue/list cards** over wide tables.

---

## 11. Search

### 11.1 Global Search (Desktop Top Bar)

- Projects, people, equipment, documents  
- Recent + scoped suggestions  
- Debounced query via API  

### 11.2 Mobile Find Tab

Search-first hub with type filters.

### 11.3 Rules

- Results respect AuthZ (API-enforced)  
- Empty and loading states calm and clear  
- QR scan entry on Find for equipment when camera available  

---

## 12. Notifications (UI)

### 12.1 Surfaces

- Bell popover (desktop)  
- `/notifications` inbox  
- Push prompts (PWA) for critical only after consent  

### 12.2 Behavior

- Unread badges = actionable counts where possible  
- Mark read optimistic + mutate  
- Deep link into owning feature  
- Preferences under Settings (Notifications API SoR)  

Do not duplicate Activity Feed as push spam.

---

## 13. File Uploads

### 13.1 Flow

```text
UI requests upload intent → API returns presigned URL
  → PUT to R2 from client
  → complete upload API (checksum)
  → attach to domain entity
```

### 13.2 UX

- Progress, retry, cancel  
- Camera capture on mobile for safety/equipment photos  
- Offline: queue file in IndexedDB/Cache Storage with mutation; sync upload then complete  
- Quarantine/failure messaging human-readable  
- Never upload via Next server as default path (bandwidth)  

---

## 14. Caching

| Layer | What |
| --- | --- |
| TanStack Query | Server state memory cache |
| HTTP cache | Generally bypass for authenticated JSON (`Cache-Control` private/no-store) |
| Service worker | App shell, static assets, allowlisted GETs for offline reference |
| IndexedDB | Offline drafts, mutation queue, media blobs pending sync |
| CDN | Public marketing/static only |

Invalidation: on mutation success + websocket/SSE optional later; start with query invalidation + focus refetch.

---

## 15. PWA & Offline Support

### 15.1 PWA

- Web app manifest + icons  
- Installable on mobile  
- Service worker via Serwist/Workbox (implementation choice later)  
- Update prompt when new shell available  

### 15.2 Offline Allowlist ([domain docs](./SAFETY_DOMAIN.md), Equipment, Training)

| Allow offline | Online-only (initial) |
| --- | --- |
| Safety drafts/submit (typed) | Admin console |
| Pre-use inspections | COR package generation |
| Acknowledgements where policy allows | Role/permission changes |
| Photo capture queue | Library admin |

### 15.3 Sync Protocol (Client View)

1. Enqueue mutation with `mutation_id`  
2. Show Sync Pill states  
3. Drain FIFO per aggregate constraints  
4. Apply server canonical state  
5. Surface conflicts per API error (no silent overwrite of sealed server state)  

### 15.4 Offline Reference Data

Cache hazard/control libraries, checklists, assigned crew snapshots with **staleness labels**—never treat as AuthZ.

---

## 16. Accessibility

Target **WCAG 2.2 AA** on primary flows.

| Area | Requirement |
| --- | --- |
| Keyboard | Full desktop operability; visible focus |
| Screen readers | Landmarks, names, live regions for sync/toasts |
| Contrast | All themes including Site High Contrast |
| Targets | ≥44×44px primary mobile controls |
| Motion | Honor `prefers-reduced-motion` |
| Forms | Errors associated; don’t rely on color alone |
| Tables | Row headers / accessible sort controls |

Continuous checks: lint a11y rules, periodic manual audits on My Actions, Safety wizard, Guest sign, Equipment pre-use.

---

## 17. Dark Mode & Themes

| Theme | Use |
| --- | --- |
| Day | Default office |
| Dark | Night/trailer |
| Site High Contrast | Outdoor glare |

- CSS variables for brand tokens (Admin branding overrides constrained tokens)  
- Theme toggle in Settings; follow OS with override  
- Charts/status colors theme-aware  
- Avoid pure black voids and neon glow  

---

## 18. Responsive Design

| Breakpoint strategy | Behavior |
| --- | --- |
| `< md` | Mobile tab shell; single column; sticky CTAs |
| `md–lg` | Hybrid; rail collapsible |
| `≥ lg` | Full rail + Place subnav + drawers |

- Project Place tables degrade to stacked lists on small screens  
- Admin warns or redirects on small viewports (read-only limited)  
- Touch-first spacing on field flows  

---

## 19. Performance

### 19.1 Budgets (Architectural)

- JS for worker Today path kept lean (code-split admin/analytics)  
- Route-based splitting per feature  
- Prefer RSC for chrome; hydrate wizards only  
- Images: responsive sizes; don’t block LCP with admin charts  
- Avoid giant client Zod schemas on first paint  

### 19.2 Techniques

- Dynamic import heavy PDF viewers / signature pads / analytics charts  
- Virtualize long queues/tables  
- Prefetch Place overview on project switch hover (desktop)  
- Query `placeholderData` / keepPrevious for filter transitions  
- Service worker precache shell only—not all API data  

### 19.3 Measurement

- Web Vitals on Vercel  
- Separate field vs admin entry metrics  
- Bundle analysis in CI for regressions  

---

## 20. Styling Architecture

- Tailwind utility-first  
- Design tokens as CSS variables (brand, status, surfaces)  
- shadcn/ui primitives in `packages/ui`, themed to Proven—not stock purple defaults  
- Expressive typography per UX direction  
- Subtle atmospheric backgrounds allowed; keep field forms high-clarity  

---

## 21. Feature Module Template

Each `features/<domain>` typically contains:

- `api/` — query options & mutation hooks  
- `components/` — feature UI  
- `schemas/` — Zod request schemas  
- `screens/` — page bodies imported by `app/`  
- `model/` — UI-only view models (display mapping)  

No domain invariant functions that contradict server rules.

---

## 22. Error Handling & Empty States

- Global error boundary + route `error.tsx`  
- API error → toast or inline; correlation id under Details  
- Empty states per UX (one CTA)  
- Sync failures actionable (Retry)  

---

## 23. Security (Frontend)

- CSP headers via Next/Vercel config  
- No secrets in client bundles  
- Guest tokens only in path/header for guest routes  
- Sanitize any rendered HTML (prefer no arbitrary HTML)  
- File type/size checks client-side as UX only  

---

## 24. Testing Strategy (Frontend)

| Layer | Focus |
| --- | --- |
| Unit | View-model mappers, Zod schemas |
| Component | Patterns (Queue Item, Sync Pill) with Testing Library |
| Feature | Wizard flows with MSW API mocks |
| E2E | Playwright: login, My Actions, offline submit happy path, guest sign |
| A11y | axe on critical pages in CI smoke |

---

## 25. Integration With Backend

- All data via `packages/api-client` → `/api/v1/...`  
- OpenAPI-driven types where generated  
- Never call Temporal/NATS from browser  
- Presigned R2 uploads only  

---

## 26. Success Criteria

The frontend architecture succeeds when:

1. Workers live in My Actions/PWA offline comfortably.  
2. Supervisors direct from Command Center/Place without chart noise.  
3. Feature folders scale without dumping logic into `app/`.  
4. TanStack Query owns server state; forms own input state.  
5. Zod never becomes the compliance engine.  
6. Accessibility and theme modes work on site and in office.  
7. Admin/analytics weight does not bloat field bundles.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Principal Frontend Architecture | Complete Next.js frontend architecture (no code) |

---

*End of Frontend Architecture*
