//! REST convention smoke tests (ADR-0013).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use proven_config::load_from_iter;
use proven_platform::{build_app, AppState};
use tower::ServiceExt;

#[tokio::test]
async fn health_sets_api_version_header() {
    let config = load_from_iter([("PROVEN_ENV", "development")]).unwrap();
    let app = build_app(AppState::for_tests(config));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("x-api-version").and_then(|v| v.to_str().ok()),
        Some("v1")
    );
}

#[tokio::test]
async fn versioned_openapi_is_served() {
    let config = load_from_iter([("PROVEN_ENV", "development")]).unwrap();
    let app = build_app(AppState::for_tests(config));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(body.contains("\"openapi\""));
    assert!(body.contains("bearerAuth") || body.contains("Bearer"));
}

#[tokio::test]
async fn enforce_authn_rejects_unauthenticated_api() {
    let config = load_from_iter([
        ("PROVEN_ENV", "development"),
        ("PROVEN_ENFORCE_AUTHN", "true"),
        ("PROVEN_RATE_LIMIT_ENABLED", "false"),
    ])
    .unwrap();
    let app = build_app(AppState::for_tests(config));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/projects")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["error"]["code"], "unauthorized");
}
