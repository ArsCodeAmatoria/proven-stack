# Proven — User Experience Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | UX Architecture |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | UX / Product Design |
| **Audience** | Design, Product, Engineering, GTM |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [PRD](../PRD.md), [Domain Model](../architecture/DOMAIN_MODEL.md), [System Architecture](../architecture/SYSTEM_ARCHITECTURE.md) |

---

## 1. Purpose

This document defines the **complete user experience** for Proven as a Construction Compliance Operating System.

The experience must feel as **intuitive as Basecamp and Connecteam**—calm, human, task-clear, low ceremony—while remaining **distinctly Proven**: proof-centered, project-scoped, field-honest, and audit-ready.

**No implementation code.** This is the UX source of truth for information architecture, navigation, key surfaces, wireframes, accessibility, and visual principles.

---

## 2. Experience Vision

> Proven should feel like the quietest powerful tool on the job site and in the trailer: you always know what needs you, what is proven, and what is at risk—without hunting through menus or dashboards.

### 2.1 Inspired By (Not Copied)

| Product | Borrow | Do not borrow |
| --- | --- | --- |
| **Basecamp** | Calm hierarchy, human language, activity as memory, one clear home, progressive disclosure | Soft “project chat” as the center of gravity; generic project management sprawl |
| **Connecteam** | Field-first mobile clarity, “what do I do now?”, large tap targets, workforce immediacy | Feature-zoo tabs; gamified clutter; generic HR toolkit aesthetics |

### 2.2 Proven’s Unique Feel

| Trait | Meaning in UX |
| --- | --- |
| **Proof over forms** | Every flow ends in visible evidence status, not “form saved” |
| **Command over dashboards** | Home is action and risk, not chart wallpaper |
| **Site reality** | Offline, gloves, sun glare, short attention, multi-contractor friction are first-class |
| **One OS** | Safety, people, equipment, training, documents share one navigation language |
| **Quiet confidence** | Strong structure, restrained chrome, no sticker-bomb UI |

---

## 3. Design Principles

1. **One job per screen** — Each view answers one question or completes one action.
2. **My Actions first** — Personal work queues beat module browsing.
3. **Project as place** — Most work is scoped to a project “place,” like a job site trailer.
4. **Plain language** — “Needs your signature,” not “Pending bilateral attestation.”
5. **Show the proof** — Status always includes evidence state (unsigned, partial, sealed).
6. **Mobile for doing, desktop for directing** — Workers execute; supervisors/admins orchestrate.
7. **Offline without apology** — Pending sync is normal UI, not an error banner lifestyle.
8. **Progressive disclosure** — Essentials first; power tools one step deeper.
9. **Calm urgency** — Priority is clear without red-alarm noise on every pixel.
10. **Accessible by default** — Field and office users both get WCAG-minded design.
11. **Consistent verbs** — Start, Continue, Submit, Review, Close, Sign, Assign, Escalate.
12. **No fake productivity** — Avoid vanity stats, badge confetti, and card grids that don’t help action.

---

## 4. Information Architecture

### 4.1 Primary IA (Authenticated)

```text
Proven
├── Home
│   ├── Command Center          (role-aware operational home — desktop primary)
│   ├── My Actions              (personal queue — all roles; mobile primary)
│   └── Activity Feed           (what happened — Basecamp-like memory)
├── Places
│   └── Projects
│       ├── Overview
│       ├── Actions
│       ├── People
│       ├── Safety
│       ├── Equipment
│       ├── Documents
│       ├── Training
│       └── Activity
├── Directory
│   ├── People
│   └── Equipment (fleet-wide)
├── Work
│   ├── Safety (cross-project inbox for safety roles)
│   ├── Documents
│   ├── Training
│   └── Signatures / Requests
├── Insight
│   ├── Analytics
│   └── COR Readiness
├── Admin
│   ├── Organization
│   ├── Users & Access
│   ├── Workflows & Templates
│   ├── Modules & Policies
│   └── Audit Log
└── System
    ├── Notifications
    ├── Profile / Preferences
    └── Help
```

### 4.2 Mobile Worker IA (Simplified)

