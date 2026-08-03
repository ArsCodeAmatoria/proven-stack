# Proven — Authentication Architecture

| Field | Value |
| --- | --- |
| **Product** | Proven |
| **Document type** | Authentication Architecture |
| **Version** | 1.0 |
| **Status** | Draft |
| **Owner** | Security Architecture / Identity |
| **Audience** | Security, Frontend, Backend (Core), Mobile/PWA |
| **Last updated** | 2026-08-03 |
| **Companion docs** | [Security Architecture](./SECURITY_ARCHITECTURE.md), [Core Domain](./CORE_DOMAIN.md), [REST API](./REST_API.md), [Signatures](./SIGNATURES_DOMAIN.md), [Offline Sync](./OFFLINE_SYNC_ARCHITECTURE.md), [Frontend Folders](./FRONTEND_FOLDER_STRUCTURE.md), [AGENTS.md](../../AGENTS.md) |

---

## 1. Purpose

This document designs Proven’s **authentication** system: **Better Auth** as the human AuthN framework for web/PWA, integrated with **Core** identity and AuthZ; JWT and refresh tokens; OAuth (Microsoft, Google); password reset; magic links; email verification; MFA; sessions; device tracking; remember-me; guest signing; and desktop / mobile / offline behaviors.

**Hard rules**

1. **Better Auth** handles human credential and session **AuthN** UX/protocols for the product app.  
2. **Core** remains system of record for **User/Principal**, **tenant binding**, **session revocation authority** used by the API, and **all AuthZ**.  
3. **Guest signing ≠ platform login** — Signatures package tokens never become full user sessions.  
4. **No permissions in JWT** — AuthZ via Core `AuthzApi` on every API call.  
5. Secrets, raw tokens, and TOTP seeds never appear in logs, events, or analytics.

**Documentation only — no implementation.**

---

## 2. Architectural Placement

```text
┌─────────────────────────────────────────────────────────────┐
│  Next.js (apps/web)                                         │
│  Better Auth handler routes (/api/auth/*)                   │
│  Session cookies · OAuth callbacks · MFA challenges         │
└───────────────────────────┬─────────────────────────────────┘
                            │ establish / sync identity
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  Core (proven-core)                                         │
│  User · Principal · Session ledger · MFA policy · Audit     │
│  AuthzApi (RBAC/ABAC)                                       │
└───────────────────────────┬─────────────────────────────────┘
                            │ validates access JWT / session
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  Rust API (/api/v1) · RLS tenant GUC · module commands      │
└─────────────────────────────────────────────────────────────┘

Guest sign path (separate):
  Signatures magic/QR token → guest routes only → seal slot
```

| Layer | Responsibility |
| --- | --- |
| **Better Auth** | Email/password, OAuth, magic link login, email verification, MFA plugins, session cookie issuance, refresh rotation at the app edge |
| **Core Identity** | Canonical user ids, tenant membership of accounts, server session rows, revoke-all, MFA *policy*, password hash storage policy alignment, audit |
| **Rust API middleware** | Validate access token; resolve `tid`/`sub`/`sid`; ensure session active; set RLS |
| **Signatures** | Guest/package tokens only |

### 2.1 Integration model (authoritative)

**Adapter / sync pattern:** Better Auth is configured with a **Proven Core adapter** (logical): user and session writes go through Core APIs or Core-owned tables exposed only to the auth adapter—not a second shadow user directory that drifts.

- Create/link user on first OAuth or verified email → Core `User` + tenant binding (invite or SSO domain rules).  
- Session create/refresh/revoke → Core `Session` ledger (`sid`).  
- Access JWT claims include Core `sub`, `tid`, `sid`.

If Better Auth and Core tables are unified physically, the adapter still enforces Core invariants (tenant required, soft-delete, audit).

---

## 3. Better Auth

### 3.1 Role

Better Auth is the **chosen AuthN framework** for Proven’s Next.js application:

| Capability | Better Auth |
| --- | --- |
| Email + password | Yes (when tenant policy allows local auth) |
| Session cookies | Yes (HTTP-only, Secure) |
| OAuth / social | Google, Microsoft (Azure AD / Entra ID) |
| Magic link login | Yes (auth magic link—not guest sign) |
| Email verification | Yes |
| MFA / 2FA plugins | TOTP; passkeys/WebAuthn where supported |
| Plugin surface | Organization/tenant plugins only if mapped to Core tenancy—**do not** invent a parallel org model |

### 3.2 Route surface (logical)

| Path | Purpose |
| --- | --- |
| `/api/auth/*` | Better Auth handler (sign-in, callback, session, sign-out) |
| App `(auth)/login` | UI shell calling Better Auth client |
| `/auth/me` (API) | Proven profile after session (Core-enriched) |

### 3.3 What Better Auth must not own alone

