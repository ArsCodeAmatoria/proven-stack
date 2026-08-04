//! `ProfileService` — company profile lifecycle + first-run default provisioning (ADR-0005 §3).

use std::sync::Arc;

use chrono::Utc;
use proven_core::AuthzApi;
use proven_core::TenancyApi;
use proven_shared::CompanyId;

use crate::application::ports::{
    CompanyProfileRepository, EventPublisher, NotificationDefaultsRepository,
    RegionalSettingsRepository, SafetySettingsRepository, StorageConfigurationRepository,
};
use crate::application::services::authz::{authorize, ActingContext};
use crate::domain::permissions;
use crate::domain::{
    CompaniesError, CompanyProfile, NotificationDefaults, ProfileStatus, RegionalSettings,
    SafetySettings, StorageConfiguration,
};
use crate::events::{ActorRef, CompaniesEvent, EventEnvelope, ResourceRef};

pub struct UpdateProfileCommand {
    pub company_id: CompanyId,
    pub trade_name: Option<String>,
    pub website: Option<String>,
    pub notes: Option<String>,
}

pub struct ProfileService {
    profiles: Arc<dyn CompanyProfileRepository>,
    safety_settings: Arc<dyn SafetySettingsRepository>,
    regional_settings: Arc<dyn RegionalSettingsRepository>,
    notification_defaults: Arc<dyn NotificationDefaultsRepository>,
    storage_configuration: Arc<dyn StorageConfigurationRepository>,
    outbox: Arc<dyn EventPublisher>,
    authz: Arc<dyn AuthzApi>,
    tenancy: Option<Arc<dyn TenancyApi>>,
}

impl ProfileService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        profiles: Arc<dyn CompanyProfileRepository>,
        safety_settings: Arc<dyn SafetySettingsRepository>,
        regional_settings: Arc<dyn RegionalSettingsRepository>,
        notification_defaults: Arc<dyn NotificationDefaultsRepository>,
        storage_configuration: Arc<dyn StorageConfigurationRepository>,
        outbox: Arc<dyn EventPublisher>,
        authz: Arc<dyn AuthzApi>,
        tenancy: Option<Arc<dyn TenancyApi>>,
    ) -> Self {
        Self {
            profiles,
            safety_settings,
            regional_settings,
            notification_defaults,
            storage_configuration,
            outbox,
            authz,
            tenancy,
        }
    }

    /// Idempotently ensures a profile shell (+ default safety/regional/notification/storage
    /// rows) exists for `company_id`. Returns the existing profile if one is already present.
    pub async fn ensure_profile(
        &self,
        ctx: &ActingContext,
        company_id: CompanyId,
    ) -> Result<CompanyProfile, CompaniesError> {
        authorize(
            self.authz.as_ref(),
            ctx,
            permissions::PROFILE_MANAGE,
            company_id,
        )
        .await?;

        if let Some(tenancy) = &self.tenancy {
            tenancy
                .get_company(company_id)
                .await
                .map_err(|_| CompaniesError::not_found("company"))?;
        }

        if let Some(existing) = self.profiles.get(company_id).await? {
            return Ok(existing);
        }

        let now = Utc::now();
        let profile = CompanyProfile {
            company_id,
            tenant_id: ctx.tenant_id,
            status: ProfileStatus::Active,
            trade_name: None,
            website: None,
            notes: None,
            created_at: now,
            updated_at: now,
            version: 1,
        };
        self.profiles.upsert(&profile).await?;

        if self.safety_settings.get(company_id).await?.is_none() {
            self.safety_settings
                .upsert(&SafetySettings::defaults(company_id, ctx.tenant_id, now))
                .await?;
        }
        if self.regional_settings.get(company_id).await?.is_none() {
            self.regional_settings
                .upsert(&RegionalSettings::defaults(company_id, ctx.tenant_id, now))
                .await?;
        }
        if self.notification_defaults.get(company_id).await?.is_none() {
            self.notification_defaults
                .upsert(&NotificationDefaults::defaults(
                    company_id,
                    ctx.tenant_id,
                    now,
                ))
                .await?;
        }
        if self.storage_configuration.get(company_id).await?.is_none() {
            self.storage_configuration
                .upsert(&StorageConfiguration::defaults(
                    company_id,
                    ctx.tenant_id,
                    now,
                ))
                .await?;
        }

        self.outbox
            .publish(EventEnvelope::new(
                ctx.tenant_id,
                ActorRef::Principal {
                    principal_id: ctx.principal,
                },
                ResourceRef {
                    resource_type: "company_profile".to_string(),
                    resource_id: company_id.as_uuid(),
                },
                None,
                None,
                CompaniesEvent::CompanyProfileEnsured {
                    tenant_id: ctx.tenant_id,
                    company_id,
                },
            ))
            .await?;

        Ok(profile)
    }

    pub async fn get_profile(&self, company_id: CompanyId) -> Result<CompanyProfile, CompaniesError> {
        self.profiles
            .get(company_id)
            .await?
            .ok_or(CompaniesError::NotFound("company_profile"))
    }

    pub async fn update_profile(
        &self,
        ctx: &ActingContext,
        cmd: UpdateProfileCommand,
    ) -> Result<CompanyProfile, CompaniesError> {
        authorize(
            self.authz.as_ref(),
            ctx,
            permissions::PROFILE_MANAGE,
            cmd.company_id,
        )
        .await?;

        let mut profile = self.get_profile(cmd.company_id).await?;
        if profile.status == ProfileStatus::Archived {
            return Err(CompaniesError::conflict("profile is archived"));
        }

        if let Some(trade_name) = cmd.trade_name {
            profile.trade_name = Some(trade_name);
        }
        if let Some(website) = cmd.website {
            profile.website = Some(website);
        }
        if let Some(notes) = cmd.notes {
            profile.notes = Some(notes);
        }
        profile.updated_at = Utc::now();
        profile.version += 1;
        self.profiles.upsert(&profile).await?;

        self.outbox
            .publish(EventEnvelope::new(
                ctx.tenant_id,
                ActorRef::Principal {
                    principal_id: ctx.principal,
                },
                ResourceRef {
                    resource_type: "company_profile".to_string(),
                    resource_id: cmd.company_id.as_uuid(),
                },
                None,
                None,
                CompaniesEvent::CompanyProfileUpdated {
                    tenant_id: ctx.tenant_id,
                    company_id: cmd.company_id,
                },
            ))
            .await?;

        Ok(profile)
    }

    pub async fn archive_profile(
        &self,
        ctx: &ActingContext,
        company_id: CompanyId,
    ) -> Result<CompanyProfile, CompaniesError> {
        authorize(
            self.authz.as_ref(),
            ctx,
            permissions::PROFILE_MANAGE,
            company_id,
        )
        .await?;

        let mut profile = self.get_profile(company_id).await?;
        profile.status = ProfileStatus::Archived;
        profile.updated_at = Utc::now();
        profile.version += 1;
        self.profiles.upsert(&profile).await?;

        self.outbox
            .publish(EventEnvelope::new(
                ctx.tenant_id,
                ActorRef::Principal {
                    principal_id: ctx.principal,
                },
                ResourceRef {
                    resource_type: "company_profile".to_string(),
                    resource_id: company_id.as_uuid(),
                },
                None,
                None,
                CompaniesEvent::CompanyProfileArchived {
                    tenant_id: ctx.tenant_id,
                    company_id,
                },
            ))
            .await?;

        Ok(profile)
    }
}
