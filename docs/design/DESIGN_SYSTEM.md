# Proven Design System

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Design System |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Product Design |
| **Audience** | Design, Frontend, Brand, Marketing |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [UX Architecture](../ux/UX_ARCHITECTURE.md), [Frontend Architecture](../architecture/FRONTEND_ARCHITECTURE.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document is the **complete Proven Design System**: visual language, design tokens, and component guidance for a Construction Compliance Operating System.

The system must feel **professional, minimal, and fast**—built for job sites and trailers—not generic SaaS. It supports **light, dark, and high-contrast site** themes across **mobile, tablet, and desktop**.

**Documentation only — no implementation.**

Logo mark reference: Lucide [fingerprint-pattern](https://lucide.dev/icons/fingerprint-pattern) (identity / proof).
README treatment: **white** fingerprint on **earthy** `#3F3A32` (`assets/brand/logo.svg`). Highlight / emphasis text uses **mustard** `#C9A227`.

---

## 2. Design Principles

1. **Proof over chrome** — Visual energy goes to status, seal, and action—not decoration.  
2. **Field first** — Readable in sun and gloves; large targets; calm motion.  
3. **Instrument, not magazine** — Steel clarity and daylight structure; no editorial cream/terracotta kits; no purple-glow SaaS.  
4. **Minimal surface** — Default no decorative cards; reduce borders and shadows.  
5. **One job per view** — Hierarchy is ruthless; secondary content steps back.  
6. **Fast perceived performance** — Skeletons over spinners when possible; instant press feedback.  
7. **Accessible by default** — WCAG 2.2 AA; status never color-only.  
8. **Brand visible** — “Proven” wordmark is a hero-level signal on branded entry surfaces.  
9. **Purposeful motion** — 2–3 signature animations; no bounce spam.  
10. **Token-driven** — Themes swap tokens; components don’t hardcode mood colors.

---

## 3. Brand Character

| Attribute | Expression |
| --- | --- |
| Industry | Construction compliance — durable, precise, trustworthy |
| Personality | Calm competent foreman + sharp auditor |
| Voice in UI | Plain verbs: Start, Seal, Fix gap, Review |
| Metaphor | Field instrument / sealed evidence |

**Avoid:** neon cyberpunk, playful stickers, confetti, purple gradients, broadsheet newspaper density, soft “wellness” beige brands.

---

## 4. Breakpoints & Layout Grid

### 4.1 Breakpoints

| Name | Range | Primary use |
| --- | --- | --- |
| **Mobile** | 0–767px | Worker PWA; tab shell |
| **Tablet** | 768–1023px | Hybrid; collapsible rail |
| **Desktop** | 1024px+ | Rail + Place subnav + drawers |
| **Wide** | 1440px+ | Comfortable admin/analytics denser grids |

### 4.2 Grid

| Context | Columns | Gutter | Margin |
| --- | --- | --- | --- |
| Mobile | 4 | 16 | 16 |
| Tablet | 8 | 16–24 | 24 |
| Desktop | 12 | 24 | 24–32 |
| Content max | — | — | Main canvas ~1200–1440 reading width; tables may full-bleed within shell |

### 4.3 Density Modes

| Mode | Use |
| --- | --- |
| **Field** | Larger type/spacing/targets; airier |
| **Office** | Compact tables; denser admin |

Toggle implicitly by surface (worker flows = field; admin = office).

---

## 5. Typography

### 5.1 Font Roles

Do **not** use Inter, Roboto, Arial, or system-ui as the brand face.

| Role | Character | Suggested direction |
| --- | --- | --- |
| **Display / Place headers** | Condensed, sturdy, industrial-legible | Condensed grotesque or technical sans (e.g. family akin to *IBM Plex Sans Condensed* / *Schibsted Grotesk* condensed—final license TBD) |
| **UI / Body** | Clear grotesque, excellent small sizes | Sturdy grotesque distinct from Inter (e.g. *Source Sans 3*, *IBM Plex Sans*, *Geist* only if differentiated by pairing—prefer non-default pair) |
| **Mono** | Evidence IDs, codes, audit | Tabular mono (*IBM Plex Mono* or similar) |

Exact font files chosen at implementation; **pairing rule**: condensed display + readable UI sans + mono.

### 5.2 Type Scale (Token Names)

| Token | Size | Line height | Use |
| --- | --- | --- | --- |
| `font.size.xs` | 12 | 16 | Meta, timestamps |
| `font.size.sm` | 14 | 20 | Secondary body, table cells |
| `font.size.md` | 16 | 24 | Default body / field inputs |
| `font.size.lg` | 18 | 28 | Emphasized body |
| `font.size.xl` | 20 | 28 | Section titles (mobile) |
| `font.size.2xl` | 24 | 32 | Page titles |
| `font.size.3xl` | 30 | 36 | Place name (tablet+) |
| `font.size.4xl` | 36 | 40 | Brand/home display (branded surfaces) |

Weights: `regular (400)`, `medium (500)`, `semibold (600)`, `bold (700)` — prefer medium/semibold over heavy black.

### 5.3 Type Rules

- Page title: one primary; don’t stack competing H1s.  
- Place names use display/condensed.  
- Tabular numerals for KPIs and due dates.  
- Minimum body 16px on field mobile inputs.  
- Letter-spacing slightly open on all-caps chips (rare—prefer sentence case).

---

## 6. Spacing

### 6.1 Space Scale (4px base)

| Token | Value |
| --- | --- |
| `space.0` | 0 |
| `space.1` | 4 |
| `space.2` | 8 |
| `space.3` | 12 |
| `space.4` | 16 |
| `space.5` | 20 |
| `space.6` | 24 |
| `space.8` | 32 |
| `space.10` | 40 |
| `space.12` | 48 |
| `space.16` | 64 |

### 6.2 Usage

| Context | Guidance |
| --- | --- |
| Field stack gaps | `space.4`–`space.6` |
| Office compact stacks | `space.2`–`space.4` |
| Section breaks | `space.8`–`space.12` |
| Rail width | ~72–88 icon+label; expanded ~200–240 |
| Mobile tab bar | Safe-area aware; height ~56–64 + inset |

---

## 7. Colors

### 7.1 Brand & Neutrals (Semantic Direction)

Ink/slate structure + restrained safety accent + proof teal.

**Light foundation**

| Token | Intent | Approx guidance |
| --- | --- | --- |
| `color.bg.canvas` | App background | Warm-neutral light gray-slate (~#EEF1F4), not pure #FFF void; subtle grain optional |
| `color.bg.surface` | Elevated panels | #FFFFFF / soft lift |
| `color.bg.subtle` | Striped rows, chips bg | Slate-100 |
| `color.fg.primary` | Primary text | Ink slate ~#0F172A |
| `color.fg.secondary` | Meta | Slate-600 |
| `color.fg.muted` | Disabled/meta | Spalte-400–500 with AA checks |
| `color.border.default` | Hairlines | Slate-200/300 |
| `color.border.strong` | Emphasis | Slate-400 |
| `color.brand.fg` | Brand wordmark/ink | Deep warm umber / ink (~#2C2620) |
| `color.brand.accent` | Highlight / emphasis text | **Mustard** `#C9A227` (not used as a logo ring) |

**Dark foundation**

| Token | Intent |
| --- | --- |
| `color.bg.canvas` | Deep slate (~#0B1220), not #000 |
| `color.bg.surface` | Elevated ~#121A2A |
| `color.fg.primary` | Off-white slate |
| `color.border.default` | Low-contrast steel borders |

**Site High Contrast**

Maximize fg/bg contrast; thicken focus rings; simplify atmosphere textures off.

### 7.2 Status Colors (Semantic)

| Token | Meaning | Notes |
| --- | --- | --- |
| `color.status.due` | Due soon | Neutral strong border/text |
| `color.status.overdue` | Past SLA | High-vis amber/red-amber — use sparingly |
| `color.status.blocked` | Cannot proceed | Red-ink + icon lock |
| `color.status.review` | Waiting | Cool slate/blue-gray |
| `color.status.sealed` / `proof` | Proven sealed | Distinct teal/green (**not** Bootstrap green clone)—signature success |
| `color.status.sync` | Offline sync | Calm info blue-steel |
| `color.status.info` | Informational | Steel blue |
| `color.focus.ring` | Focus | High-contrast ring 2–3px |

**Rule:** Always pair status color with icon and/or text label.

### 7.3 Safety Accent Budget

High-visibility accent reserved for **true urgency** (overdue critical, blocked, critical finding)—not for primary buttons by default. Primary actions use ink/brand solid, not alarm red.

### 7.4 Charts Palette

Ordered, colorblind-safe sequence distinct in light/dark; avoid red/green-only encodings—use shape/pattern too.

### 7.5 Tenant Branding Overrides

Admin branding may override logo + limited accent tokens within contrast guards; cannot break status semantics.

---

## 8. Icons

### 8.1 Library

Primary set: **Lucide** (consistent with fingerprint logo mark).

### 8.2 Sizes

| Token | Size | Use |
| --- | --- | --- |
| `icon.sm` | 16 | Inline meta |
| `icon.md` | 20 | Default UI |
| `icon.lg` | 24 | Nav / empty states |
| `icon.xl` | 32 | Feature empty/hero |

### 8.3 Rules

- Stroke icons; match Lucide 2px optical weight.  
- Nav icons always with labels on mobile; desktop rail shows label.  
- Status icons mandatory with color.  
- Don’t invent a second icon family mid-product.  
- Decorative icons `aria-hidden`; actionable icons named.

---

## 9. Elevation, Radius, Borders

| Token | Guidance |
| --- | --- |
| `radius.none` | 0 — tables, dense admin optional |
| `radius.sm` | 4 — inputs chips |
| `radius.md` | 8 — controls, queue items |
| `radius.lg` | 12 — sheets/panels (rare) |
| `radius.full` | Pills — **avoid** as default fashion; use sparingly for counts |

Shadows: **one subtle level max** for floating drawers; prefer border + surface shift over multi-layer glow. No neon shadows.

Atmosphere: optional micro-gradient or fine noise on canvas—never overpower content.

---

## 10. Design Tokens Catalog

### 10.1 Token Groups

```text
foundation/
  color/ (bg, fg, border, brand, status, focus, chart)
  font/ (family, size, weight, lineHeight)
  space/
  radius/
  borderWidth/
  shadow/
  motion/ (duration, easing)
  zIndex/
  breakpoint/
  opacity/
component/   # optional aliases referencing foundation
  button/
  input/
  table/
  …
```

### 10.2 Motion Tokens

| Token | Value guidance |
| --- | --- |
| `motion.duration.instant` | 0–80ms |
| `motion.duration.fast` | 120–160ms |
| `motion.duration.normal` | 200–240ms |
| `motion.duration.slow` | 320–400ms (seal only) |
| `motion.easing.standard` | ease-out / emphasized decelerate |
| `motion.easing.linear` | progress bars |

Respect `prefers-reduced-motion: reduce` → snap or crossfade opacity only.

### 10.3 Z-Index Scale

`base` → `sticky` → `dropdown` → `drawer` → `modal` → `toast` → `spotlight`

---

## 11. Component Library

Components ship in `packages/ui` (primitives) + app patterns. Below is the **spec**, not code.

---

### 11.1 Buttons

| Variant | Use |
| --- | --- |
| **Primary** | One main CTA per view (Start, Continue, Seal) |
| **Secondary** | Supporting actions |
| **Ghost** | Tertiary/toolbar |
| **Destructive** | Void, delete—always confirm when irreversible |
| **Proof** | Optional seal CTA using proof color—only for finalize signature |

Sizes: `sm` office · `md` default · `lg` field.

Rules:

- Min height field `44px`; office `36–40px`.  
- Full-width primary on mobile sticky footers.  
- Loading: replace label with spinner + `aria-busy`; don’t layout-shift.  
- Don’t use multiple primaries side by side.

---

### 11.2 Cards

**Default: no cards.**

Allowed:

- **Queue Item** — interactive work unit (My Actions)  
- **Interactive selection tiles** — rare  

Disallowed:

- Card grids for KPIs on Home  
- Hero media cards with overlays  
- Nested cards  

Queue Item anatomy: title, context line, status chip, primary action.

---

### 11.3 Tables

- Header sticky optional; left-align text; right-align numbers.  
- Row height office compact; zebra via `bg.subtle` not heavy lines.  
- Hover state subtle; selected row clear.  
- Empty table → Empty State pattern.  
- Mobile: transform to definition list / stacked rows.  
- Sorting indicators accessible.

---

### 11.4 Forms

| Element | Spec |
| --- | --- |
| Label | Above field; sentence case |
| Input | 16px text mobile; clear border; focus ring token |
| Helper | `fg.secondary` |
| Error | Text + icon; `aria-invalid` |
| Required | Indicate explicitly; don’t rely on color alone |
| Groups | One question cluster per section in wizards |

Wizard: progress as simple steps text (“2 of 5”), not gamified.

Signature pad: high-contrast canvas; Clear + Seal actions.

---

### 11.5 Dialogs & Sheets

| Type | Use |
| --- | --- |
| **Modal dialog** | Destructive confirms; short decisions desktop |
| **Drawer / Sheet** | Review/sign side panel desktop; mobile bottom sheet |
| **Full screen mobile** | Long guest/sign or wizard when needed |

Rules: focus trap; Escape closes; restore focus; title required; avoid modal stacks.

---

### 11.6 Charts

Used in Analytics—not Command Center first viewport.

| Type | Use |
| --- | --- |
| KPI numeral | Single metric |
| Line | Trends |
| Bar | Comparisons |
| Heatmap | COR elements / site risk |
| Table | Precise values |

Rules: show as-of freshness; tooltip accessible; no pie charts for many categories; reduce motion disables animated draws.

---

### 11.7 Notifications (UI)

| Type | Behavior |
| --- | --- |
| **Toast** | Transient confirmations; max 1–2 visible |
| **Inline callout** | Persistent page warnings |
| **Inbox row** | Notification center |
| **Badge** | Actionable count only |

Critical: stronger border/icon; never toast-only for safety-critical—pair with inbox.

---

### 11.8 Loading

| Pattern | When |
| --- | --- |
| **Skeleton** | First load of lists/Place overview |
| **Spinner** | Inline button / short waits |
| **Progress deterministic** | Uploads, sync drain known count |
| **Sync Pill** | Offline queue |

Prefer skeletons shaped like content (rows, header)—not generic gray blobs without structure.

---

### 11.9 Skeletons

- Match final layout geometry.  
- Subtle shimmer; disabled under reduced motion (static pulse opacity).  
- Dark theme skeletons use surface-elevated delta—not bright flash.

---

### 11.10 Empty States

Anatomy:

1. Icon (muted)  
2. Plain-language title  
3. One sentence  
4. One primary CTA  

Example: “Nothing needs you on Harbour Bridge West.” → Switch project.

No illustrations that compete with brand; no humor that undermines safety context.

---

### 11.11 Error States

| Level | Treatment |
| --- | --- |
| Field | Inline error |
| Section | Callout |
| Page | Error State layout + Retry |
| Sync conflict | Explain + Refresh / Keep server |

Show correlation id under “Details” for support—not in primary message.

---

### 11.12 Navigation Components

- **Rail** — desktop primary  
- **Tab Bar** — mobile five tabs  
- **Place Subnav** — horizontal scroll if needed  
- **Project Switcher** — combobox pattern accessible  

---

### 11.13 Status Chips & Proof Seal

**Chip:** label + optional icon; soft bg; never sole meaning carrier.

**Proof Seal:** distinct treatment for sealed evidence—checkmark + “Sealed”/“Proven” + teal proof token; animate once on completion (see Motion).

---

### 11.14 Search

- Combobox with results groups  
- Keyboard navigate  
- Recent searches  
- Mobile Find: large input, type filters as text toggles (not pill fashion overload)

---

## 12. Accessibility

| Requirement | System rule |
| --- | --- |
| Contrast | AA minimum; Site HC targets AAA where practical |
| Focus | Visible `focus.ring` all interactive elements |
| Hit target | ≥44×44 field mode |
| Color | Status + text + icon |
| Keyboard | Full desktop paths |
| SR | Landmarks: banner/nav/main; live regions for toasts/sync |
| Forms | Labels always |
| Motion | Reduced-motion paths |
| Zoom | Usable at 200% |

---

## 13. Motion & Animation

### 13.1 Signature Motions (Only These by Default)

1. **Queue completion** — item soft-fade/collapse when done (`fast`).  
2. **Proof seal** — brief seal stamp/check (`normal`–`slow`); no bounce.  
3. **Sync state** — Sync Pill subtle indeterminate while syncing (`linear` pulse).  

Optional: drawer slide (`fast`); page crossfade minimal.

### 13.2 Forbidden

- Parallax heroes in-app  
- Confetti  
- Continuous bouncing CTAs  
- Skeleton shimmer if reduced motion  
- Chart animations longer than `normal`  

---

## 14. Responsive Behavior Summary

| Pattern | Mobile | Tablet | Desktop |
| --- | --- | --- | --- |
| Nav | Tab bar | Tab or slim rail | Rail + top bar |
| Primary CTA | Sticky bottom | Sticky or inline | Inline header/footer |
| Tables | Stacked | Horizontal scroll or stack | Full table |
| Review | Full screen / sheet | Sheet | Drawer |
| Density | Field | Mixed | Office |
| Analytics | Limited / link out | Partial | Full dashboards |
| Admin | Redirect/warn | Limited | Full |

---

## 15. Content Style (UI Copy)

- Sentence case labels  
- Verbs users do on site  
- Prefer “Sealed” / “Needs signature” / “Gap: WHMIS expired”  
- Avoid jargon without context  
- Truncate with expand; don’t hide critical status  

---

## 16. Component Library Inventory

**Foundation primitives:** Button, IconButton, Input, Textarea, Select, Checkbox, Radio, Switch, Label, HelperText, Spinner, Skeleton, Avatar, Badge, Chip, Divider, Tooltip, Popover, DropdownMenu, Dialog, Sheet/Drawer, Tabs, Breadcrumb (rare), Toast/Sonner-style.

**Proven patterns:** QueueItem, ProofSeal, SyncPill, EligibilityPill, EvidencePanel, PlaceHeader, EmptyState, ErrorState, StatusCallout, DataTable, FormField, WizardShell, SearchCombobox, NotificationInboxRow, ProjectSwitcher, RailNav, MobileTabBar, MetricKpi (analytics only), ChartFrame.

**Implementation mapping:** primitives ≈ shadcn/ui in `packages/ui`, restyled to tokens; patterns in `apps/web/components` + features.

---

## 17. Theme Matrix

| Token group | Light | Dark | Site HC |
| --- | --- | --- | --- |
| Canvas | Day slate-gray | Deep slate | Max contrast pair |
| Surface | White lift | Elevated slate | Flat high contrast |
| Proof | Teal sealed | Teal adjusted for dark AA | High contrast proof |
| Overdue | Restrained alarm | Adjusted alarm | Strong alarm + label |
| Focus | Clear ring | Clear ring | Thicker ring |
| Atmosphere | Subtle texture optional | Subtle | Off |

---

## 18. Do / Don’t

| Do | Don’t |
| --- | --- |
| One primary button | Three competing primaries |
| Lists & timelines | KPI card mosaics on Home |
| Seal proof clearly | Green checkbox as only cue |
| Large field targets | Tiny ghost links for Start |
| Tokens for color | Hardcoded hex in features |
| Lucide icons | Mixed icon packs |
| Calm motion | Decorative bounce |
| Brand wordmark on entry | Generic empty logo mark only |

---

## 19. Governance

1. New components require token usage review.  
2. Status colors cannot be repurposed for marketing.  
3. Exceptions documented in ADR/design critique.  
4. Figma library (future) mirrors token names 1:1 with code tokens.  
5. Accessibility regression blocks release of primary flows.

---

## 20. Success Criteria

The design system succeeds when:

1. A worker can complete My Actions in bright sun without squinting.  
2. Sealed proof is unmistakable in light and dark.  
3. Desktop admin feels dense and professional without ERP gloom.  
4. The UI is recognizably Proven without purple SaaS clichés.  
5. Motion clarifies seal/sync/completion—and nothing else.  
6. Tokens enable tenant branding without breaking compliance semantics.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Senior Product Design | Complete Proven Design System (documentation only) |

---

*End of Proven Design System*
