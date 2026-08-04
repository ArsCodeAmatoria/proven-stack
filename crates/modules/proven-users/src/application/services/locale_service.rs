//! `LocaleService` — language/timezone preferences (ADR-0006 §3).

use std::sync::Arc;

use chrono::Utc;
use proven_core::AuthzApi;
use proven_shared::UserId;

use crate::application::ports::LocaleRepository;
use crate::application::services::audit_recorder::AuditRecorder;
use crate::application::services::authz::{authorize_self_or_permission, ActingContext};
use crate::domain::permissions;
use crate::domain::validation::validate_non_empty;
use crate::domain::{LocalePreferences, UsersError};
use crate::events::UsersEvent;

pub struct UpsertLocaleCommand {
    pub user_id: UserId,
    pub language_code: Option<String>,
    pub time_zone: Option<String>,
}

pub struct LocaleService {
    locale: Arc<dyn LocaleRepository>,
    audit: Arc<AuditRecorder>,
    authz: Arc<dyn AuthzApi>,
}

impl LocaleService {
    pub fn new(
        locale: Arc<dyn LocaleRepository>,
        audit: Arc<AuditRecorder>,
        authz: Arc<dyn AuthzApi>,
    ) -> Self {
        Self {
            locale,
            audit,
            authz,
        }
    }

    pub async fn get(&self, user_id: UserId) -> Result<LocalePreferences, UsersError> {
        self.locale
            .get(user_id)
            .await?
            .ok_or(UsersError::NotFound("locale_preferences"))
    }

    pub async fn upsert(
        &self,
        ctx: &ActingContext,
        cmd: UpsertLocaleCommand,
    ) -> Result<LocalePreferences, UsersError> {
        authorize_self_or_permission(
            self.authz.as_ref(),
            ctx,
            permissions::PREFERENCES_MANAGE,
            cmd.user_id,
        )
        .await?;

        let mut prefs =
            self.locale.get(cmd.user_id).await?.unwrap_or_else(|| {
                LocalePreferences::defaults(cmd.user_id, ctx.tenant_id, Utc::now())
            });

        if let Some(language_code) = cmd.language_code {
            validate_non_empty("language_code", &language_code)?;
            prefs.language_code = language_code;
        }
        if let Some(time_zone) = cmd.time_zone {
            validate_non_empty("time_zone", &time_zone)?;
            prefs.time_zone = time_zone;
        }
        prefs.updated_at = Utc::now();
        self.locale.upsert(&prefs).await?;

        self.audit
            .record(
                ctx,
                cmd.user_id,
                "locale_updated",
                "locale_preferences",
                None,
                "Locale preferences updated",
                serde_json::json!({
                    "language_code": prefs.language_code,
                    "time_zone": prefs.time_zone,
                }),
                UsersEvent::LocaleUpdated {
                    tenant_id: ctx.tenant_id,
                    user_id: cmd.user_id,
                },
            )
            .await?;

        Ok(prefs)
    }
}