```text
Tab bar:
  Today (My Actions)
  Project (current project place)
  Scan/Find (people/equipment/docs search)
  Activity
  Menu (Training, Documents, Profile, Help, Switch project)
```

### 4.3 Mental Model: Places, Queues, Proof

| Concept | User meaning |
| --- | --- |
| **Place** | A project site context where work happens |
| **Queue** | Things waiting on me or my crew |
| **Proof** | Signed, sealed, current evidence |
| **Gap** | Missing/expired/blocked compliance |
| **Feed** | Chronological memory of what happened |

### 4.4 Object Hierarchy

```text
Tenant
  └── Project (Place)
        ├── People memberships
        ├── Safety activities & actions
        ├── Equipment assignments
        ├── Documents & acknowledgements
        ├── Training requirements
        └── Activity events
  └── Fleet People / Equipment (directory)
  └── Program (COR, templates, policies)
```

---

## 5. Navigation

### 5.1 Desktop Navigation

**Left primary rail (narrow, icon + label):**

- Home  
- My Actions (badge count)  
- Projects  
- People  
- Safety  
- Equipment  
- Documents  
- Training  
- Analytics  
- Admin (permission-gated)  

**Top bar:**

- Global search  
- Current project switcher (sticky context)  
- Notifications bell  
- Help  
- Account / theme  

**Secondary pattern:** Inside a Project Place, a **horizontal place nav** appears under the header (Overview · Actions · People · Safety · Equipment · Documents · Training · Activity).

### 5.2 Mobile Navigation

- **Bottom tabs** for the five worker essentials (Today, Project, Find, Activity, Menu).
- **No hamburger-as-home.** Menu is overflow, not the product.
- Project switcher is always one tap from Today and Project tabs.
- Destructive/admin items stay out of worker tabs.

### 5.3 Navigation Rules

1. Deep links land with clear back context (“Back to My Actions” / “Back to Harbour Bridge West”).
2. Cross-module jumps preserve project context whenever possible.
3. Admin never hijacks field flows.
4. Badge counts show **actionable** items only (not vanity notifications).

---

## 6. Design Principles for Layout & Visual Language

### 6.1 Layout

- **Desktop:** Stable rail + content canvas; prefer lists, timelines, and workboards over card mosaics.
- **Mobile:** Single column; primary CTA sticky where needed; thumb-zone primary actions.
- **Density:** Office views denser; field views airier with larger controls.

### 6.2 Visual Direction (Unique to Proven)

Proven should feel like a **trusted field operations instrument**—clear steel and site daylight—not a purple SaaS kit, not a cream editorial magazine, not a neon dark-glam dashboard.

| Token idea | Direction |
| --- | --- |
| Brand signal | Strong wordmark “Proven” in home headers; product name is visible, not an eyebrow |
| Color | Deep slate / ink for structure; high-visibility safety accent used sparingly for true urgency; success as “sealed proof” teal/green distinct from generic Bootstrap green |
| Surfaces | Soft layered depth via subtle gradients or fine site-texture patterns; avoid flat white void and avoid heavy multi-shadow cards |
| Typography | Distinctive pair: sturdy grotesque for UI + readable condensed for job-site headers (not Inter/Roboto/Arial defaults) |
| Motion | 2–3 purposeful motions: queue item completion, proof seal, sync state — not decorative bounce |
| Cards | Default **no cards** on marketing/hero-like surfaces; in-app, use cards only when they contain an interaction or discrete work item |

### 6.3 Status Language

| Status | Meaning | Visual cue |
| --- | --- | --- |
| Due | Needs action soon | Neutral emphasis |
| Overdue | Past SLA | Strong accent |
| Blocked | Cannot proceed | Explicit lock/reason |
| In review | Waiting on someone else | Quiet waiting state |
| Proven / Sealed | Evidence complete | Distinct “proof” treatment |
| Syncing | Offline queue draining | Persistent but calm |

---

## 7. Desktop Experience

### 7.1 Who It’s For

Supervisors, safety coordinators, project managers, training admins, executives, company admins.

### 7.2 Desktop Character

- **Command Center** as default Home for directing roles.
- Multi-pane capable: list + detail without route thrash.
- Keyboard search (`/`) and quick assign patterns.
- Tables allowed for People/Equipment/Admin; still human headers, not ERP coldness.
- Side panels for reviews/signatures without losing the queue.

