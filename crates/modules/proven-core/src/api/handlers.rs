//! Axum HTTP handlers — thin adapters over `CoreServices`. All business rules live in
//! `application::services`; handlers only parse/validate transport concerns and map errors.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use proven_shared::{
    AppError, CausationId, CompanyId, CorrelationId, FeatureFlagKey, FileObjectId, GrantId,
    PageRequest, PersonId, ProblemDetails, ProjectId, RoleId, SettingKey, TenantId, UserId,
};
use uuid::Uuid;

use crate::api::dto::{
    AppendAuditRequest, ApplyScanResultRequest, AuditSearchHttpQuery, AuthorizeHttpRequest,
    CompleteFileUploadRequest, CreateFileUploadIntentRequest, CreatePublicShareLinkRequest,
    CreateTeamRequest, EvaluateFlagRequest, EvaluateFlagResponse, GetSettingQuery,
    GrantAccessRequest, GrantProjectMembershipRequest, InviteUserRequest, IsProjectMemberResponse,
    ListPermissionOverridesQuery, ProvisionTenantRequest, ProvisionTenantResponse,
    RegisterCompanyRequest, RequestAuditExportRequest, UpdateFileMetadataRequest,
    UpsertAuditRetentionPolicyRequest, UpsertPermissionOverrideRequest, UpsertSettingRequest,
};
use crate::api::extractors::CorePrincipal;
use crate::application::services::{
    AppendAuditEntryCommand, ApplyScanResultCommand, AuthorizeRequest,
    CreateFileUploadIntentCommand, CreatePublicShareLinkCommand, CreateTeamCommand,
    GrantAccessCommand, GrantProjectMembershipCommand, InviteUserCommand, ProvisionTenantCommand,
    RegisterCompanyCommand, UpsertPermissionOverrideCommand, UpsertSettingCommand,
};
use crate::application::{
    AuditApi, AuthzApi, FileApi, FlagsApi, IdentityApi, LicenseApi, MembershipApi, SettingsApi,
    TenancyApi,
};
use crate::domain::{
    permissions, AbacContext, AccessScope, AuditSearchQuery, CompanyType, CoreError, GrantKind,
    RoleEngine,
};
use crate::CoreModule;

/// Adapts [`CoreError`] to the platform's RFC-7807-ish problem body.
pub struct ApiError(CoreError);

impl From<CoreError> for ApiError {
    fn from(value: CoreError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let app_error: AppError = self.0.into();
        let status = StatusCode::from_u16(app_error.status_code())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        if matches!(app_error, AppError::Internal(_)) {
            tracing::error!(error = %app_error, "core internal API error");
        }
        (status, Json(ProblemDetails::from(&app_error))).into_response()
    }
}

type ApiResult<T> = Result<Json<T>, ApiError>;

pub async fn provision_tenant(
    State(module): State<CoreModule>,
    Json(body): Json<ProvisionTenantRequest>,
) -> ApiResult<ProvisionTenantResponse> {
    let result = module
        .services
        .provision_tenant(ProvisionTenantCommand {
            slug: body.slug,
            display_name: body.display_name,
            region_code: proven_shared::RegionCode::new(body.region_code),
            owner_company_name: body.owner_company_name,
            owner_company_type: body.owner_company_type.unwrap_or(CompanyType::Prime),
            admin_email: body.admin_email,
            admin_display_name: body.admin_display_name,
            seats_limit: body.seats_limit.unwrap_or(5),
        })
        .await?;

    Ok(Json(ProvisionTenantResponse {
        tenant: result.tenant,
        owner_company: result.owner_company,
        admin_user: result.admin_user,
        license: result.license,
    }))
}

pub async fn get_tenant(
    State(module): State<CoreModule>,
    Path(id): Path<Uuid>,
) -> ApiResult<crate::domain::Tenant> {
    let tenant = module.services.get_tenant(TenantId::from_uuid(id)).await?;
    Ok(Json(tenant))
}

pub async fn register_company(
    State(module): State<CoreModule>,
    Json(body): Json<RegisterCompanyRequest>,
) -> ApiResult<crate::domain::Company> {
    let company = module
        .services
        .register_company(RegisterCompanyCommand {
            tenant_id: TenantId::from_uuid(body.tenant_id),
            legal_name: body.legal_name,
            display_name: body.display_name,
            company_type: body.company_type,
        })
        .await?;
    Ok(Json(company))
}

