//! Axum HTTP handlers — thin adapters over `CompaniesServices`. All business rules live in
//! `application::services`; handlers only parse/validate transport concerns and map errors.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use proven_shared::{AppError, CompanyId, FileObjectId, ProblemDetails, UserId};
use uuid::Uuid;

use crate::api::dto::{
    AddAddressRequest, AddContactRequest, CreateBusinessUnitRequest, UpdateAddressRequest,
    UpdateBusinessUnitRequest, UpdateContactRequest, UpdateProfileRequest,
    UpsertBrandingRequest, UpsertDefaultTemplateRequest, UpsertNotificationDefaultsRequest,
    UpsertRegionalSettingsRequest, UpsertSafetySettingsRequest,
    UpsertStorageConfigurationRequest,
};
use crate::api::extractors::CompaniesPrincipal;
use crate::application::services::{
    AddAddressCommand, AddContactCommand, CreateBusinessUnitCommand, UpdateAddressCommand,
    UpdateBusinessUnitCommand, UpdateContactCommand, UpdateProfileCommand,
    UpsertBrandingCommand, UpsertDefaultTemplateCommand, UpsertNotificationDefaultsCommand,
    UpsertRegionalSettingsCommand, UpsertSafetySettingsCommand,
    UpsertStorageConfigurationCommand,
};
use crate::application::CompaniesApi;
use crate::domain::{
    Address, AddressId, BusinessUnit, BusinessUnitId, CompaniesError, CompanyBranding,
    CompanyProfile, Contact, ContactId, DefaultTemplate, NotificationDefaults, RegionalSettings,
    SafetySettings, StorageConfiguration,
};
use crate::CompaniesModule;

/// Adapts [`CompaniesError`] to the platform's RFC-7807-ish problem body.
pub struct ApiError(CompaniesError);

impl From<CompaniesError> for ApiError {
    fn from(value: CompaniesError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let app_error: AppError = self.0.into();
        let status = StatusCode::from_u16(app_error.status_code())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        if matches!(app_error, AppError::Internal(_)) {
            tracing::error!(error = %app_error, "companies internal API error");
        }
        (status, Json(ProblemDetails::from(&app_error))).into_response()
    }
}

type ApiResult<T> = Result<Json<T>, ApiError>;

pub async fn ensure_profile(
    State(module): State<CompaniesModule>,
    principal: CompaniesPrincipal,
    Path(company_id): Path<Uuid>,
) -> ApiResult<CompanyProfile> {
    let profile = module
        .services
        .ensure_profile(principal.acting_context(), CompanyId::from_uuid(company_id))
        .await?;
    Ok(Json(profile))
}

pub async fn get_profile(
    State(module): State<CompaniesModule>,
    Path(company_id): Path<Uuid>,
) -> ApiResult<CompanyProfile> {
    let profile = module
        .services
        .get_profile(CompanyId::from_uuid(company_id))
        .await?;
    Ok(Json(profile))
}

pub async fn update_profile(
    State(module): State<CompaniesModule>,
    principal: CompaniesPrincipal,
    Path(company_id): Path<Uuid>,
    Json(body): Json<UpdateProfileRequest>,
) -> ApiResult<CompanyProfile> {
    let profile = module
        .services
        .update_profile(
            principal.acting_context(),
            UpdateProfileCommand {
                company_id: CompanyId::from_uuid(company_id),
                trade_name: body.trade_name,
                website: body.website,
                notes: body.notes,
            },
        )
        .await?;
    Ok(Json(profile))
}

pub async fn archive_profile(
    State(module): State<CompaniesModule>,
    principal: CompaniesPrincipal,
    Path(company_id): Path<Uuid>,
) -> ApiResult<CompanyProfile> {
    let profile = module
        .services
        .archive_profile(principal.acting_context(), CompanyId::from_uuid(company_id))
        .await?;
    Ok(Json(profile))
}

pub async fn create_business_unit(
    State(module): State<CompaniesModule>,
    principal: CompaniesPrincipal,
    Path(company_id): Path<Uuid>,
    Json(body): Json<CreateBusinessUnitRequest>,
) -> ApiResult<BusinessUnit> {
    let unit = module
        .services
        .create_business_unit(
            principal.acting_context(),
            CreateBusinessUnitCommand {
                company_id: CompanyId::from_uuid(company_id),
                name: body.name,
                code: body.code,
                parent_id: body.parent_id.map(BusinessUnitId::from_uuid),
                org_unit_id: body.org_unit_id,
            },
        )
        .await?;
    Ok(Json(unit))
}