- Permission grants / RBAC  
- Project membership  
- License entitlements  
- Guest signature evidence  
- Service/API keys for integrations  

Those remain Core / Admin / Signatures.

### 3.4 Tenant resolution

On sign-in:

1. Resolve tenant from: SSO tenant mapping, email domain allowlist, active invite token, or user-selected tenant when multi-tenant membership exists.  
2. Reject sign-in if no active tenant binding (except invite acceptance flow).  
3. Bind session to exactly one **active tenant context** (`tid`); switching tenant = new session or explicit switch endpoint that re-issues tokens.

---

## 4. JWT (Access Tokens)

### 4.1 Characteristics

| Property | Spec |
| --- | --- |
| **Lifetime** | Short (e.g. 5–15 minutes) |
| **Transport** | `Authorization: Bearer` to Rust API; web may also use BFF cookie session that mint/attach Bearer server-side |
| **Alg** | Asymmetric (RS256/ES256) preferred; `kid` for rotation |
| **Issuer / audience** | Proven auth issuer / `proven-api` |

### 4.2 Claims

| Claim | Meaning |
| --- | --- |
| `sub` | Core user / principal id |
| `tid` | Active tenant id |
| `sid` | Core session id |
| `iat` / `exp` | Issued / expiry |
| `iss` / `aud` | Issuer / API audience |
| `amr` | Auth methods (e.g. `pwd`, `otp`, `oauth`, `mfa`) |
| `acr` | Assurance level (e.g. MFA satisfied) |
| `did` | Device id (optional, for tracking) |

**Excluded:** roles, permission arrays, PII dumps, email (optional minimal `email_verified` boolean only if needed).

### 4.3 Validation (API)

1. Verify signature + `iss`/`aud`/`exp`.  
2. Confirm `sid` not revoked (Core session store / deny-list).  
3. Confirm tenant active and user not deactivated.  
4. Build `RequestContext`; set RLS GUC.

---

## 5. Refresh Tokens

| Property | Spec |
| --- | --- |
| **Storage (web)** | HTTP-only Secure cookie (Better Auth session/refresh) — not `localStorage` |
| **Storage (native future)** | Secure OS storage; same rotation rules |
| **Lifetime** | Longer than access; bounded by absolute session and remember-me policy |
| **Rotation** | **Rotate on every refresh**; invalidate previous refresh (reuse detection) |
| **Binding** | Tied to `sid` + `did` (device) |
| **Reuse detection** | If revoked/old refresh presented → revoke **entire session family** + alert |
| **Revocation** | Logout, password change, MFA reset, admin revoke, compromise → refresh dead |

Refresh never authorizes API domain calls directly—only mints new access tokens after Core session check.

---

## 6. OAuth

### 6.1 Protocol

- **Authorization Code + PKCE** for browser/PWA.  
- Better Auth social/OAuth plugins for provider wiring.  
- Map external `sub` + issuer → Core `User` identity link (`IdentityProviderAccount`).

### 6.2 Microsoft

| Item | Design |
| --- | --- |
| **Provider** | Microsoft identity platform / Entra ID |
| **Tenants** | Support common + **organizational** directories; enterprise customers bring Entra tenant ids |
| **Claims used** | `oid`/`sub`, email, name, `tid` (Entra tenant) for mapping |
| **Mapping** | Entra tenant → Proven tenant (admin-configured); user provision JIT or invite-only per policy |
| **MFA** | Prefer trusting Entra MFA via `amr`/`acr` when present; tenant may still require Proven MFA for break-glass locals |

### 6.3 Google

| Item | Design |
| --- | --- |
| **Provider** | Google OAuth 2.0 / OIDC |
| **Use** | Contractor/lightweight tenants; also Google Workspace domain restrict optional |
| **Mapping** | Verified email → user; domain allowlist optional |
| **Caution** | Consumer Gmail accounts only if tenant policy allows |

### 6.4 Account linking

- Link OAuth to existing user when verified email matches and policy allows.  
- Prevent linking hijack: require authenticated session or verified email proof.  
- Audit `IdentityLinked` / `IdentityUnlinked`.

### 6.5 Enterprise SSO vs social

Enterprise may disable Google/password and allow **Microsoft only**. Tenant auth policy in Core settings drives Better Auth enabled providers.

---

## 7. Password Authentication

| Topic | Design |
| --- | --- |
| **When enabled** | Tenant policy; disabled for SSO-only tenants (break-glass exceptions audited) |
| **Hashing** | Modern KDF (Argon2id); Better Auth + Core policy alignment—single hash SoR |
| **Policy** | Per [Security Architecture](./SECURITY_ARCHITECTURE.md) password policy |
| **Lockout** | Progressive backoff; audit failures; CAPTCHA/bot at Cloudflare on abuse |

---

## 8. Password Reset