### 7.3 Desktop Home Hierarchy

1. What needs direction today (exceptions, overdue, blocked)  
2. Crew/project health strip (only if actionable)  
3. Activity memory  
4. Shortcuts to Places  

Avoid first-viewport chart walls.

---

## 8. Mobile PWA Experience

### 8.1 Who It’s For

Workers, operators, foremen on site; supervisors in the field.

### 8.2 Mobile Character

- **Today / My Actions** is the home tab.
- Large tap targets; glove-tolerant spacing.
- High contrast outdoor mode support (pairs with theme; see Dark mode).
- Offline banner only when pending or failed; “Saved on device” is reassuring, not alarming.
- Camera/file capture in-flow for evidence.
- Install prompt treated as benefit (“Use Proven like an app on site”), not nag.

### 8.3 Field Constraints Honored

| Constraint | UX response |
| --- | --- |
| Bad network | Queue + sync status + retry |
| One hand / standing | Sticky primary CTA; minimal typing via choosers |
| Sun glare | Contrast themes; avoid gray-on-gray |
| Time pressure | Short flows; save & continue |
| Shared devices ( occasional ) | Clear identity chip; easy switch user with auth safeguards |

---

## 9. Core Surfaces

---

### 9.1 Dashboard → Command Center

Proven does **not** lead with a generic “Dashboard.” The desktop home is the **Command Center**.

**Purpose:** Answer “What needs my direction across sites right now?”

**Primary blocks:**

1. **Needs you** — approvals, reviews, escalations  
2. **At risk** — overdue corrective actions, expiry within window, blocked work  
3. **Sites snapshot** — compact project list with proof health (not 12 charts)  
4. **Jump back** — recent projects  

**Anti-patterns:** KPI tile grids, donut charts above the fold, “welcome” empty marketing.

#### Wireframe — Desktop Command Center

```text
┌─ Rail ─┬─ Command Center ─────────────────────────────────────── 🔔  Acc ─┐
│ Home●  │ Good morning, Sam                                              │
│ Actions│                                                                 │
│ Proj   │ ┌ Needs you (8) ─────────────┐  ┌ At risk (5) ───────────────┐ │
│ People │ │ □ Review FLHA – Pier 3     │  │ ! CA overdue – Edge prot.  │ │
│ Safety │ │ □ Sign toolbox – Crew B    │  │ ! Cert expires in 3d – C12 │ │
│ Equip  │ │ □ Close incident draft     │  │ ! Training gap – 2 workers │ │
│ Docs   │ │ View all actions →         │  │ View risks →               │ │
│ Train  │ └────────────────────────────┘  └────────────────────────────┘ │
│ Analyt │                                                                 │
│ Admin  │ Sites                                                           │
│        │ Harbour Bridge West    Proof 92%   3 due   Open →               │
│        │ Yard 14 Retrofit       Proof 71%   6 due   Open →               │
│        │                                                                 │
│        │ Recent activity                                      See feed → │
│        │ · Alex submitted pre-use – Crane 04                             │
│        │ · Priya closed corrective action #182                           │
└────────┴─────────────────────────────────────────────────────────────────┘
```

---

### 9.2 My Actions

**Purpose:** Personal work queue—“What do I need to do?”

**Universal across roles**; content filtered by assignments and permissions.

**Groupings:**

- Due today  
- Waiting on you (signatures, reviews)  
- Overdue  
- Waiting on others (informational, secondary)  

**Mobile:** default tab.  
**Desktop:** reachable from rail; also embedded as Command Center “Needs you.”

#### Wireframe — Mobile My Actions

```text
┌─────────────────────────┐
│ Proven        Harbour ▾ │
│ Today                   │
│─────────────────────────│
│ Due today (3)           │
│ ┌─────────────────────┐ │
│ │ Start FLHA          │ │
│ │ Pier 3 · 10:00      │ │
│ │ [Start]             │ │
│ └─────────────────────┘ │
│ ┌─────────────────────┐ │
│ │ Sign toolbox talk   │ │
│ │ Crew B              │ │
│ │ [Sign]              │ │
│ └─────────────────────┘ │
│ Overdue (1)             │
│ ┌─────────────────────┐ │
│ │ Training: WHMIS     │ │
│ │ Expired             │ │
│ │ [Fix]               │ │
│ └─────────────────────┘ │
│─────────────────────────│
│ Today  Proj  Find  Act  │
└─────────────────────────┘
```

