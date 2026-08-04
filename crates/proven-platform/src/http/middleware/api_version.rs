//! Advertise `X-Api-Version` on every response (URI versioning remains `/api/v1`).

use axum::extract::Request;
use axum::http::{HeaderName, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;
use proven_shared::{API_VERSION, API_VERSION_HEADER};

pub async fn api_version_layer(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    if let Ok(value) = HeaderValue::from_str(API_VERSION) {
        response.headers_mut().insert(
            HeaderName::from_static(API_VERSION_HEADER),
            value,
        );
    }
    response
}
