//! HTTP surface, versioned under `/api/v1/core/*` (CORE_DOMAIN.md §13.2, §22).

use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::api::handlers;
use crate::CoreModule;

/// Build the Core HTTP router. Callers `.merge()` this into the platform host router.
pub fn router(module: CoreModule) -> Router {
    Router::new()
        .route("/api/v1/core/tenants", post(handlers::provision_tenant))
        .route("/api/v1/core/tenants/{id}", get(handlers::get_tenant))
        .route("/api/v1/core/companies", post(handlers::register_company))
        .route("/api/v1/core/companies/{id}", get(handlers::get_company))
        .route("/api/v1/core/users/invite", post(handlers::invite_user))
        .route("/api/v1/core/users/{id}", get(handlers::get_user))
        .route("/api/v1/core/grants", post(handlers::grant_access))
        .route("/api/v1/core/grants/{id}", delete(handlers::revoke_access))
        .route("/api/v1/core/authz/authorize", post(handlers::authorize))
        .route(
            "/api/v1/core/authz/overrides",
            post(handlers::upsert_permission_override).get(handlers::list_permission_overrides),
        )
        .route(
            "/api/v1/core/authz/overrides/{id}",
            delete(handlers::revoke_permission_override),
        )
        .route("/api/v1/core/roles", get(handlers::list_system_roles))
        .route(
            "/api/v1/core/memberships",
            post(handlers::grant_project_membership),
        )
        .route(
            "/api/v1/core/memberships/projects/{project_id}",
            get(handlers::is_project_member),
        )
        .route("/api/v1/core/teams", post(handlers::create_team))
        .route(
            "/api/v1/core/audit",
            post(handlers::append_audit).get(handlers::search_audit),
        )
        .route(
            "/api/v1/core/audit/exports",
            post(handlers::request_audit_export),
        )
        .route(
            "/api/v1/core/audit/exports/{id}",
            get(handlers::get_audit_export),
        )
        .route(
            "/api/v1/core/audit/retention-policy",
            get(handlers::get_audit_retention_policy).put(handlers::put_audit_retention_policy),
        )
        .route(
            "/api/v1/core/settings",
            put(handlers::upsert_setting).get(handlers::get_setting),
        )
        .route("/api/v1/core/flags/evaluate", post(handlers::evaluate_flag))
        .route(
            "/api/v1/core/licenses/current",
            get(handlers::get_current_license),
        )
        .route(
            "/api/v1/core/files/upload-intents",
            post(handlers::create_upload_intent),
        )
        .route(
            "/api/v1/core/files/shares/{token}",
            get(handlers::resolve_public_share_link),
        )
        .route(
            "/api/v1/core/files/{id}",
            get(handlers::get_file).delete(handlers::soft_delete_file),
        )
        .route(
            "/api/v1/core/files/{id}/complete",
            post(handlers::complete_upload),
        )
        .route(
            "/api/v1/core/files/{id}/versions",
            get(handlers::list_file_versions),
        )
        .route(
            "/api/v1/core/files/{id}/metadata",
            put(handlers::update_file_metadata),
        )
        .route(
            "/api/v1/core/files/{id}/download-link",
            post(handlers::create_private_download_link),
        )
        .route(
            "/api/v1/core/files/{id}/share-links",
            post(handlers::create_public_share_link),
        )
        .route(
            "/api/v1/core/files/{id}/scan-result",
            post(handlers::apply_scan_result),
        )
        .with_state(module)
}