```text
Request reset (email) → rate limit → send one-time link/token (hashed at rest)
  → user opens link → verify token → set new password
  → revoke all sessions for user → require re-login
  → optional force MFA re-check if enrolled
```

| Control | Spec |
| --- | --- |
| **TTL** | Short (e.g. 15–60 minutes) |
| **Single-use** | Yes |
| **Enumeration** | Generic response (“if account exists…”) |
| **Transport** | Email via Notifications/provider; no token in query logs |
| **Better Auth** | Use built-in forget-password flow wired to Proven email templates |

---

## 9. Magic Links (Authentication)

**Auth magic links** log a user into Proven (Better Auth). Distinct from **guest sign** links (§16).

| Property | Spec |
| --- | --- |
| **TTL** | Short; single-use |
| **Rate limit** | Per email + IP (Cloudflare + API) |
| **Result** | Full session + JWT after redeem (and email verified) |
| **Tenant** | Must resolve tenant (invite or existing membership) |
| **Offline** | Cannot redeem offline; link opens when online |

---

## 10. Email Verification

| State | Behavior |
| --- | --- |
| **Unverified** | May sign up / limited access per policy; cannot become tenant admin; cannot export PII |
| **Verified** | Full access per grants |
| **Flow** | Send verification message on register or email change; one-time token; mark verified in Core |
| **OAuth** | Trust provider `email_verified` when present; still bind per tenant rules |
| **Change email** | Re-verify; notify old email; step-up auth required |

---

## 11. MFA

| Factor | Support |
| --- | --- |
| **TOTP** | Primary; encrypted secret at rest |
| **Passkeys / WebAuthn** | Preferred where device supports (Better Auth plugin) |
| **Backup codes** | One-time, hashed; shown once |

### 11.1 Policy

| Policy | Meaning |
| --- | --- |
| **Off** | Not required (discouraged for prod tenants) |
| **Privileged roles** | Admins, API key managers, exporters, COR finalize—must enroll |
| **All users** | Tenant-wide mandatory |
| **Step-up** | Recent MFA (`acr`) required for grant changes, key create, mass export |

### 11.2 Flows

1. Login password/OAuth → if MFA required and enrolled → challenge → then session.  
2. Enrollment in Settings → verify factor → audit.  
3. Recovery: backup codes or admin-assisted reset (audited); revoke sessions.

### 11.3 SSO

If Microsoft/Google assertion indicates MFA, map into `amr`/`acr`. Tenant may waive Proven TOTP when IdP MFA is sufficient.

Guest signing does **not** use platform MFA.

---

## 12. Session Management

| Property | Spec |
| --- | --- |
| **SoR** | Core `Session` row (`sid`) + Better Auth session cookie aligned |
| **Establish** | After successful AuthN (+ MFA) |
| **Idle timeout** | Tenant-configurable; shorter for privileged |
| **Absolute timeout** | Hard cap regardless of activity |
| **Logout** | Revoke `sid`; clear cookies; refresh invalidated |
| **Logout all** | On password reset, MFA reset, compromise, admin action |
| **List sessions** | Settings UI: device, approx location/IP (policy), last seen, revoke one |

Session events: `SessionEstablished`, `SessionRefreshed`, `SessionRevoked` (audit/security analytics optional).

---

## 13. Device Tracking

| Field (logical) | Use |
| --- | --- |
| `device_id` | Stable opaque id (cookie/app install id)—not a hardware fingerprint mandate |
| User-Agent / client type | desktop_web / mobile_pwa / unknown |
| Last IP | Policy-limited retention |
| Last seen | Session list |
| Trust state | new / known / remembered |

### 13.1 Behaviors

- New device → optional email notify; step-up MFA if policy.  
- Anomalous IP/UA change mid-session → step-up or soft challenge.  
- Do not use invasive fingerprinting that violates privacy policy; minimize data (PIPEDA/GDPR).  

`did` may appear in JWT for binding refresh tokens.

---

## 14. Remember Me

| Mode | Behavior |
| --- | --- |
| **Off (default on shared devices)** | Shorter absolute/idle timeouts |
| **On** | Extends refresh/session absolute lifetime within platform max; still requires MFA when policy demands at login |
| **Not a bypass** | Does not skip MFA enrollment requirements |
| **Revocation** | Password change / logout all clears remembered sessions |
| **UX** | Checkbox on login; explain shared-device risk on mobile and desktop |

Remember-me must not store access tokens in non-HTTP-only storage.

---

## 15. Desktop vs Mobile