---

### 9.3 Activity Feed

**Purpose:** Shared memory of what happened—Basecamp-like timeline, compliance-flavored.

**Shows:** submissions, signs, closures, assignments, expiries, package exports—permission-scoped.

**Does not replace My Actions.** Feed is awareness; Actions is obligation.

**Filters:** Project · Type · People · Mine.

#### Wireframe — Activity Feed (Desktop)

```text
┌─ Activity ──────────────────────────────────────────────────────────┐
│ Filters: [This project ▾] [All types ▾] [Everyone ▾]     Search…    │
│                                                                     │
│ Today                                                               │
│ ○ 13:42  Alex R. sealed pre-use inspection · Crane 04 · Proven     │
│ ○ 13:10  Sam K. assigned FLHA to Crew B                            │
│ ○ 11:05  System: Welding cert enters expiry window · M. Chen       │
│                                                                     │
│ Yesterday                                                           │
│ ○ 16:20  Jordan closed CA #182 · Edge protection                   │
│ ○ 09:15  Toolbox talk completed · 12 signatures sealed             │
└─────────────────────────────────────────────────────────────────────┘
```

---

### 9.4 Projects

**Purpose:** Project as **Place**—the digital job site.

**List (desktop):** searchable table/list with proof health, open actions, participants.  
**Place Overview:** short status narrative + Needs attention + shortcuts—not a widget fair.

**Place tabs:** Overview · Actions · People · Safety · Equipment · Documents · Training · Activity

#### Wireframe — Project Place Overview

```text
┌─ Projects / Harbour Bridge West ────────────────────────────────────┐
│ Harbour Bridge West          Active · Vancouver · Prime: Northline  │
│ [Overview] Actions People Safety Equipment Docs Training Activity   │
│                                                                     │
│ Needs attention (4)                              View all actions → │
│ · 2 FLHAs awaiting review                                           │
│ · 1 equipment out of service on site                                │
│ · 1 training gap blocking night shift                               │
│                                                                     │
│ Proof health        People on site      Equipment ready             │
│ Sealed 92%          48 assigned         31/33                       │
│                                                                     │
│ Today on this site                                                  │
│ · Toolbox 07:30 · Crew A (sealed)                                   │
│ · Lift plan ack due · Crane team                                    │
└─────────────────────────────────────────────────────────────────────┘
```

---

### 9.5 People

**Purpose:** Directory of workers/staff with eligibility signals and crew context.

**Views:**

- Directory (company/tenant)  
- Project People (membership + role)  
- Person profile (training, docs, acknowledgements, recent proof)—permission aware  

**Supervisor lens:** crew list with “Eligible / Gaps” not HR sprawl.

#### Wireframe — People (Project)

```text
┌─ People on Harbour Bridge West ──────────────────────────── [Add] ─┐
│ Search people…          Trade ▾   Status ▾                         │
│                                                                    │
│ Name            Trade     Role        Eligibility                  │
│ Alex Rivera     Labour    Worker      Ready                        │
│ M. Chen         Welder    Worker      Gap · WHMIS expired          │
│ Sam K.          —         Supervisor  Ready                        │
│                                                                    │
│ Side panel (on row): profile snapshot · assign · message path via  │
│ notifications only (no social network)                             │
└────────────────────────────────────────────────────────────────────┘
```

---

### 9.6 Safety

**Purpose:** Run and evidence safety work—activities, attendance, corrective actions, incidents.

**Entry points:**

- My Actions (start/sign/review)  
- Project → Safety  
- Cross-project Safety inbox (safety coordinators)  

**Flow character:** short steps, save draft, submit, seal signatures, show proof state.

**Lists:** Open activities · Corrective actions · Incidents (as released)

#### Wireframe — Safety Activity (Mobile)

