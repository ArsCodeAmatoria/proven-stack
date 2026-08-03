//! Request / correlation ID helpers (AuthN-agnostic).

use http::HeaderMap;
use uuid::Uuid;

pub const REQUEST_ID_HEADER: &str = "x-request-id";
pub const CORRELATION_ID_HEADER: &str = "x-correlation-id";

/// Read `x-correlation-id`, falling back to `x-request-id`.
pub fn correlation_id_from_headers(headers: &HeaderMap) -> Option<String> {
    header_value(headers, CORRELATION_ID_HEADER)
        .or_else(|| header_value(headers, REQUEST_ID_HEADER))
}

/// Ensure a correlation id exists: prefer inbound header, else generate UUID.
pub fn ensure_correlation_id(headers: &HeaderMap) -> String {
    correlation_id_from_headers(headers).unwrap_or_else(|| Uuid::new_v4().to_string())
}

fn header_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}
