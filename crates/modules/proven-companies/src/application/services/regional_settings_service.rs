//! `RegionalSettingsService` — company localization defaults (ADR-0005 §3).

use std::sync::Arc;

use chrono::Utc;
use proven_core::AuthzApi;
use proven_shared::CompanyId;

use crate::application::ports::{EventPublisher, RegionalSettingsRepository};
use crate::application::services::authz::{authorize, ActingContext};
use crate::domain::validation::{validate_currency_code, validate_non_empty};
use crate::domain::{permissions, CompaniesError, MeasurementSystem, RegionalSettings};
use crate::events::{ActorRef, CompaniesEvent, EventEnvelope, ResourceRef};

pub struct UpsertRegionalSettingsCommand {
    pub company_id: CompanyId,
    pub primary_region: Option<String>,
    pub locales: Option<Vec<String>>,
    pub timezone: Option<String>,
    pub measurement_system: Option<MeasurementSystem>,
    pub currency_code: Option<String>,
}

pub struct RegionalSettingsService {
    settings: Arc<dyn RegionalSettingsRepository>,
    outbox: Arc<dyn EventPublisher>,
    authz: Arc<dyn AuthzApi>,
}

impl RegionalSettingsService {
    pub fn new(
        settings: Arc<dyn RegionalSettingsRepository>,
        outbox: Arc<dyn EventPublisher>,
        authz: Arc<dyn AuthzApi>,
    ) -> Self {
        Self {
            settings,
            outbox,
            authz,
        }
    }

    pub async fn get(&self, company_id: CompanyId) -> Result<RegionalSettings, CompaniesError> {
        self.settings
            .get(company_id)
            .await?
            .ok_or(CompaniesError::NotFound("regional_settings"))
    }

    pub async fn upsert(
        &self,
        ctx: &ActingContext,
        cmd: UpsertRegionalSettingsCommand,
    ) -> Result<RegionalSettings, CompaniesError> {
        authorize(
            self.authz.as_ref(),
            ctx,
            permissions::REGIONAL_SETTINGS_MANAGE,
            cmd.company_id,
        )
        .await?;
        if let Some(primary_region) = &cmd.primary_region {
            validate_non_empty("primary_region", primary_region)?;
        }
        if let Some(timezone) = &cmd.timezone {
            validate_non_empty("timezone", timezone)?;
        }
        if let Some(currency_code) = &cmd.currency_code {
            validate_currency_code(currency_code)?;
        }

        let now = Utc::now();
        let mut settings = self
            .settings
            .get(cmd.company_id)
            .await?
            .unwrap_or_else(|| RegionalSettings::defaults(cmd.company_id, ctx.tenant_id, now));

        if let Some(primary_region) = cmd.primary_region {
            settings.primary_region = primary_region;
        }
        if let Some(locales) = cmd.locales {
            settings.locales = locales;
        }
        if let Some(timezone) = cmd.timezone {
            settings.timezone = timezone;
        }
        if let Some(measurement_system) = cmd.measurement_system {
            settings.measurement_system = measurement_system;
        }
        if let Some(currency_code) = cmd.currency_code {
            settings.currency_code = currency_code;
        }
        settings.updated_at = now;
        self.settings.upsert(&settings).await?;

        self.outbox
            .publish(EventEnvelope::new(
                ctx.tenant_id,
                ActorRef::Principal {
                    principal_id: ctx.principal,
                },
                ResourceRef {
                    resource_type: "regional_settings".to_string(),
                    resource_id: cmd.company_id.as_uuid(),
                },
                None,
                None,
                CompaniesEvent::RegionalSettingsUpdated {
                    tenant_id: ctx.tenant_id,
                    company_id: cmd.company_id,
                },
            ))
            .await?;

        Ok(settings)
    }
}
