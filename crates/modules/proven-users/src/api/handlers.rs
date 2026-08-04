//! Axum HTTP handlers — thin adapters over `UsersServices`. All business rules live in
//! `application::services`; handlers only parse/validate transport concerns and map errors.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use proven_shared::{AppError, FileObjectId, ProblemDetails, UserId};
use uuid::Uuid;

use crate::api::dto::{
    AddEmergencyContactRequest, AssignKindRequest, EnsureProfileRequest,
    UpdateEmergencyContactRequest, UpdateProfileRequest, UpsertAccessibilityRequest,
    UpsertAuthenticationProfileRequest, UpsertAvatarRequest, UpsertLocaleRequest,
    UpsertNotificationPreferencesRequest, UpsertSettingRequest, UpsertSignatureProfileRequest,
};
use crate::api::extractors::UsersPrincipal;
use crate::application::services::{
    AddEmergencyContactCommand, AssignUserKindCommand, UpdateEmergencyContactCommand,
    UpdateProfileCommand, UpsertAccessibilityCommand, UpsertAuthenticationProfileCommand,
    UpsertAvatarCommand, UpsertLocaleCommand, UpsertNotificationPreferencesCommand,
    UpsertSignatureProfileCommand, UpsertUserSettingCommand,
};
use crate::application::UsersApi;
use crate::domain::{
    AccessibilityPreferences, AuthenticationProfile, Avatar, DigitalSignatureProfile,
    EmergencyContact, EmergencyContactId, LocalePreferences, NotificationPreferences,
    ProfileAuditEntry, UserKind, UserKindAssignment, UserProfile, UserSetting, UsersError,
};
use crate::UsersModule;

/// Adapts [`UsersError`] to the platform's RFC-7807-ish problem body.
pub struct ApiError(UsersError);

impl From<UsersError> for ApiError {
    fn from(value: UsersError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let app_error: AppError = self.0.into();
        let status = StatusCode::from_u16(app_error.status_code())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        if matches!(app_error, AppError::Internal(_)) {
            tracing::error!(error = %app_error, "users internal API error");
        }
        (status, Json(ProblemDetails::from(&app_error))).into_response()
    }
}

type ApiResult<T> = Result<Json<T>, ApiError>;

fn parse_user_kind(raw: &str) -> Result<UserKind, ApiError> {
    match raw {
        "worker" => Ok(UserKind::Worker),
        "supervisor" => Ok(UserKind::Supervisor),
        "manager" => Ok(UserKind::Manager),
        "safety_coordinator" => Ok(UserKind::SafetyCoordinator),
        "administrator" => Ok(UserKind::Administrator),
        "external" => Ok(UserKind::External),
        "guest" => Ok(UserKind::Guest),
        other => Err(UsersError::validation(format!("unknown user kind: {other}")).into()),
    }
}

pub async fn ensure_profile(
    State(module): State<UsersModule>,
    principal: UsersPrincipal,
    Path(user_id): Path<Uuid>,
    Json(body): Json<EnsureProfileRequest>,
) -> ApiResult<UserProfile> {
    let profile = module
        .services
        .ensure_profile(
            principal.acting_context(),
            UserId::from_uuid(user_id),
            body.display_name,
        )
        .await?;
    Ok(Json(profile))
}

pub async fn get_profile(
    State(module): State<UsersModule>,
    Path(user_id): Path<Uuid>,
) -> ApiResult<UserProfile> {
    let profile = module
        .services
        .get_profile(UserId::from_uuid(user_id))
        .await?;
    Ok(Json(profile))
}

pub async fn update_profile(
    State(module): State<UsersModule>,
    principal: UsersPrincipal,
    Path(user_id): Path<Uuid>,
    Json(body): Json<UpdateProfileRequest>,
) -> ApiResult<UserProfile> {
    let profile = module
        .services
        .update_profile(
            principal.acting_context(),
            UpdateProfileCommand {
                user_id: UserId::from_uuid(user_id),
                display_name: body.display_name,
                preferred_name: body.preferred_name,
                job_title: body.job_title,
                phone: body.phone,
                bio: body.bio,
            },
        )
        .await?;
    Ok(Json(profile))
}

pub async fn archive_profile(
    State(module): State<UsersModule>,
    principal: UsersPrincipal,
    Path(user_id): Path<Uuid>,
) -> ApiResult<UserProfile> {
    let profile = module
        .services
        .archive_profile(principal.acting_context(), UserId::from_uuid(user_id))
        .await?;
    Ok(Json(profile))
}

pub async fn assign_kind(
    State(module): State<UsersModule>,
    principal: UsersPrincipal,
    Path(user_id): Path<Uuid>,
    Json(body): Json<AssignKindRequest>,
) -> ApiResult<UserKindAssignment> {
    let assignment = module
        .services
        .assign_kind(
            principal.acting_context(),
            AssignUserKindCommand {
                user_id: UserId::from_uuid(user_id),
                kind: body.kind,
                is_primary: body.is_primary,
            },
        )
        .await?;
    Ok(Json(assignment))
}

