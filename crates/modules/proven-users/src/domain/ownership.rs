//! Ownership boundary between Core, People, and Users (ADR-0006). Documented here as constants
//! (not just prose) so the invariant is discoverable from code and can be asserted on in tests.
//!
//! ## The boundary
//!
//! **Core** (`proven-core`, `core.users`) is the System of Record for **login identity**:
//! `UserId`, invite/activate/lock lifecycle, credentials, SSO links, sessions, and AuthZ grants.
//! Core mints the stable `UserId` every other module references.
//!
//! **People** (future `proven-people`, out of scope here) is the System of Record for
//! operational **workforce `Person` profiles** — certifications, crew assignments, trade,
//! employment status. Users never becomes workforce SoR; it only stores an optional, unenforced
//! `PersonId` pointer for cross-linking once People exists.
//!
//! **Users** (this crate, `users.*` schema) is the System of Record for **account profile &
//! preferences**, keyed by that same `UserId` + `TenantId` — classifications (`UserKind`),
//! avatar, locale, accessibility, notification preferences, authentication *preference* mirror
//! flags, digital signature preferences, emergency contacts, a settings bag, and an append-only
//! profile audit log.
//!
//! ## Specific non-goals (each independently important — do not "helpfully" add these)
//!
//! - **`UserKind` ≠ Core RBAC ≠ People workforce roles.** `UserKind` (Worker, Supervisor,
//!   Manager, SafetyCoordinator, Administrator, External, Guest) is a profile tag for UX/directory
//!   filtering only. It is never consulted by an AuthZ decision — every permission check still
//!   flows exclusively through `proven_core::AuthzApi` (ADR-0003). It is also not a People
//!   workforce role (trade, crew position, certification-gated title) — those live in People.
//! - **Guest classification ≠ guest signing tokens.** A `UserKind::Guest` profile is a shell for
//!   a guest-capable principal to hold preferences against. The short-lived tokens used to let an
//!   unauthenticated guest sign a document belong to the (future) Signatures module and are never
//!   created, stored, or validated here.
//! - **No project assignments.** This module never creates, reads, updates, or deletes a
//!   `ProjectMembership` or any project-scoped assignment view. That is Core
//!   (`ProjectMembership`) today and may grow People-side assignment views later — never Users.
//! - **Authentication profile never stores password hashes.** `AuthenticationProfile` holds
//!   *preference* mirror flags only (MFA preference, last auth method, OAuth link booleans). It
//!   is not a second credential store; Core (`proven_core::domain::User`/`Session`) remains the
//!   sole authentication SoR.
//! - **Digital signature profile ≠ signature packages.** `DigitalSignatureProfile` stores a
//!   user's signing *preferences and assurance hints* (default signature type, typed name
//!   default, re-auth requirement). It never stores an executed signature package or document
//!   binding — that is Signatures' job.
//! - **Profile audit ≠ Core audit.** `ProfileAuditEntry` is an append-only log of *this module's*
//!   profile mutations for fast, module-local history views. It is not a second security SoR and
//!   never replaces `proven_core::AuditApi`.
//!
//! If a future change requires this module to reach into Core, People, Projects, or Signatures
//! data, that is a signal the design is wrong — talk to that module's public API/trait instead
//! (or emit/consume an event), never its schema.

/// Human-readable restatement of the boundary above, for logs/docs that want a single string.
pub const OWNERSHIP_NOTE: &str = "\
Core (proven-core) owns login identity: UserId, invite/activate/lock lifecycle, credentials, SSO \
links, sessions, and AuthZ grants. \
\
People (future proven-people) owns operational workforce Person profiles — Users never becomes \
workforce SoR, it only stores an optional, unenforced PersonId pointer. \
\
Users (this crate) owns ACCOUNT PROFILE & PREFERENCES keyed by UserId + TenantId: UserKind \
classification tags, avatar, locale, accessibility, notification preferences, authentication \
preference mirror flags (never password hashes), digital signature preferences (never signature \
packages), emergency contacts, a settings bag, and an append-only profile audit log. \
\
Out of scope — never created, owned, or mutated here: project assignments, People workforce SoR, \
Core RBAC/AuthZ decisions, password hashes/credentials, signature packages, and guest signing \
tokens.";

/// Business modules / concerns that this crate is explicitly forbidden from implementing or
/// depending on.
pub const FORBIDDEN_MODULES: &[&str] = &[
    "projects",
    "people",
    "workforce",
    "equipment",
    "documents",
    "training",
    "safety",
    "signatures",
    "cor_audit",
];