pub async fn get_company(
    State(module): State<CoreModule>,
    Path(id): Path<Uuid>,
) -> ApiResult<crate::domain::Company> {
    let company = module
        .services
        .get_company(CompanyId::from_uuid(id))
        .await?;
    Ok(Json(company))
}

pub async fn invite_user(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Json(body): Json<InviteUserRequest>,
) -> ApiResult<crate::domain::User> {
    let user = module
        .services
        .invite_user(InviteUserCommand {
            tenant_id: principal.tenant_id,
            email: body.email,
            display_name: body.display_name,
            invited_by: Some(principal.user_id),
        })
        .await?;
    Ok(Json(user))
}

pub async fn get_user(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Path(id): Path<Uuid>,
) -> ApiResult<crate::domain::User> {
    let user = module
        .services
        .get_user(principal.tenant_id, UserId::from_uuid(id))
        .await?;
    Ok(Json(user))
}

pub async fn grant_access(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Json(body): Json<GrantAccessRequest>,
) -> ApiResult<crate::domain::AccessGrant> {
    let grant = module
        .services
        .grant_access(GrantAccessCommand {
            tenant_id: principal.tenant_id,
            user_id: UserId::from_uuid(body.user_id),
            role_id: RoleId::from_uuid(body.role_id),
            scope: AccessScope {
                scope_type: body.scope_type,
                scope_id: body.scope_id,
            },
            grant_kind: body.grant_kind.unwrap_or(GrantKind::Standard),
            expires_at: body.expires_at,
            created_by: Some(principal.user_id),
        })
        .await?;
    Ok(Json(grant))
}