pub async fn list_kinds(
    State(module): State<UsersModule>,
    Path(user_id): Path<Uuid>,
) -> ApiResult<Vec<UserKindAssignment>> {
    let kinds = module
        .services
        .list_kinds(UserId::from_uuid(user_id))
        .await?;
    Ok(Json(kinds))
}

pub async fn remove_kind(
    State(module): State<UsersModule>,
    principal: UsersPrincipal,
    Path((user_id, kind)): Path<(Uuid, String)>,
) -> Result<StatusCode, ApiError> {
    let kind = parse_user_kind(&kind)?;
    module
        .services
        .remove_kind(principal.acting_context(), UserId::from_uuid(user_id), kind)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_avatar(
    State(module): State<UsersModule>,
    Path(user_id): Path<Uuid>,
) -> ApiResult<Avatar> {
    let avatar = module
        .services
        .get_avatar(UserId::from_uuid(user_id))
        .await?;
    Ok(Json(avatar))
}

pub async fn upsert_avatar(
    State(module): State<UsersModule>,
    principal: UsersPrincipal,
    Path(user_id): Path<Uuid>,
    Json(body): Json<UpsertAvatarRequest>,
) -> ApiResult<Avatar> {
    let avatar = module
        .services
        .upsert_avatar(
            principal.acting_context(),
            UpsertAvatarCommand {
                user_id: UserId::from_uuid(user_id),
                file_object_id: body.file_object_id.map(FileObjectId::from_uuid),
                avatar_url: body.avatar_url,
            },
        )
        .await?;
    Ok(Json(avatar))
}

pub async fn get_locale(
    State(module): State<UsersModule>,
    Path(user_id): Path<Uuid>,
) -> ApiResult<LocalePreferences> {
    let prefs = module
        .services
        .get_locale(UserId::from_uuid(user_id))
        .await?;
    Ok(Json(prefs))
}

pub async fn upsert_locale(
    State(module): State<UsersModule>,
    principal: UsersPrincipal,
    Path(user_id): Path<Uuid>,
    Json(body): Json<UpsertLocaleRequest>,
) -> ApiResult<LocalePreferences> {
    let prefs = module
        .services
        .upsert_locale(
            principal.acting_context(),
            UpsertLocaleCommand {
                user_id: UserId::from_uuid(user_id),
                language_code: body.language_code,
                time_zone: body.time_zone,
            },
        )
        .await?;
    Ok(Json(prefs))
}

pub async fn get_accessibility(
    State(module): State<UsersModule>,
    Path(user_id): Path<Uuid>,
) -> ApiResult<AccessibilityPreferences> {
    let prefs = module
        .services
        .get_accessibility(UserId::from_uuid(user_id))
        .await?;
    Ok(Json(prefs))
}

pub async fn upsert_accessibility(
    State(module): State<UsersModule>,
    principal: UsersPrincipal,
    Path(user_id): Path<Uuid>,
    Json(body): Json<UpsertAccessibilityRequest>,
) -> ApiResult<AccessibilityPreferences> {
    let prefs = module
        .services
        .upsert_accessibility(
            principal.acting_context(),
            UpsertAccessibilityCommand {
                user_id: UserId::from_uuid(user_id),
                reduce_motion: body.reduce_motion,
                high_contrast: body.high_contrast,
                large_text: body.large_text,
                screen_reader_hints: body.screen_reader_hints,
            },
        )
        .await?;
    Ok(Json(prefs))
}

pub async fn get_notification_preferences(
    State(module): State<UsersModule>,
    Path(user_id): Path<Uuid>,
) -> ApiResult<NotificationPreferences> {
    let prefs = module
        .services
        .get_notification_preferences(UserId::from_uuid(user_id))
        .await?;
    Ok(Json(prefs))
}

pub async fn upsert_notification_preferences(
    State(module): State<UsersModule>,
    principal: UsersPrincipal,
    Path(user_id): Path<Uuid>,
    Json(body): Json<UpsertNotificationPreferencesRequest>,
) -> ApiResult<NotificationPreferences> {
    let prefs = module
        .services
        .upsert_notification_preferences(
            principal.acting_context(),
            UpsertNotificationPreferencesCommand {
                user_id: UserId::from_uuid(user_id),
                email_enabled: body.email_enabled,
                push_enabled: body.push_enabled,
                sms_enabled: body.sms_enabled,
                in_app_enabled: body.in_app_enabled,
                digest_cadence: body.digest_cadence,
                quiet_hours_start: body.quiet_hours_start,
                quiet_hours_end: body.quiet_hours_end,
            },
        )
        .await?;
    Ok(Json(prefs))
}

pub async fn get_authentication_profile(
    State(module): State<UsersModule>,
    Path(user_id): Path<Uuid>,
) -> ApiResult<AuthenticationProfile> {
    let profile = module
        .services
        .get_authentication_profile(UserId::from_uuid(user_id))
        .await?;
    Ok(Json(profile))
}

pub async fn upsert_authentication_profile(
    State(module): State<UsersModule>,
    principal: UsersPrincipal,
    Path(user_id): Path<Uuid>,
    Json(body): Json<UpsertAuthenticationProfileRequest>,
) -> ApiResult<AuthenticationProfile> {
    let profile = module
        .services
        .upsert_authentication_profile(
            principal.acting_context(),
            UpsertAuthenticationProfileCommand {
                user_id: UserId::from_uuid(user_id),
                mfa_preferred: body.mfa_preferred,
                password_login_enabled: body.password_login_enabled,
                oauth_google_linked: body.oauth_google_linked,
                oauth_microsoft_linked: body.oauth_microsoft_linked,
                magic_link_preferred: body.magic_link_preferred,
                last_auth_method: body.last_auth_method,
            },
        )
        .await?;
    Ok(Json(profile))
}

pub async fn get_signature_profile(
    State(module): State<UsersModule>,
    Path(user_id): Path<Uuid>,
) -> ApiResult<DigitalSignatureProfile> {
    let profile = module
        .services
        .get_signature_profile(UserId::from_uuid(user_id))
        .await?;
    Ok(Json(profile))
}

pub async fn upsert_signature_profile(
    State(module): State<UsersModule>,
    principal: UsersPrincipal,
    Path(user_id): Path<Uuid>,
    Json(body): Json<UpsertSignatureProfileRequest>,
) -> ApiResult<DigitalSignatureProfile> {
    let profile = module
        .services
        .upsert_signature_profile(
            principal.acting_context(),
            UpsertSignatureProfileCommand {
                user_id: UserId::from_uuid(user_id),
                default_signature_type: body.default_signature_type,
                typed_name_default: body.typed_name_default,
                signature_image_file_id: body.signature_image_file_id.map(FileObjectId::from_uuid),
                require_reauth_to_sign: body.require_reauth_to_sign,
            },
        )
        .await?;
    Ok(Json(profile))
}

pub async fn add_emergency_contact(
    State(module): State<UsersModule>,
    principal: UsersPrincipal,
    Path(user_id): Path<Uuid>,
    Json(body): Json<AddEmergencyContactRequest>,
) -> ApiResult<EmergencyContact> {
    let contact = module
        .services
        .add_emergency_contact(
            principal.acting_context(),
            AddEmergencyContactCommand {
                user_id: UserId::from_uuid(user_id),
                full_name: body.full_name,
                relationship: body.relationship,
                phone: body.phone,
                email: body.email,
                is_primary: body.is_primary,
            },
        )
        .await?;
    Ok(Json(contact))
}

pub async fn list_emergency_contacts(
    State(module): State<UsersModule>,
    Path(user_id): Path<Uuid>,
) -> ApiResult<Vec<EmergencyContact>> {
    let contacts = module
        .services
        .list_emergency_contacts(UserId::from_uuid(user_id))
        .await?;
    Ok(Json(contacts))
}

pub async fn update_emergency_contact(
    State(module): State<UsersModule>,
    principal: UsersPrincipal,
    Path((user_id, contact_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateEmergencyContactRequest>,
) -> ApiResult<EmergencyContact> {
    let contact = module
        .services
        .update_emergency_contact(
            principal.acting_context(),
            UpdateEmergencyContactCommand {
                user_id: UserId::from_uuid(user_id),
                contact_id: EmergencyContactId::from_uuid(contact_id),
                full_name: body.full_name,
                relationship: body.relationship,
                phone: body.phone,
                email: body.email,
                is_primary: body.is_primary,
            },
        )
        .await?;
    Ok(Json(contact))
}

pub async fn remove_emergency_contact(
    State(module): State<UsersModule>,
    principal: UsersPrincipal,
    Path((user_id, contact_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    module
        .services
        .remove_emergency_contact(
            principal.acting_context(),
            UserId::from_uuid(user_id),
            EmergencyContactId::from_uuid(contact_id),
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_setting(
    State(module): State<UsersModule>,
    Path((user_id, key)): Path<(Uuid, String)>,
) -> ApiResult<UserSetting> {
    let setting = module
        .services
        .get_setting(UserId::from_uuid(user_id), key)
        .await?;
    Ok(Json(setting))
}

pub async fn upsert_setting(
    State(module): State<UsersModule>,
    principal: UsersPrincipal,
    Path((user_id, key)): Path<(Uuid, String)>,
    Json(body): Json<UpsertSettingRequest>,
) -> ApiResult<UserSetting> {
    let setting = module
        .services
        .upsert_setting(
            principal.acting_context(),
            UpsertUserSettingCommand {
                user_id: UserId::from_uuid(user_id),
                key,
                value: body.value,
            },
        )
        .await?;
    Ok(Json(setting))
}

pub async fn list_audit_history(
    State(module): State<UsersModule>,
    principal: UsersPrincipal,
    Path(user_id): Path<Uuid>,
) -> ApiResult<Vec<ProfileAuditEntry>> {
    let history = module
        .services
        .list_audit_history(principal.acting_context(), UserId::from_uuid(user_id))
        .await?;
    Ok(Json(history))
}
