//! HTTP surface, versioned under `/api/v1/companies/*` (ADR-0005 §4).

use axum::routing::{get, post};
use axum::Router;

use crate::api::handlers;
use crate::CompaniesModule;

/// Build the Companies HTTP router. Callers `.merge()` this into the platform host router.
pub fn router(module: CompaniesModule) -> Router {
    Router::new()
        .route(
            "/api/v1/companies/{company_id}/profile/ensure",
            post(handlers::ensure_profile),
        )
        .route(
            "/api/v1/companies/{company_id}/profile",
            get(handlers::get_profile).patch(handlers::update_profile),
        )
        .route(
            "/api/v1/companies/{company_id}/profile/archive",
            post(handlers::archive_profile),
        )
        .route(
            "/api/v1/companies/{company_id}/business-units",
            get(handlers::list_business_units).post(handlers::create_business_unit),
        )
        .route(
            "/api/v1/companies/{company_id}/business-units/{unit_id}",
            axum::routing::patch(handlers::update_business_unit),
        )
        .route(
            "/api/v1/companies/{company_id}/business-units/{unit_id}/archive",
            post(handlers::archive_business_unit),
        )
        .route(
            "/api/v1/companies/{company_id}/addresses",
            get(handlers::list_addresses).post(handlers::add_address),
        )
        .route(
            "/api/v1/companies/{company_id}/addresses/{address_id}",
            axum::routing::patch(handlers::update_address).delete(handlers::remove_address),
        )
        .route(
            "/api/v1/companies/{company_id}/contacts",
            get(handlers::list_contacts).post(handlers::add_contact),
        )
        .route(
            "/api/v1/companies/{company_id}/contacts/{contact_id}",
            axum::routing::patch(handlers::update_contact).delete(handlers::remove_contact),
        )
        .route(
            "/api/v1/companies/{company_id}/branding",
            get(handlers::get_branding).put(handlers::upsert_branding),
        )
        .route(
            "/api/v1/companies/{company_id}/safety-settings",
            get(handlers::get_safety_settings).put(handlers::upsert_safety_settings),
        )
        .route(
            "/api/v1/companies/{company_id}/regional-settings",
            get(handlers::get_regional_settings).put(handlers::upsert_regional_settings),
        )
        .route(
            "/api/v1/companies/{company_id}/default-templates",
            get(handlers::list_default_templates).put(handlers::upsert_default_template),
        )
        .route(
            "/api/v1/companies/{company_id}/notification-defaults",
            get(handlers::get_notification_defaults).put(handlers::upsert_notification_defaults),
        )
        .route(
            "/api/v1/companies/{company_id}/storage",
            get(handlers::get_storage_configuration).put(handlers::upsert_storage_configuration),
        )
        .with_state(module)
}