pub async fn revoke_access(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    module
        .services
        .revoke_access(
            principal.tenant_id,
            GrantId::from_uuid(id),
            Some(principal.user_id),
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn authorize(
    State(module): State<CoreModule>,
    Json(body): Json<AuthorizeHttpRequest>,
) -> ApiResult<crate::domain::AuthzDecision> {
    let decision = module
        .services
        .authorize(AuthorizeRequest {
            tenant_id: TenantId::from_uuid(body.tenant_id),
            principal: proven_shared::PrincipalId::from_uuid(body.principal_id),
            permission: body.permission.as_str().into(),
            resource: AccessScope {
                scope_type: body.scope_type,
                scope_id: body.scope_id,
            },
            abac: AbacContext {
                resource_attributes: body.abac.resource_attributes,
                assurance_level: body.abac.assurance_level,
                resource_state: body.abac.resource_state,
            },
        })
        .await?;
    Ok(Json(decision))
}

pub async fn upsert_permission_override(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Json(body): Json<UpsertPermissionOverrideRequest>,
) -> ApiResult<crate::domain::PermissionOverride> {
    let override_ = module
        .services
        .upsert_permission_override(UpsertPermissionOverrideCommand {
            tenant_id: principal.tenant_id,
            user_id: UserId::from_uuid(body.user_id),
            permission: body.permission.as_str().into(),
            effect: body.effect,
            scope: AccessScope {
                scope_type: body.scope_type,
                scope_id: body.scope_id,
            },
            reason: body.reason,
            expires_at: body.expires_at,
            created_by: Some(principal.user_id),
        })
        .await?;
    Ok(Json(override_))
}

pub async fn revoke_permission_override(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    module
        .services
        .revoke_permission_override(
            principal.tenant_id,
            proven_shared::PermissionOverrideId::from_uuid(id),
            Some(principal.user_id),
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_permission_overrides(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Query(query): Query<ListPermissionOverridesQuery>,
) -> ApiResult<Vec<crate::domain::PermissionOverride>> {
    let overrides = module
        .services
        .list_permission_overrides(principal.tenant_id, UserId::from_uuid(query.user_id))
        .await?;
    Ok(Json(overrides))
}

/// Thin catalog browse endpoint — lists the platform-shipped system roles (ADR-0007 §5). Tenant
/// custom roles are not listed here since `RoleRepository` only supports point lookups today;
/// see `docs/development/ENTERPRISE_RBAC.md`.
pub async fn list_system_roles(
    State(module): State<CoreModule>,
    _principal: CorePrincipal,
) -> ApiResult<Vec<crate::domain::RoleDefinition>> {
    let mut roles = Vec::new();
    for role_id in RoleEngine::system_role_ids() {
        if let Some(role) = module.services.get_role(role_id).await? {
            roles.push(role);
        }
    }
    Ok(Json(roles))
}

pub async fn grant_project_membership(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Json(body): Json<GrantProjectMembershipRequest>,
) -> ApiResult<crate::domain::ProjectMembership> {
    let membership = module
        .services
        .grant_project_membership(GrantProjectMembershipCommand {
            tenant_id: principal.tenant_id,
            project_id: ProjectId::from_uuid(body.project_id),
            user_id: Some(UserId::from_uuid(body.user_id)),
            person_id: None::<PersonId>,
            membership_role: body.membership_role,
            granted_by: Some(principal.user_id),
        })
        .await?;
    Ok(Json(membership))
}

pub async fn is_project_member(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Path(project_id): Path<Uuid>,
) -> ApiResult<IsProjectMemberResponse> {
    let is_member = module
        .services
        .is_project_member(
            principal.tenant_id,
            ProjectId::from_uuid(project_id),
            principal.principal_id(),
        )
        .await?;
    Ok(Json(IsProjectMemberResponse {
        project_id,
        is_member,
    }))
}

pub async fn create_team(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Json(body): Json<CreateTeamRequest>,
) -> ApiResult<crate::domain::Team> {
    let team = module
        .services
        .create_team(CreateTeamCommand {
            tenant_id: principal.tenant_id,
            name: body.name,
            project_id: body.project_id.map(ProjectId::from_uuid),
        })
        .await?;
    Ok(Json(team))
}

pub async fn append_audit(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Json(body): Json<AppendAuditRequest>,
) -> ApiResult<crate::domain::AuditEntry> {
    let entry = module
        .services
        .append(AppendAuditEntryCommand {
            tenant_id: principal.tenant_id,
            actor_user_id: Some(principal.user_id),
            actor_type: "user".to_string(),
            action: body.action,
            resource_type: body.resource_type,
            resource_id: body.resource_id,
            correlation_id: body.correlation_id.map(CorrelationId::from_uuid),
            causation_id: body.causation_id.map(CausationId::from_uuid),
            payload: body.payload,
            module_key: body.module_key,
            category: body.category,
            outcome: body.outcome,
            project_id: body.project_id.map(ProjectId::from_uuid),
            company_id: body.company_id.map(CompanyId::from_uuid),
            old_value: body.old_value,
            new_value: body.new_value,
            ..Default::default()
        })
        .await?;
    Ok(Json(entry))
}

/// `core.audit.read` — filtered audit search (AUDIT_LOGGING_ARCHITECTURE.md §11). Superseded the
/// old unfiltered `query_audit` handler — `AuditSearchQuery::default()` (no query params) is
/// behaviorally identical to the old endpoint, so no route was lost.
pub async fn search_audit(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Query(query): Query<AuditSearchHttpQuery>,
) -> ApiResult<proven_shared::Page<crate::domain::AuditEntry>> {
    require_permission(&module, &principal, permissions::AUDIT_READ).await?;

    let page = module
        .services
        .search(
            principal.tenant_id,
            AuditSearchQuery {
                actor_user_id: query.actor_user_id.map(UserId::from_uuid),
                action: query.action,
                module_key: query.module_key,
                category: query.category,
                project_id: query.project_id.map(ProjectId::from_uuid),
                company_id: query.company_id.map(CompanyId::from_uuid),
                resource_type: query.resource_type,
                resource_id: query.resource_id,
                workflow_instance_id: query.workflow_instance_id,
                signature_package_id: query.signature_package_id,
                outcome: query.outcome,
                from: query.from,
                to: query.to,
                q: query.q,
            },
            PageRequest {
                limit: query.limit.unwrap_or(50),
                offset: query.offset.unwrap_or(0),
            },
        )
        .await?;
    Ok(Json(page))
}

/// `core.audit.export` — request an audit export job (AUDIT_LOGGING_ARCHITECTURE.md §10).
pub async fn request_audit_export(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Json(body): Json<RequestAuditExportRequest>,
) -> ApiResult<crate::domain::AuditExportJob> {
    require_permission(&module, &principal, permissions::AUDIT_EXPORT).await?;

    let job = module
        .services
        .request_export(
            principal.tenant_id,
            Some(principal.user_id),
            AuditSearchQuery {
                actor_user_id: body.actor_user_id.map(UserId::from_uuid),
                action: body.action,
                module_key: body.module_key,
                category: body.category,
                project_id: body.project_id.map(ProjectId::from_uuid),
                company_id: body.company_id.map(CompanyId::from_uuid),
                resource_type: body.resource_type,
                resource_id: body.resource_id,
                workflow_instance_id: body.workflow_instance_id,
                signature_package_id: body.signature_package_id,
                outcome: body.outcome,
                from: body.from,
                to: body.to,
                q: body.q,
            },
        )
        .await?;
    Ok(Json(job))
}

/// `core.audit.export` — poll an export job's status/result.
pub async fn get_audit_export(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Path(id): Path<Uuid>,
) -> ApiResult<crate::domain::AuditExportJob> {
    require_permission(&module, &principal, permissions::AUDIT_EXPORT).await?;

    let job = module.services.get_export(principal.tenant_id, id).await?;
    Ok(Json(job))
}

/// `core.audit.read` — current retention policy (defaults are returned when unset).
pub async fn get_audit_retention_policy(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
) -> ApiResult<crate::domain::AuditRetentionPolicy> {
    require_permission(&module, &principal, permissions::AUDIT_READ).await?;

    let policy = module
        .services
        .get_retention_policy(principal.tenant_id)
        .await?;
    Ok(Json(policy))
}

/// `core.audit.export` — upsert the tenant's retention policy (ops/compliance concern).
pub async fn put_audit_retention_policy(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Json(body): Json<UpsertAuditRetentionPolicyRequest>,
) -> ApiResult<crate::domain::AuditRetentionPolicy> {
    require_permission(&module, &principal, permissions::AUDIT_EXPORT).await?;

    let policy = module
        .services
        .upsert_retention_policy(crate::domain::AuditRetentionPolicy {
            tenant_id: principal.tenant_id,
            standard_days: body.standard_days,
            security_days: body.security_days,
            compliance_days: body.compliance_days,
            restricted_days: body.restricted_days,
            export_before_purge: body.export_before_purge,
            updated_at: chrono::Utc::now(),
        })
        .await?;
    Ok(Json(policy))
}

/// Fail-closed permission check for the Audit Engine's read/export HTTP surface — Core's own
/// handlers must obey the same `AuthzApi` decision path modules are required to use (ADR-0003).
async fn require_permission(
    module: &CoreModule,
    principal: &CorePrincipal,
    permission: &str,
) -> Result<(), ApiError> {
    let decision = module
        .services
        .authorize(AuthorizeRequest::new(
            principal.tenant_id,
            principal.principal_id(),
            permission.into(),
            AccessScope::tenant(),
        ))
        .await?;
    if decision.is_allowed() {
        Ok(())
    } else {
        Err(ApiError(CoreError::Forbidden(format!(
            "missing permission: {permission}"
        ))))
    }
}

pub async fn upsert_setting(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Json(body): Json<UpsertSettingRequest>,
) -> ApiResult<crate::domain::SettingEntry> {
    let entry = module
        .services
        .upsert(UpsertSettingCommand {
            tenant_id: principal.tenant_id,
            scope_type: body.scope_type,
            scope_id: body.scope_id,
            key: SettingKey(body.key),
            value: body.value,
        })
        .await?;
    Ok(Json(entry))
}

pub async fn get_setting(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Query(query): Query<GetSettingQuery>,
) -> ApiResult<crate::domain::SettingEntry> {
    let entry = module
        .services
        .get(
            principal.tenant_id,
            query.scope_type,
            query.scope_id,
            &SettingKey(query.key),
        )
        .await?
        .ok_or(CoreError::NotFound("setting"))?;
    Ok(Json(entry))
}

pub async fn evaluate_flag(
    State(module): State<CoreModule>,
    Json(body): Json<EvaluateFlagRequest>,
) -> ApiResult<EvaluateFlagResponse> {
    let key = FeatureFlagKey(body.key.clone());
    let enabled = module
        .services
        .evaluate(
            &key,
            body.tenant_id.map(TenantId::from_uuid),
            body.user_id.map(UserId::from_uuid),
        )
        .await?;
    Ok(Json(EvaluateFlagResponse {
        key: body.key,
        enabled,
    }))
}

pub async fn get_current_license(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
) -> ApiResult<crate::domain::License> {
    let license = module.services.get_current(principal.tenant_id).await?;
    Ok(Json(license))
}

pub async fn create_upload_intent(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Json(body): Json<CreateFileUploadIntentRequest>,
) -> ApiResult<crate::domain::UploadIntent> {
    let object_class = body
        .object_class
        .as_deref()
        .map(|raw| {
            crate::domain::FileObjectClass::parse(raw).ok_or_else(|| {
                CoreError::validation(format!("unknown object_class '{raw}'"))
            })
        })
        .transpose()
        .map_err(ApiError::from)?;

    let intent = module
        .services
        .create_upload_intent(CreateFileUploadIntentCommand {
            tenant_id: principal.tenant_id,
            content_type: body.content_type,
            retention_class: body.retention_class,
            access_class: body.access_class,
            created_by: Some(principal.user_id),
            object_class,
            original_filename: body.original_filename,
            metadata: body.metadata,
            parent_file_id: body.parent_file_id.map(FileObjectId::from_uuid),
            is_temporary: body.is_temporary,
            expires_at: None,
        })
        .await?;
    Ok(Json(intent))
}

pub async fn get_file(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Path(id): Path<Uuid>,
) -> ApiResult<crate::domain::FileObject> {
    let file = module
        .services
        .get_file(principal.tenant_id, FileObjectId::from_uuid(id))
        .await?;
    Ok(Json(file))
}

pub async fn list_file_versions(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Path(id): Path<Uuid>,
) -> ApiResult<Vec<crate::domain::FileObject>> {
    let versions = module
        .services
        .list_file_versions(principal.tenant_id, FileObjectId::from_uuid(id))
        .await?;
    Ok(Json(versions))
}

pub async fn complete_upload(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Path(id): Path<Uuid>,
    Json(body): Json<CompleteFileUploadRequest>,
) -> ApiResult<crate::domain::FileObject> {
    let file = module
        .services
        .complete_upload(
            principal.tenant_id,
            FileObjectId::from_uuid(id),
            body.checksum_sha256,
            body.byte_size,
        )
        .await?;
    Ok(Json(file))
}

pub async fn soft_delete_file(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Path(id): Path<Uuid>,
) -> ApiResult<crate::domain::FileObject> {
    let file = module
        .services
        .soft_delete_file(
            principal.tenant_id,
            FileObjectId::from_uuid(id),
            Some(principal.user_id),
        )
        .await?;
    Ok(Json(file))
}

pub async fn update_file_metadata(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateFileMetadataRequest>,
) -> ApiResult<crate::domain::FileObject> {
    let file = module
        .services
        .update_file_metadata(
            principal.tenant_id,
            FileObjectId::from_uuid(id),
            body.metadata,
            Some(principal.user_id),
        )
        .await?;
    Ok(Json(file))
}

pub async fn create_private_download_link(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Path(id): Path<Uuid>,
) -> ApiResult<crate::domain::DownloadLink> {
    let link = module
        .services
        .create_private_download_link(
            principal.tenant_id,
            FileObjectId::from_uuid(id),
            Some(principal.user_id),
        )
        .await?;
    Ok(Json(link))
}

pub async fn create_public_share_link(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Path(id): Path<Uuid>,
    Json(body): Json<CreatePublicShareLinkRequest>,
) -> ApiResult<crate::domain::FileShareLink> {
    let link = module
        .services
        .create_public_share_link(CreatePublicShareLinkCommand {
            tenant_id: principal.tenant_id,
            file_id: FileObjectId::from_uuid(id),
            created_by: Some(principal.user_id),
            ttl_hours: body.ttl_hours,
            max_downloads: body.max_downloads,
        })
        .await?;
    Ok(Json(link))
}

pub async fn resolve_public_share_link(
    State(module): State<CoreModule>,
    Path(token): Path<String>,
) -> ApiResult<crate::domain::DownloadLink> {
    let link = module.services.resolve_public_share_link(&token).await?;
    Ok(Json(link))
}

pub async fn apply_scan_result(
    State(module): State<CoreModule>,
    principal: CorePrincipal,
    Path(id): Path<Uuid>,
    Json(body): Json<ApplyScanResultRequest>,
) -> ApiResult<crate::domain::FileObject> {
    let outcome = match body.outcome.as_str() {
        "clean" => crate::domain::VirusScanOutcome::Clean {
            detail: body.detail,
        },
        "infected" => crate::domain::VirusScanOutcome::Infected {
            detail: body.detail,
        },
        "pending" => crate::domain::VirusScanOutcome::Pending {
            detail: body.detail,
        },
        "error" => crate::domain::VirusScanOutcome::Error {
            detail: body.detail.unwrap_or_else(|| "scan_error".into()),
        },
        other => {
            return Err(ApiError::from(CoreError::validation(format!(
                "unknown scan outcome '{other}'"
            ))));
        }
    };

    let file = module
        .services
        .apply_scan_result(ApplyScanResultCommand {
            tenant_id: principal.tenant_id,
            file_id: FileObjectId::from_uuid(id),
            outcome,
            actor_user_id: Some(principal.user_id),
        })
        .await?;
    Ok(Json(file))
}
