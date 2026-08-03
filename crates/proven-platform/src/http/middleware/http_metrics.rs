//! Low-cardinality HTTP RED metrics.

use std::time::Instant;

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use metrics::{counter, histogram};

pub async fn http_metrics_layer(request: Request, next: Next) -> Response {
    let method = request.method().as_str().to_owned();
    let started = Instant::now();
    let response = next.run(request).await;
    let status = response.status().as_u16();
    let status_class = match status {
        100..=199 => "1xx",
        200..=299 => "2xx",
        300..=399 => "3xx",
        400..=499 => "4xx",
        _ => "5xx",
    };

    counter!(
        "http_server_requests_total",
        "method" => method.clone(),
        "status_class" => status_class
    )
    .increment(1);

    histogram!(
        "http_server_request_duration_seconds",
        "method" => method,
        "status_class" => status_class
    )
    .record(started.elapsed().as_secs_f64());

    response
}
