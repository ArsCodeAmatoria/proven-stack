//! `NotificationDefaultsService` — company-wide notification defaults (ADR-0005 §3).

use std::sync::Arc;

use chrono::Utc;
use proven_core::AuthzApi;
use proven_shared::CompanyId;

use crate::application::ports::{EventPublisher, NotificationDefaultsRepository};
use crate::application::services::authz::{authorize, ActingContext};
use crate::domain::validation::validate_non_empty;
use crate::domain::{permissions, CompaniesError, DigestCadence, NotificationDefaults};
use crate::events::{ActorRef, CompaniesEvent, EventEnvelope, ResourceRef};

pub struct UpsertNotificationDefaultsCommand {
    pub company_id: CompanyId,
    pub email_enabled: Option<bool>,
    pub push_enabled: Option<bool>,
    pub sms_enabled: Option<bool>,
    pub digest_cadence: Option<DigestCadence>,
    pub quiet_hours_start: Option<String>,
    pub quiet_hours_end: Option<String>,
    pub default_locale: Option<String>,
}

pub struct NotificationDefaultsService {
    defaults: Arc<dyn NotificationDefaultsRepository>,
    outbox: Arc<dyn EventPublisher>,
    authz: Arc<dyn AuthzApi>,
}

impl NotificationDefaultsService {
    pub fn new(
        defaults: Arc<dyn NotificationDefaultsRepository>,
        outbox: Arc<dyn EventPublisher>,
        authz: Arc<dyn AuthzApi>,
    ) -> Self {
        Self {
            defaults,
            outbox,
            authz,
        }
    }

    pub async fn get(&self, company_id: CompanyId) -> Result<NotificationDefaults, CompaniesError> {
        self.defaults
            .get(company_id)
            .await?
            .ok_or(CompaniesError::NotFound("notification_defaults"))
    }

    pub async fn upsert(
        &self,
        ctx: &ActingContext,
        cmd: UpsertNotificationDefaultsCommand,
    ) -> Result<NotificationDefaults, CompaniesError> {
        authorize(
            self.authz.as_ref(),
            ctx,
            permissions::NOTIFICATION_DEFAULTS_MANAGE,
            cmd.company_id,
        )
        .await?;
        if let Some(default_locale) = &cmd.default_locale {
            validate_non_empty("default_locale", default_locale)?;
        }

        let now = Utc::now();
        let mut defaults = self
            .defaults
            .get(cmd.company_id)
            .await?
            .unwrap_or_else(|| NotificationDefaults::defaults(cmd.company_id, ctx.tenant_id, now));

        if let Some(email_enabled) = cmd.email_enabled {
            defaults.email_enabled = email_enabled;
        }
        if let Some(push_enabled) = cmd.push_enabled {
            defaults.push_enabled = push_enabled;
        }
        if let Some(sms_enabled) = cmd.sms_enabled {
            defaults.sms_enabled = sms_enabled;
        }
        if let Some(digest_cadence) = cmd.digest_cadence {
            defaults.digest_cadence = digest_cadence;
        }
        if cmd.quiet_hours_start.is_some() {
            defaults.quiet_hours_start = cmd.quiet_hours_start;
        }
        if cmd.quiet_hours_end.is_some() {
            defaults.quiet_hours_end = cmd.quiet_hours_end;
        }
        if let Some(default_locale) = cmd.default_locale {
            defaults.default_locale = default_locale;
        }
        defaults.updated_at = now;
        self.defaults.upsert(&defaults).await?;

        self.outbox
            .publish(EventEnvelope::new(
                ctx.tenant_id,
                ActorRef::Principal {
                    principal_id: ctx.principal,
                },
                ResourceRef {
                    resource_type: "notification_defaults".to_string(),
                    resource_id: cmd.company_id.as_uuid(),
                },
                None,
                None,
                CompaniesEvent::NotificationDefaultsUpdated {
                    tenant_id: ctx.tenant_id,
                    company_id: cmd.company_id,
                },
            ))
            .await?;

        Ok(defaults)
    }
}