```text
┌─ FLHA · Pier 3 ────────────────┐
│ Draft · Offline saved          │
│────────────────────────────────│
│ 1. Task & location             │
│ 2. Hazards                     │
│ 3. Controls                    │
│ 4. Crew                        │
│ 5. Sign                        │
│────────────────────────────────│
│ Hazards                        │
│ [+] Add hazard                 │
│ · Fall from height             │
│   Control: harness + spotter   │
│────────────────────────────────│
│        [Save]     [Continue]   │
└────────────────────────────────┘
```

---

### 9.7 Equipment

**Purpose:** Find assets, see readiness, run inspections, surface expiries.

**Mobile:** scan/search → readiness → pre-use check → sign → proof.  
**Desktop:** fleet tables, assignment, cert windows, out-of-service reasons.

#### Wireframe — Equipment Readiness (Mobile)

```text
┌─ Crane 04 ─────────────────────┐
│ Ready · Assigned HB West       │
│────────────────────────────────│
│ Inspection   Current           │
│ Certification Current (12d)    │
│ Documents    2 controlled      │
│────────────────────────────────│
│ [Start pre-use check]          │
│                                │
│ Recent proof                   │
│ · Pre-use sealed · Today 06:55 │
└────────────────────────────────┘
```

---

### 9.8 Documents

**Purpose:** Controlled documents—current version, acknowledge, distribute.

**UX emphasis:** “You are viewing the effective version” always visible.  
**Worker:** required reads/acks in My Actions.  
**Admin/Safety:** library, publish, retire, distribution.

Avoid generic file-manager energy; prefer **required reading queues** + library.

---

### 9.9 Training

**Purpose:** Know what’s required, what’s expired, how to become eligible again.

**Worker:** gap cards with Fix path.  
**Admin:** requirements by role/trade/project; matrix views on desktop.  
**Supervisor:** crew gaps before shift.

Training UX should feel like **eligibility repair**, not an LMS mall.

---

### 9.10 Analytics

**Purpose:** Insight for directing roles—trends and hotspots after action queues.

**Placement:** not Home for field users; Insight area for PM/Safety/Exec.

**Views:**

- Portfolio proof health  
- Overdue trends  
- Training currency  
- Equipment readiness  
- COR readiness (linked)

**Rule:** Charts explain exceptions; they don’t replace My Actions / Command Center.

#### Wireframe — Analytics (Desktop)

```text
┌─ Analytics ─────────────────────────────────────────────────────────┐
│ Range [30d ▾]   Scope [All projects ▾]                              │
│                                                                     │
│ Exceptions needing program attention                                │
│ · Corrective actions aging > 7d rose on 2 sites                     │
│ · Training expiries cluster: Welding · Yard 14                      │
│                                                                     │
│ Proof completion        Overdue CA trend        Readiness           │
│ (simple trend)          (simple trend)          (projects list)     │
│                                                                     │
│ [Export]  [Open COR readiness]                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

### 9.11 Administration

**Purpose:** Configure the OS—org, access, templates, policies, modules—without field clutter.

**Desktop-only primary.** Mobile shows read-only account essentials.

**Sections:**

- Organization & companies  
- Users & access  
- Project templates  
- Workflow templates  
- Document/signature policies  
- Module entitlements  
- Audit log  

Admin language stays human: “Who can close incidents?” not raw permission codes (codes available in advanced).

---

### 9.12 Notifications

**Purpose:** Timely nudges for assignments, expiries, escalations, signature requests.

**Inbox + bell.** Group by day; mark actionable vs FYI.  
**Preferences:** user can quiet FYI; tenants can force critical safety channels.

**Mobile:** system push for critical; in-app inbox for the rest.  
**Do not** duplicate every feed item as a push.

---

### 9.13 Guest Signing

**Purpose:** Let non-account participants (or lightly gated guests) sign when policy allows—e.g., visitor orientation, sub acknowledgement—without full product access.

**Characteristics:**

- Magic-link or code path, project-scoped, time-boxed  
- Single-purpose screen: read → sign → done  
- Strong identity capture as policy requires (name, company, phone)  
- No navigation into the OS  
- Branded Proven minimal chrome; proof confirmation at end  

#### Wireframe — Guest Signing

```text
┌──────────────────────────────────────────┐
│ Proven                                   │
│ Harbour Bridge West · Visitor orientation│
│──────────────────────────────────────────│
│ Please review the site rules (v3).       │
│ [Open document]                          │
│                                          │
│ Your details                             │
│ Name ________________________________    │
│ Company _____________________________    │
│                                          │
│ Signature                                │
│ ┌──────────────────────────────────────┐ │
│ │                                      │ │
│ └──────────────────────────────────────┘ │
│ [Clear]              [Seal signature]    │
│                                          │
│ Encrypted · Timestamped · Project-bound  │
└──────────────────────────────────────────┘
```

---

### 9.14 Digital Signatures

**Purpose:** Audit-grade assent bound to person/time/version/context.

**In-product patterns:**

- Signature step inside flows (Safety, Inspection, Acknowledgement, Training evidence)  
- Multi-signer progress (“3 of 12 sealed”)  
- Proof panel: who, when, what version, device session meta as policy allows  
- Void path with reason (permissioned)  

**Language:** use **Seal** for finalizing a signature package; **Proven** state when package complete.

**Desktop review:** side panel with signature evidence while keeping the activity visible.

#### Wireframe — Multi-signer Progress

```text
┌─ Toolbox talk · Crew B ─────────────────────────────────────────────┐
│ Status: Waiting on signatures · 8 of 12 sealed                      │
│ ████████████░░░░                                                     │
│                                                                     │
│ Sealed: Alex, Sam, …                                                │
│ Waiting: M. Chen, J. Ortiz, …                          [Remind]     │
│                                                                     │
│ [View evidence]                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 10. Accessibility

