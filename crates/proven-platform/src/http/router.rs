use axum::middleware;
use axum::routing::get;
use axum::Router;
use proven_config::Environment;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::http::db::{db_health, db_version};
use crate::http::docs::redoc;
use crate::http::health::{api_health, health, healthz, readyz};
use crate::http::metrics::metrics;
use crate::http::middleware::{
    correlation_layer, http_metrics_layer, propagate_request_id_layer, set_request_id_layer,
    trace_layer,
};
use crate::openapi::ApiDoc;
use crate::state::AppState;

/// Compose the foundation HTTP router (no domain modules).
pub fn build_router(state: AppState) -> Router {
    let environment = state.config().environment;

    let api = Router::new()
        .route("/health", get(health))
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/metrics", get(metrics))
        .route("/redoc", get(redoc))
        .route("/api/v1/health", get(api_health))
        .route("/api/v1/health/db", get(db_health))
        .route("/api/v1/db/version", get(db_version))
        .layer(middleware::from_fn(http_metrics_layer))
        .layer(trace_layer())
        .layer(middleware::from_fn(correlation_layer))
        .layer(propagate_request_id_layer())
        .layer(set_request_id_layer())
        .with_state(state);

    let mut app = Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(api);

    if !matches!(environment, Environment::Production) {
        app = app.layer(CorsLayer::permissive());
    }

    app
}
