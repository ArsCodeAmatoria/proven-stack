//! `NotificationService` — notification channel preferences (ADR-0006 §3).

use std::sync::Arc;

use chrono::Utc;
use proven_core::AuthzApi;
use proven_shared::UserId;

use crate::application::ports::NotificationRepository;
use crate::application::services::audit_recorder::AuditRecorder;
use crate::application::services::authz::{authorize_self_or_permission, ActingContext};
use crate::domain::permissions;
use crate::domain::validation::validate_hhmm;
use crate::domain::{DigestCadence, NotificationPreferences, UsersError};
use crate::events::UsersEvent;

pub struct UpsertNotificationPreferencesCommand {
    pub user_id: UserId,
    pub email_enabled: Option<bool>,
    pub push_enabled: Option<bool>,
    pub sms_enabled: Option<bool>,
    pub in_app_enabled: Option<bool>,
    pub digest_cadence: Option<DigestCadence>,
    pub quiet_hours_start: Option<String>,
    pub quiet_hours_end: Option<String>,
}

pub struct NotificationService {
    notification: Arc<dyn NotificationRepository>,
    audit: Arc<AuditRecorder>,
    authz: Arc<dyn AuthzApi>,
}

impl NotificationService {
    pub fn new(
        notification: Arc<dyn NotificationRepository>,
        audit: Arc<AuditRecorder>,
        authz: Arc<dyn AuthzApi>,
    ) -> Self {
        Self {
            notification,
            audit,
            authz,
        }
    }

    pub async fn get(&self, user_id: UserId) -> Result<NotificationPreferences, UsersError> {
        self.notification
            .get(user_id)
            .await?
            .ok_or(UsersError::NotFound("notification_preferences"))
    }

    pub async fn upsert(
        &self,
        ctx: &ActingContext,
        cmd: UpsertNotificationPreferencesCommand,
    ) -> Result<NotificationPreferences, UsersError> {
        authorize_self_or_permission(
            self.authz.as_ref(),
            ctx,
            permissions::PREFERENCES_MANAGE,
            cmd.user_id,
        )
        .await?;

        if let Some(start) = &cmd.quiet_hours_start {
            validate_hhmm("quiet_hours_start", start)?;
        }
        if let Some(end) = &cmd.quiet_hours_end {
            validate_hhmm("quiet_hours_end", end)?;
        }

        let mut prefs = self
            .notification
            .get(cmd.user_id)
            .await?
            .unwrap_or_else(|| {
                NotificationPreferences::defaults(cmd.user_id, ctx.tenant_id, Utc::now())
            });

        if let Some(email_enabled) = cmd.email_enabled {
            prefs.email_enabled = email_enabled;
        }
        if let Some(push_enabled) = cmd.push_enabled {
            prefs.push_enabled = push_enabled;
        }
        if let Some(sms_enabled) = cmd.sms_enabled {
            prefs.sms_enabled = sms_enabled;
        }
        if let Some(in_app_enabled) = cmd.in_app_enabled {
            prefs.in_app_enabled = in_app_enabled;
        }
        if let Some(digest_cadence) = cmd.digest_cadence {
            prefs.digest_cadence = digest_cadence;
        }
        if cmd.quiet_hours_start.is_some() {
            prefs.quiet_hours_start = cmd.quiet_hours_start;
        }
        if cmd.quiet_hours_end.is_some() {
            prefs.quiet_hours_end = cmd.quiet_hours_end;
        }
        prefs.updated_at = Utc::now();
        self.notification.upsert(&prefs).await?;

        self.audit
            .record(
                ctx,
                cmd.user_id,
                "notification_preferences_updated",
                "notification_preferences",
                None,
                "Notification preferences updated",
                serde_json::json!({}),
                UsersEvent::NotificationPreferencesUpdated {
                    tenant_id: ctx.tenant_id,
                    user_id: cmd.user_id,
                },
            )
            .await?;

        Ok(prefs)
    }
}