pub async fn list_business_units(
    State(module): State<CompaniesModule>,
    Path(company_id): Path<Uuid>,
) -> ApiResult<Vec<BusinessUnit>> {
    let units = module
        .services
        .list_business_units(CompanyId::from_uuid(company_id))
        .await?;
    Ok(Json(units))
}

pub async fn update_business_unit(
    State(module): State<CompaniesModule>,
    principal: CompaniesPrincipal,
    Path((company_id, unit_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateBusinessUnitRequest>,
) -> ApiResult<BusinessUnit> {
    let unit = module
        .services
        .update_business_unit(
            principal.acting_context(),
            UpdateBusinessUnitCommand {
                company_id: CompanyId::from_uuid(company_id),
                business_unit_id: BusinessUnitId::from_uuid(unit_id),
                name: body.name,
                code: body.code,
                parent_id: body.parent_id.map(BusinessUnitId::from_uuid),
                org_unit_id: body.org_unit_id,
            },
        )
        .await?;
    Ok(Json(unit))
}

pub async fn archive_business_unit(
    State(module): State<CompaniesModule>,
    principal: CompaniesPrincipal,
    Path((company_id, unit_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<BusinessUnit> {
    let unit = module
        .services
        .archive_business_unit(
            principal.acting_context(),
            CompanyId::from_uuid(company_id),
            BusinessUnitId::from_uuid(unit_id),
        )
        .await?;
    Ok(Json(unit))
}

pub async fn add_address(
    State(module): State<CompaniesModule>,
    principal: CompaniesPrincipal,
    Path(company_id): Path<Uuid>,
    Json(body): Json<AddAddressRequest>,
) -> ApiResult<Address> {
    let address = module
        .services
        .add_address(
            principal.acting_context(),
            AddAddressCommand {
                company_id: CompanyId::from_uuid(company_id),
                business_unit_id: body.business_unit_id.map(BusinessUnitId::from_uuid),
                kind: body.kind,
                line1: body.line1,
                line2: body.line2,
                city: body.city,
                region: body.region,
                postal_code: body.postal_code,
                country_code: body.country_code,
                is_primary: body.is_primary,
            },
        )
        .await?;
    Ok(Json(address))
}

pub async fn list_addresses(
    State(module): State<CompaniesModule>,
    Path(company_id): Path<Uuid>,
) -> ApiResult<Vec<Address>> {
    let addresses = module
        .services
        .list_addresses(CompanyId::from_uuid(company_id))
        .await?;
    Ok(Json(addresses))
}

pub async fn update_address(
    State(module): State<CompaniesModule>,
    principal: CompaniesPrincipal,
    Path((company_id, address_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateAddressRequest>,
) -> ApiResult<Address> {
    let address = module
        .services
        .update_address(
            principal.acting_context(),
            UpdateAddressCommand {
                company_id: CompanyId::from_uuid(company_id),
                address_id: AddressId::from_uuid(address_id),
                business_unit_id: body.business_unit_id.map(BusinessUnitId::from_uuid),
                kind: body.kind,
                line1: body.line1,
                line2: body.line2,
                city: body.city,
                region: body.region,
                postal_code: body.postal_code,
                country_code: body.country_code,
                is_primary: body.is_primary,
            },
        )
        .await?;
    Ok(Json(address))
}

pub async fn remove_address(
    State(module): State<CompaniesModule>,
    principal: CompaniesPrincipal,
    Path((company_id, address_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    module
        .services
        .remove_address(
            principal.acting_context(),
            CompanyId::from_uuid(company_id),
            AddressId::from_uuid(address_id),
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_contact(
    State(module): State<CompaniesModule>,
    principal: CompaniesPrincipal,
    Path(company_id): Path<Uuid>,
    Json(body): Json<AddContactRequest>,
) -> ApiResult<Contact> {
    let contact = module
        .services
        .add_contact(
            principal.acting_context(),
            AddContactCommand {
                company_id: CompanyId::from_uuid(company_id),
                business_unit_id: body.business_unit_id.map(BusinessUnitId::from_uuid),
                kind: body.kind,
                full_name: body.full_name,
                title: body.title,
                email: body.email,
                phone: body.phone,
                user_id: body.user_id.map(UserId::from_uuid),
                is_primary: body.is_primary,
            },
        )
        .await?;
    Ok(Json(contact))
}

pub async fn list_contacts(
    State(module): State<CompaniesModule>,
    Path(company_id): Path<Uuid>,
) -> ApiResult<Vec<Contact>> {
    let contacts = module
        .services
        .list_contacts(CompanyId::from_uuid(company_id))
        .await?;
    Ok(Json(contacts))
}

pub async fn update_contact(
    State(module): State<CompaniesModule>,
    principal: CompaniesPrincipal,
    Path((company_id, contact_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateContactRequest>,
) -> ApiResult<Contact> {
    let contact = module
        .services
        .update_contact(
            principal.acting_context(),
            UpdateContactCommand {
                company_id: CompanyId::from_uuid(company_id),
                contact_id: ContactId::from_uuid(contact_id),
                business_unit_id: body.business_unit_id.map(BusinessUnitId::from_uuid),
                kind: body.kind,
                full_name: body.full_name,
                title: body.title,
                email: body.email,
                phone: body.phone,
                user_id: body.user_id.map(UserId::from_uuid),
                is_primary: body.is_primary,
            },
        )
        .await?;
    Ok(Json(contact))
}

pub async fn remove_contact(
    State(module): State<CompaniesModule>,
    principal: CompaniesPrincipal,
    Path((company_id, contact_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    module
        .services
        .remove_contact(
            principal.acting_context(),
            CompanyId::from_uuid(company_id),
            ContactId::from_uuid(contact_id),
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_branding(
    State(module): State<CompaniesModule>,
    Path(company_id): Path<Uuid>,
) -> ApiResult<CompanyBranding> {
    let branding = module
        .services
        .get_branding(CompanyId::from_uuid(company_id))
        .await?;
    Ok(Json(branding))
}

pub async fn upsert_branding(
    State(module): State<CompaniesModule>,
    principal: CompaniesPrincipal,
    Path(company_id): Path<Uuid>,
    Json(body): Json<UpsertBrandingRequest>,
) -> ApiResult<CompanyBranding> {
    let branding = module
        .services
        .upsert_branding(
            principal.acting_context(),
            UpsertBrandingCommand {
                company_id: CompanyId::from_uuid(company_id),
                logo_file_id: body.logo_file_id.map(FileObjectId::from_uuid),
                wordmark_file_id: body.wordmark_file_id.map(FileObjectId::from_uuid),
                primary_color: body.primary_color,
                secondary_color: body.secondary_color,
                accent_color: body.accent_color,
                favicon_file_id: body.favicon_file_id.map(FileObjectId::from_uuid),
            },
        )
        .await?;
    Ok(Json(branding))
}

pub async fn get_safety_settings(
    State(module): State<CompaniesModule>,
    Path(company_id): Path<Uuid>,
) -> ApiResult<SafetySettings> {
    let settings = module
        .services
        .get_safety_settings(CompanyId::from_uuid(company_id))
        .await?;
    Ok(Json(settings))
}

pub async fn upsert_safety_settings(
    State(module): State<CompaniesModule>,
    principal: CompaniesPrincipal,
    Path(company_id): Path<Uuid>,
    Json(body): Json<UpsertSafetySettingsRequest>,
) -> ApiResult<SafetySettings> {
    let settings = module
        .services
        .upsert_safety_settings(
            principal.acting_context(),
            UpsertSafetySettingsCommand {
                company_id: CompanyId::from_uuid(company_id),
                require_flha_before_work: body.require_flha_before_work,
                require_toolbox_talk_weekly: body.require_toolbox_talk_weekly,
                incident_notify_emails: body.incident_notify_emails,
                default_risk_matrix: body.default_risk_matrix,
                allow_offline_safety_submit: body.allow_offline_safety_submit,
            },
        )
        .await?;
    Ok(Json(settings))
}

pub async fn get_regional_settings(
    State(module): State<CompaniesModule>,
    Path(company_id): Path<Uuid>,
) -> ApiResult<RegionalSettings> {
    let settings = module
        .services
        .get_regional_settings(CompanyId::from_uuid(company_id))
        .await?;
    Ok(Json(settings))
}

pub async fn upsert_regional_settings(
    State(module): State<CompaniesModule>,
    principal: CompaniesPrincipal,
    Path(company_id): Path<Uuid>,
    Json(body): Json<UpsertRegionalSettingsRequest>,
) -> ApiResult<RegionalSettings> {
    let settings = module
        .services
        .upsert_regional_settings(
            principal.acting_context(),
            UpsertRegionalSettingsCommand {
                company_id: CompanyId::from_uuid(company_id),
                primary_region: body.primary_region,
                locales: body.locales,
                timezone: body.timezone,
                measurement_system: body.measurement_system,
                currency_code: body.currency_code,
            },
        )
        .await?;
    Ok(Json(settings))
}

pub async fn list_default_templates(
    State(module): State<CompaniesModule>,
    Path(company_id): Path<Uuid>,
) -> ApiResult<Vec<DefaultTemplate>> {
    let templates = module
        .services
        .list_default_templates(CompanyId::from_uuid(company_id))
        .await?;
    Ok(Json(templates))
}

pub async fn upsert_default_template(
    State(module): State<CompaniesModule>,
    principal: CompaniesPrincipal,
    Path(company_id): Path<Uuid>,
    Json(body): Json<UpsertDefaultTemplateRequest>,
) -> ApiResult<DefaultTemplate> {
    let template = module
        .services
        .upsert_default_template(
            principal.acting_context(),
            UpsertDefaultTemplateCommand {
                company_id: CompanyId::from_uuid(company_id),
                kind: body.kind,
                template_ref: body.template_ref,
                label: body.label,
            },
        )
        .await?;
    Ok(Json(template))
}

pub async fn get_notification_defaults(
    State(module): State<CompaniesModule>,
    Path(company_id): Path<Uuid>,
) -> ApiResult<NotificationDefaults> {
    let defaults = module
        .services
        .get_notification_defaults(CompanyId::from_uuid(company_id))
        .await?;
    Ok(Json(defaults))
}

pub async fn upsert_notification_defaults(
    State(module): State<CompaniesModule>,
    principal: CompaniesPrincipal,
    Path(company_id): Path<Uuid>,
    Json(body): Json<UpsertNotificationDefaultsRequest>,
) -> ApiResult<NotificationDefaults> {
    let defaults = module
        .services
        .upsert_notification_defaults(
            principal.acting_context(),
            UpsertNotificationDefaultsCommand {
                company_id: CompanyId::from_uuid(company_id),
                email_enabled: body.email_enabled,
                push_enabled: body.push_enabled,
                sms_enabled: body.sms_enabled,
                digest_cadence: body.digest_cadence,
                quiet_hours_start: body.quiet_hours_start,
                quiet_hours_end: body.quiet_hours_end,
                default_locale: body.default_locale,
            },
        )
        .await?;
    Ok(Json(defaults))
}

pub async fn get_storage_configuration(
    State(module): State<CompaniesModule>,
    Path(company_id): Path<Uuid>,
) -> ApiResult<StorageConfiguration> {
    let config = module
        .services
        .get_storage_configuration(CompanyId::from_uuid(company_id))
        .await?;
    Ok(Json(config))
}

pub async fn upsert_storage_configuration(
    State(module): State<CompaniesModule>,
    principal: CompaniesPrincipal,
    Path(company_id): Path<Uuid>,
    Json(body): Json<UpsertStorageConfigurationRequest>,
) -> ApiResult<StorageConfiguration> {
    let config = module
        .services
        .upsert_storage_configuration(
            principal.acting_context(),
            UpsertStorageConfigurationCommand {
                company_id: CompanyId::from_uuid(company_id),
                object_prefix: body.object_prefix,
                max_upload_bytes: body.max_upload_bytes,
                allowed_content_types: body.allowed_content_types,
                retention_class_default: body.retention_class_default,
                quarantine_enabled: body.quarantine_enabled,
            },
        )
        .await?;
    Ok(Json(config))
}
