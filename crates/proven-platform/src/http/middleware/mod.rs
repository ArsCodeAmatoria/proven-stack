//! HTTP middleware stack (request-id, correlation-id, tracing, metrics, AuthN/AuthZ, rate limit).

mod api_version;
mod authn;
mod authz;
mod correlation;
mod http_metrics;
mod rate_limit;

use axum::http::HeaderName;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::{DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

pub use api_version::api_version_layer;
pub use authn::{authentication_layer, AuthnPolicy};
pub use authz::{require_permission, AuthzPrincipal};
pub use correlation::{correlation_layer, CorrelationId};
pub use http_metrics::http_metrics_layer;
pub use rate_limit::{rate_limit_layer, RateLimitState};

pub fn request_id_header() -> HeaderName {
    HeaderName::from_static("x-request-id")
}

pub fn set_request_id_layer() -> SetRequestIdLayer<MakeRequestUuid> {
    SetRequestIdLayer::new(request_id_header(), MakeRequestUuid)
}

pub fn propagate_request_id_layer() -> PropagateRequestIdLayer {
    PropagateRequestIdLayer::new(request_id_header())
}

pub fn trace_layer() -> TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    impl Fn(&axum::http::Request<axum::body::Body>) -> tracing::Span + Clone,
> {
    TraceLayer::new_for_http()
        .make_span_with(|request: &axum::http::Request<_>| {
            let request_id = request
                .headers()
                .get("x-request-id")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("-");
            let correlation_id = request
                .headers()
                .get("x-correlation-id")
                .and_then(|v| v.to_str().ok())
                .unwrap_or(request_id);
            tracing::info_span!(
                "http.request",
                method = %request.method(),
                uri = %request.uri().path(),
                request_id = %request_id,
                correlation_id = %correlation_id,
            )
        })
        .on_request(DefaultOnRequest::new().level(Level::DEBUG))
        .on_response(DefaultOnResponse::new().level(Level::INFO))
        .on_failure(DefaultOnFailure::new().level(Level::ERROR))
}
