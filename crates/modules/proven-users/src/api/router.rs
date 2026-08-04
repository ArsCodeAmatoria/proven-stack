//! HTTP surface, versioned under `/api/v1/users/*` (ADR-0006 §3).

use axum::routing::{get, post};
use axum::Router;

use crate::api::handlers;
use crate::UsersModule;

/// Build the Users HTTP router. Callers `.merge()` this into the platform host router.
pub fn router(module: UsersModule) -> Router {
    Router::new()
        .route(
            "/api/v1/users/{user_id}/profile/ensure",
            post(handlers::ensure_profile),
        )
        .route(
            "/api/v1/users/{user_id}/profile",
            get(handlers::get_profile).patch(handlers::update_profile),
        )
        .route(
            "/api/v1/users/{user_id}/profile/archive",
            post(handlers::archive_profile),
        )
        .route(
            "/api/v1/users/{user_id}/kinds",
            get(handlers::list_kinds).post(handlers::assign_kind),
        )
        .route(
            "/api/v1/users/{user_id}/kinds/{kind}",
            axum::routing::delete(handlers::remove_kind),
        )
        .route(
            "/api/v1/users/{user_id}/avatar",
            get(handlers::get_avatar).put(handlers::upsert_avatar),
        )
        .route(
            "/api/v1/users/{user_id}/locale",
            get(handlers::get_locale).put(handlers::upsert_locale),
        )
        .route(
            "/api/v1/users/{user_id}/accessibility",
            get(handlers::get_accessibility).put(handlers::upsert_accessibility),
        )
        .route(
            "/api/v1/users/{user_id}/notification-preferences",
            get(handlers::get_notification_preferences)
                .put(handlers::upsert_notification_preferences),
        )
        .route(
            "/api/v1/users/{user_id}/authentication",
            get(handlers::get_authentication_profile).put(handlers::upsert_authentication_profile),
        )
        .route(
            "/api/v1/users/{user_id}/signature-profile",
            get(handlers::get_signature_profile).put(handlers::upsert_signature_profile),
        )
        .route(
            "/api/v1/users/{user_id}/emergency-contacts",
            get(handlers::list_emergency_contacts).post(handlers::add_emergency_contact),
        )
        .route(
            "/api/v1/users/{user_id}/emergency-contacts/{contact_id}",
            axum::routing::patch(handlers::update_emergency_contact)
                .delete(handlers::remove_emergency_contact),
        )
        .route(
            "/api/v1/users/{user_id}/settings/{key}",
            get(handlers::get_setting).put(handlers::upsert_setting),
        )
        .route(
            "/api/v1/users/{user_id}/audit-history",
            get(handlers::list_audit_history),
        )
        .with_state(module)
}
