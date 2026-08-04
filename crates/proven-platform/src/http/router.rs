use axum::Extension;
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
    api_version_layer, authentication_layer, correlation_layer, http_metrics_layer,
    propagate_request_id_layer, rate_limit_layer, set_request_id_layer, trace_layer, AuthnPolicy,
    RateLimitState,
};
use crate::http::temporal::temporal_health;
use crate::openapi::ApiDoc;
use crate::openapi::openapi_json;
use crate::state::AppState;

/// Compose the HTTP router: platform host + Core + Companies + Users + Projects.
///
/// Cross-cutting REST conventions (ADR-0013) apply to the merged app: API version header,
/// AuthN credential gate (when enabled), rate limits, correlation, metrics, and tracing.
pub fn build_router(state: AppState) -> Router {
    let environment = state.config().environment;
    let api = &state.config().api;
    let rate_limit = RateLimitState::new(api.rate_limit_per_minute, api.rate_limit_enabled);
    let authn = AuthnPolicy {
        enforce_credentials: api.enforce_authn,
    };

    let core = state.core().clone();
    let companies = state.companies().clone();
    let users = state.users().clone();
    let projects = state.projects().clone();

    let platform = Router::new()
        .route("/health", get(health))
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/metrics", get(metrics))
        .route("/redoc", get(redoc))
        .route("/api/v1/health", get(api_health))
        .route("/api/v1/health/db", get(db_health))
        .route("/api/v1/health/temporal", get(temporal_health))
        .route("/api/v1/db/version", get(db_version))
        .route("/api/v1/openapi.json", get(openapi_json))
        .with_state(state);

    let mut app = Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(platform)
        .merge(core.router())
        .merge(companies.router())
        .merge(users.router())
        .merge(projects.router())
        .layer(Extension(rate_limit))
        .layer(Extension(authn))
        .layer(middleware::from_fn(rate_limit_layer))
        .layer(middleware::from_fn(authentication_layer))
        .layer(middleware::from_fn(api_version_layer))
        .layer(middleware::from_fn(http_metrics_layer))
        .layer(trace_layer())
        .layer(middleware::from_fn(correlation_layer))
        .layer(propagate_request_id_layer())
        .layer(set_request_id_layer());

    if !matches!(environment, Environment::Production) {
        app = app.layer(CorsLayer::permissive());
    }

    app
}
