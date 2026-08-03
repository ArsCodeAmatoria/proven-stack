//! Correlation ID middleware — generates or forwards `x-correlation-id`.

use axum::extract::Request;
use axum::http::{HeaderName, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;
use proven_observability::{ensure_correlation_id, CORRELATION_ID_HEADER};
use uuid::Uuid;

/// Extension holding the active correlation id for handlers.
#[derive(Clone, Debug)]
pub struct CorrelationId(pub String);

impl CorrelationId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub async fn correlation_layer(mut request: Request, next: Next) -> Response {
    let correlation_id = ensure_correlation_id(request.headers());

    if request.headers().get(CORRELATION_ID_HEADER).is_none() {
        if let Ok(value) = HeaderValue::from_str(&correlation_id) {
            request
                .headers_mut()
                .insert(HeaderName::from_static(CORRELATION_ID_HEADER), value);
        }
    }

    // Prefer inbound request-id; if absent, mirror correlation id for continuity.
    if request.headers().get("x-request-id").is_none() {
        if let Ok(value) = HeaderValue::from_str(&correlation_id) {
            request
                .headers_mut()
                .insert(HeaderName::from_static("x-request-id"), value);
        } else if let Ok(value) = HeaderValue::from_str(&Uuid::new_v4().to_string()) {
            request
                .headers_mut()
                .insert(HeaderName::from_static("x-request-id"), value);
        }
    }

    let ext = CorrelationId(correlation_id.clone());
    tracing::debug!(correlation_id = %ext.as_str(), "correlation id bound");
    request.extensions_mut().insert(ext);

    let mut response = next.run(request).await;
    if let Ok(value) = HeaderValue::from_str(&correlation_id) {
        response
            .headers_mut()
            .insert(HeaderName::from_static(CORRELATION_ID_HEADER), value);
    }
    response
}
