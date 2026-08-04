//! REST API conventions — wire shapes shared by every module (ADR-0013).
//!
//! No Axum dependency. Handlers build these types; the platform maps them to HTTP.

mod envelope;
mod error;
mod list;
mod paging;
mod validation;
mod versioning;

pub use envelope::{DataEnvelope, ListEnvelope, PaginationMeta};
pub use error::{ErrorBody, ErrorResponse, ProblemDetails, ERROR_DOC_BASE_URL};
pub use list::{
    parse_multi_value, require_known_filters, ListQuery, SortDirection, SortField,
};
pub use paging::{
    CursorPage, CursorPageRequest, OffsetPage, OffsetPageRequest, DEFAULT_PAGE_LIMIT,
    MAX_PAGE_LIMIT,
};
pub use validation::{require_non_empty, require_uuid, ValidationReport};
pub use versioning::{API_VERSION, API_VERSION_HEADER, API_V1_PREFIX, CURRENT_API_VERSION};

// Backward-compatible aliases used across domain repositories.
pub use paging::{OffsetPage as Page, OffsetPageRequest as PageRequest};
