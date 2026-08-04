//! Ownership boundary for the Projects Place module (ADR-0009).
//!
//! ## The boundary
//!
//! **Projects** (this crate) is the System of Record for the **Project Place** aggregate:
//! code, name, lifecycle status, primary location, and company participants (prime contractor,
//! client, and future subcontractors). It mints `ProjectId`.
//!
//! **Core** remains the System of Record for **project membership ACL** (who may access the
//! project as a principal/person) and AuthZ decisions. Projects orchestrates membership via
//! `MembershipApi`; it never stores a competing ACL table.
//!
//! ## Responsibilities (skeleton vs deferred)
//!
//! | Responsibility | Skeleton status |
//! | --- | --- |
//! | Project lifecycle (create / update / archive) | **Implemented** |
//! | Location (primary site) | **Implemented** (embedded; areas deferred) |
//! | Prime Contractor / Client | **Implemented** (participants) |
//! | Status | **Implemented** (`ProjectStatus`) |
//! | Workers | **Orchestrated** via Core membership (no People SoR) |
//! | Equipment | Deferred — Equipment module owns assets/assignments |
//! | Safety | Deferred — no safety features, inspections, or forms here |
//! | Documents | Deferred — Documents module owns binaries/versions; links later |
//! | Settings | Deferred — schema placeholder only; no settings API yet |

/// Human-readable restatement of the boundary.
pub const OWNERSHIP_NOTE: &str = "\
Projects (this crate) owns the Project Place aggregate: code, name, status, primary location, \
prime contractor, and client participants; it mints ProjectId. \
\
Core owns project membership ACL and AuthZ; Projects orchestrates GrantProjectMembership and \
never stores a competing ACL. \
\
Out of scope for this skeleton — never implemented here: Safety activities/inspections/forms, \
Equipment assignment authority, Documents binaries/versions, full Settings API, areas, \
required controls, templates, and dashboard projections.";

/// Modules / capabilities this crate must not implement in the skeleton.
pub const FORBIDDEN_IN_SKELETON: &[&str] = &[
    "safety",
    "inspections",
    "forms",
    "equipment_assignment",
    "document_binaries",
    "people_workforce",
];
