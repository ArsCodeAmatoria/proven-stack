use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use proven_config::load_from_iter;
use proven_platform::{build_app, AppState};
use tower::ServiceExt;

#[tokio::test]
async fn health_returns_200() {
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
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(body.contains("\"status\":\"ok\""));
}