### 10.1 Standard

Target **WCAG 2.2 AA** for primary flows (My Actions, Safety complete/sign, Equipment pre-use, Guest signing, Admin core).

### 10.2 Requirements

| Area | UX requirement |
| --- | --- |
| Contrast | Text/status meet AA; outdoor theme available |
| Keyboard | All desktop actions reachable; visible focus |
| Screen readers | Meaningful labels; status announced; live regions for sync/errors |
| Targets | ≥ 44×44 px primary controls on mobile |
| Motion | Respect reduced-motion; seal animation optional |
| Forms | Errors inline, associated labels, no color-only meaning |
| Auth | Accessible SSO and guest flows |
| Language | Plain language; avoid icon-only critical actions |

### 10.3 Field Accessibility

- Voice/OS zoom friendly  
- One-column reading order  
- Haptics optional, never required  

---

## 11. Dark Mode

### 11.1 Themes

| Theme | Use |
| --- | --- |
| **Day** | Default office; light structured surfaces |
| **Dusk / Dark** | Low-light trailers, night shifts |
| **Site High Contrast** | Outdoor glare; maximized contrast (may be light or dark base) |

User preference in Profile; follow OS preference by default with override.

### 11.2 Rules

- Dark mode is a **first-class theme**, not inverted colors.
- Status colors remain distinguishable in all themes.
- Signature canvas contrast must remain clear.
- Avoid pure `#000` voids and glowing neon accents.
- Charts and maps (if any) ship theme-aware palettes.

---

## 12. Cross-Cutting UX Patterns

### 12.1 Sync Pattern

```text
Saved on device → Syncing… → Proven on server
                     ↘ Failed – Retry
```

Always retain local draft until server accepts or user discards intentionally.

### 12.2 Empty States

Human, specific, one CTA.  
Example: “Nothing needs you on Harbour Bridge West. Switch project or review activity.”

### 12.3 Errors

Say what happened and what to do. Never raw stack traces. Correlation ID behind “Details” for support.

### 12.4 Confirmations

Use confirmations for void, close, revoke access, publish controlled docs—not for every save.

### 12.5 Search

Global search: projects, people, equipment, documents. Recent + scoped suggestions. Mobile Find tab is search-first.

---

## 13. Role Experience Maps

