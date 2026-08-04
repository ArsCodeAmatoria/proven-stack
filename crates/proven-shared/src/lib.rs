//! Shared kernel for Proven — IDs, errors, and REST API conventions.
//! No business rules.

mod api;
mod error;
mod ids;

pub use api::{
    parse_multi_value, require_known_filters, require_non_empty, require_uuid, CursorPage,
    CursorPageRequest, DataEnvelope, ErrorBody, ErrorResponse, ListEnvelope, ListQuery,
    OffsetPage, OffsetPageRequest, Page, PageRequest, PaginationMeta, ProblemDetails,
    SortDirection, SortField, ValidationReport, API_VERSION, API_VERSION_HEADER, API_V1_PREFIX,
    CURRENT_API_VERSION, DEFAULT_PAGE_LIMIT, ERROR_DOC_BASE_URL, MAX_PAGE_LIMIT,
};
pub use error::{AppError, FieldError};
pub use ids::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub service: String,
}