| Concern | Desktop (Command Center) | Mobile PWA (My Actions) |
| --- | --- | --- |
| **Shell** | `(app)` rail; longer forms | Tab shell; installable PWA |
| **Cookies** | SameSite appropriate for site | Same; careful on iOS PWA storage |
| **OAuth** | Full browser redirect | System browser / in-app redirect return to PWA URL |
| **MFA** | TOTP/passkey | Prefer passkey/WebAuthn; TOTP fallback |
| **Session UX** | Session list in settings | Same; emphasize logout on shared phones |
| **Idle** | May be longer for office | Shorter default for field phones optional |
| **Biometrics** | OS passkey | OS passkey / device unlock as UX for passkeys—not a replacement for server session |

---

## 16. Guest Signing (Non-Account Auth)

| Property | Spec |
| --- | --- |
| **Owner** | Signatures module |
| **Credential** | Opaque magic link / QR session token (hashed at rest) |
| **Scope** | Single package/slot (or package) only |
| **Not issued by** | Better Auth user session |
| **Cannot** | Call global `/api/v1` as a tenant user; open Command Center |
| **Assurance** | Signer name/email capture + optional IDV per policy |
| **TTL / revoke** | Short; workflow expiry; void package |
| **UI** | `app/(auth)/guest/sign/[token]` isolated layout |

See [Signatures Domain](./SIGNATURES_DOMAIN.md) and [Temporal GuestSignatureWorkflow](./TEMPORAL_WORKFLOWS.md).

---

## 17. Offline Authentication Behavior

| Topic | Design |
| --- | --- |
| **Requirement** | Valid session to **start** offline field work; drain requires refresh if access expired |
| **Refresh** | Attempt silent refresh when online; if refresh fails → pause sync queue; prompt login |
| **Storage** | No passwords; refresh only per Security cookie/secure store policy—not plaintext in IndexedDB |
| **Mutations** | Idempotent outbox; AuthZ rechecked on sync |
| **Sealed proof** | Never claim sealed without server ACK |
| **Guest** | Guest redeem/sign generally **online** |
| **MFA step-up** | Cannot complete offline; block sensitive offline seal if policy needs step-up |

Align with [Offline Sync Architecture](./OFFLINE_SYNC_ARCHITECTURE.md).

---

## 18. End-to-End Flows (Summary)

### 18.1 Password + MFA (web)

1. User submits credentials via Better Auth client.  
2. Verify password → MFA challenge if required.  
3. Create Core session + Better Auth session cookie.  
4. Issue access JWT; refresh cookie set (remember-me adjusts TTL).  
5. API calls Bearer/cookie bridge → AuthZ.

### 18.2 Microsoft / Google OAuth

1. PKCE authorize redirect → provider → callback to Better Auth.  
2. Link/create Core user; resolve tenant.  
3. MFA per policy → session + JWT.

### 18.3 Magic link login

1. Request link → email.  
2. Redeem → verified session (email verified).  
3. Same session issuance as password path.

### 18.4 Password reset

1. Request → email token.  
2. Set password → revoke all sessions → re-login.

### 18.5 Guest seal

1. Host issues Signatures link.  
2. Guest opens token route → seal only.  
3. No Better Auth user session.

---

## 19. API Surface (Logical)

| Endpoint family | Owner |
| --- | --- |
| `/api/auth/*` | Better Auth (Next) |
| `POST /auth/logout`, refresh bridge, `/auth/me` | Proven API / BFF as documented in REST |
| Guest redeem/seal | Signatures routes |
| Service credentials | Workers—not Better Auth |

Exact path consolidation (BFF vs Rust) may use Next auth routes + Rust verification; single documented contract for mobile later.

---

## 20. Security Controls Recap

| Control | Apply |
| --- | --- |
| Rate limit | Login, reset, magic link, OAuth callback abuse |
| Cloudflare | Bot/WAF on auth paths |
| Audit | Login success/fail, MFA, reset, revoke, link identity |
| Encryption | TLS; MFA secrets encrypted; tokens hashed when stored |
| Enumeration resistance | Generic messages on reset/magic link |
| Step-up | Privileged mutations |

---

## 21. Testing Guidance

- OAuth callback success/fail and account linking  
- Refresh reuse detection  
- MFA mandatory enforcement  
- Session revoke stops API access before JWT natural expiry  
- Guest token cannot hit authenticated app APIs  
- Offline: expired refresh pauses outbox  
- Tenant isolation on sign-in  

---

## 22. Success Criteria

1. Better Auth covers password, Google, Microsoft, magic link login, email verification, MFA, and cookie sessions.  
2. Core owns user/tenant/session revocation and all AuthZ.  
3. JWT is short-lived; refresh rotates with reuse detection.  
4. Remember-me and device tracking improve UX without weakening MFA.  
5. Guest signing remains package-scoped and separate.  
6. Desktop, mobile PWA, and offline behaviors are explicit and safe.

---

## Document Control

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0 | 2026-08-03 | Security Architecture | Better Auth–centric AuthN design |

---

*End of Authentication Architecture*
