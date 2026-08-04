//! `SafetySettingsService` — company-wide safety program defaults (ADR-0005 §3).

use std::sync::Arc;

use chrono::Utc;
use proven_core::AuthzApi;
use proven_shared::CompanyId;

use crate::application::ports::{EventPublisher, SafetySettingsRepository};
use crate::application::services::authz::{authorize, ActingContext};
use crate::domain::validation::{validate_email, validate_non_empty};
use crate::domain::{permissions, CompaniesError, SafetySettings};
use crate::events::{ActorRef, CompaniesEvent, EventEnvelope, ResourceRef};

pub struct UpsertSafetySettingsCommand {
    pub company_id: CompanyId,
    pub require_flha_before_work: Option<bool>,
    pub require_toolbox_talk_weekly: Option<bool>,
    pub incident_notify_emails: Option<Vec<String>>,
    pub default_risk_matrix: Option<String>,
    pub allow_offline_safety_submit: Option<bool>,
}

pub struct SafetySettingsService {
    settings: Arc<dyn SafetySettingsRepository>,
    outbox: Arc<dyn EventPublisher>,
    authz: Arc<dyn AuthzApi>,
}

impl SafetySettingsService {
    pub fn new(
        settings: Arc<dyn SafetySettingsRepository>,
        outbox: Arc<dyn EventPublisher>,
        authz: Arc<dyn AuthzApi>,
    ) -> Self {
        Self {
            settings,
            outbox,
            authz,
        }
    }

    pub async fn get(&self, company_id: CompanyId) -> Result<SafetySettings, CompaniesError> {
        self.settings
            .get(company_id)
            .await?
            .ok_or(CompaniesError::NotFound("safety_settings"))
    }

    pub async fn upsert(
        &self,
        ctx: &ActingContext,
        cmd: UpsertSafetySettingsCommand,
    ) -> Result<SafetySettings, CompaniesError> {
        authorize(
            self.authz.as_ref(),
            ctx,
            permissions::SAFETY_SETTINGS_MANAGE,
            cmd.company_id,
        )
        .await?;
        if let Some(emails) = &cmd.incident_notify_emails {
            for email in emails {
                validate_email(email)?;
            }
        }
        if let Some(default_risk_matrix) = &cmd.default_risk_matrix {
            validate_non_empty("default_risk_matrix", default_risk_matrix)?;
        }

        let now = Utc::now();
        let mut settings = self
            .settings
            .get(cmd.company_id)
            .await?
            .unwrap_or_else(|| SafetySettings::defaults(cmd.company_id, ctx.tenant_id, now));

        if let Some(require_flha_before_work) = cmd.require_flha_before_work {
            settings.require_flha_before_work = require_flha_before_work;
        }
        if let Some(require_toolbox_talk_weekly) = cmd.require_toolbox_talk_weekly {
            settings.require_toolbox_talk_weekly = require_toolbox_talk_weekly;
        }
        if let Some(emails) = cmd.incident_notify_emails {
            settings.incident_notify_emails = emails;
        }
        if let Some(default_risk_matrix) = cmd.default_risk_matrix {
            settings.default_risk_matrix = default_risk_matrix;
        }
        if let Some(allow_offline_safety_submit) = cmd.allow_offline_safety_submit {
            settings.allow_offline_safety_submit = allow_offline_safety_submit;
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
                    resource_type: "safety_settings".to_string(),
                    resource_id: cmd.company_id.as_uuid(),
                },
                None,
                None,
                CompaniesEvent::SafetySettingsUpdated {
                    tenant_id: ctx.tenant_id,
                    company_id: cmd.company_id,
                },
            ))
            .await?;

        Ok(settings)
    }
}
