//! Ownership boundary between Core and Companies (ADR-0005). Documented here as constants (not
//! just prose) so the invariant is discoverable from code and can be asserted on in tests.
//!
//! ## The boundary
//!
//! **Core** (`proven-core`, `core.companies`) is the System of Record for a Company's *legal
//! identity*: `legal_name`, `display_name`, `company_type`, lifecycle `status`
//! (active/deactivated), and tenancy/AuthZ scoping. Core also mints the stable `CompanyId` every
//! other module references.
//!
//! **Companies** (this crate, `companies.*` schema) is the System of Record for a Company's
//! *profile & configuration*, keyed by that same `CompanyId` + `TenantId` — business units,
//! addresses, contacts, branding, safety/regional defaults, default template pointers,
//! notification defaults, and storage configuration.
//!
//! A Core `Company` **logically owns** (via `CompanyId` foreign references) Projects, Workers,
//! Equipment, Documents, Training, and Safety resources — but those are owned and implemented by
//! their own modules (see `docs/architecture/DOMAIN_MODULES_OVERVIEW.md`). This module:
//!
//! - **never** creates, reads, updates, or deletes rows in `projects.*`, `people.*`,
//!   `equipment.*`, `documents.*`, `training.*`, or `safety.*`;
//! - **never** depends on any business-module crate;
//! - only ever *references* those domains indirectly through opaque pointers (e.g. a
//!   `DefaultTemplate.template_file_id`), which it never dereferences or interprets.
//!
//! If a future change requires this module to reach into another business module's data, that
//! is a signal the design is wrong — talk to that module's public API/trait instead (or emit/
//! consume an event), never its schema.

/// Human-readable restatement of the boundary above, for logs/docs that want a single string.
pub const OWNERSHIP_NOTE: &str = "\
Core (proven-core) owns Company legal identity: legal_name, display_name, company_type, and \
lifecycle status (active/deactivated); it mints CompanyId. \
\
Companies (this crate) owns PROFILE & CONFIGURATION keyed by CompanyId + TenantId: business \
units, addresses, contacts, branding, safety settings, regional settings, default template \
pointers, notification defaults, and storage configuration. \
\
Out of scope — never created, owned, or mutated here: Projects, Workers/People, Equipment, \
Documents, Training, and Safety incident/inspection resources. Those modules key off CompanyId \
but are implemented elsewhere and are out of scope for this crate.";

/// Business modules that this crate is explicitly forbidden from implementing or depending on.
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