| Role | Default Home | Primary daily tools |
| --- | --- | --- |
| Worker | My Actions (mobile) | Today queue, Project, Sign, Training gaps |
| Supervisor | Command Center / Today | Crew actions, Safety run, Equipment readiness |
| Safety coordinator | Command Center | Safety inbox, CA, Documents, COR |
| PM | Command Center | Project places, exceptions, Analytics |
| Equipment manager | Equipment + Actions | Fleet readiness, expiries |
| Training admin | Training | Requirements, gaps, evidence |
| Admin | Admin | Access, templates, policies |
| Guest | Guest Signing only | Read + seal |

---

## 14. End-to-End Journey Sketches

### 14.1 Morning Crew Start

```text
Supervisor opens Command Center
  → checks At risk / Needs you
  → opens Project Place
  → starts toolbox talk
  → workers receive My Actions + push
  → workers seal signatures (online/offline)
  → progress hits 12/12 Proven
  → Activity Feed records sealed event
```

### 14.2 Pre-Use Inspection Offline

```text
Operator opens Equipment via Find
  → readiness shows last known
  → completes pre-use offline
  → seals signature on device
  → reconnect syncs
  → status becomes Proven
  → failure would notify supervisor Actions
```

### 14.3 Guest Orientation

```text
Gate issues guest link
  → guest opens minimal flow
  → reads controlled doc version
  → seals signature
  → receives confirmation
  → project People/visitors shows proof
```

---

## 15. Wireframe Index (Summary)

| ID | Surface | Primary viewport |
| --- | --- | --- |
| W1 | Command Center | Desktop Home |
| W2 | My Actions | Mobile Home |
| W3 | Activity Feed | Desktop/Mobile |
| W4 | Project Place Overview | Desktop |
| W5 | People list | Desktop |
| W6 | Safety activity wizard | Mobile |
| W7 | Equipment readiness | Mobile |
| W8 | Analytics | Desktop |
| W9 | Guest Signing | Mobile/Desktop guest |
| W10 | Multi-signer progress | Both |

Additional detailed wireframes for Documents, Training, Admin, and Notifications follow the same list/detail + queue patterns defined above and should reuse components: **Queue Item**, **Proof Seal**, **Place Header**, **Status Chip**, **Sync Pill**.

---

## 16. Component Pattern Library (UX-level)

| Pattern | Use |
| --- | --- |
| Queue Item | My Actions, Command Center Needs you |
| Proof Seal | Signature complete state |
| Place Header | Project context + tabs |
| Eligibility Pill | Ready / Gap / Blocked |
| Sync Pill | Offline state |
| Evidence Panel | Who/when/version |
| Empty State | Human + one CTA |
| Review Drawer | Desktop side panel for approve/sign |
| Sticky CTA | Mobile flow continuation |

Cards are used for Queue Items and discrete work objects—not as decorative chrome.

---

## 17. Content & Microcopy Principles

- Prefer verbs users do on site: Start, Continue, Submit, Sign/Seal, Close, Fix gap.  
- Name people and projects early.  
- Put consequence in overdue/blocked states (“Blocked: WHMIS expired”).  
- Celebrate sparingly: completion check + “Proven” is enough—no confetti.  
- System voice is calm, competent, localizable.

---

## 18. UX Success Metrics

| Signal | Indicator |
| --- | --- |
| Time to first action | Worker completes first My Actions item quickly after login |
| Signature completion | Multi-signer flows finish without support |
| Offline success | Sync failure rate low; users understand pending state |
| Navigation clarity | Fewer “where do I go?” support tickets |
| Command Center usefulness | Directing roles start day in Home, not buried modules |
| Accessibility | Critical flows usable with keyboard/screen reader |

---

## 19. Out of Scope for UX v1

- Social chat/network as a product surface  
- Gamification badges  
- Marketing dashboard first-run walls inside the app  
- Dense ERP multi-toolbar desktops  
- Icon-only navigation without labels on mobile  

---

## 20. Design Handoff Notes

1. IA and principles in this document gate visual UI exploration.  
2. Visual brand should pass the **brand test**: first viewport remains recognizably Proven if chrome labels were removed—via wordmark hierarchy and unique material language.  
3. Engineering should map surfaces to modules without exposing module names as raw IA unless user-meaningful.  
4. Any new surface must declare: primary user, primary question, proof outcome, online/offline behavior.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Senior UX Architecture | Initial complete UX architecture for Proven |

---

*End of UX Architecture*
