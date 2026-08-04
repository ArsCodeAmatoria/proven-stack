//! API versioning constants (URI + response header).

/// Current public API major version segment.
pub const API_VERSION: &str = "v1";

/// URI prefix for versioned REST routes.
pub const API_V1_PREFIX: &str = "/api/v1";

/// Response header advertising the API version serving the request.
pub const API_VERSION_HEADER: &str = "x-api-version";

/// Semver-ish document version for OpenAPI `info.version` alignment.
pub const CURRENT_API_VERSION: &str = "1.0.0";
