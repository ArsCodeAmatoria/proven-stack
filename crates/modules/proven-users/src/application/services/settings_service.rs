//! `SettingsService` — arbitrary user-scoped key/value settings bag (ADR-0006 §3).

use std::sync::Arc;

use chrono::Utc;
use proven_core::AuthzApi;
use proven_shared::UserId;

use crate::application::ports::UserSettingRepository;
use crate::application::services::audit_recorder::AuditRecorder;
use crate::application::services::authz::{authorize_self_or_permission, ActingContext};
use crate::domain::permissions;
use crate::domain::validation::validate_non_empty;
use crate::domain::{UserSetting, UsersError};
use crate::events::UsersEvent;

pub struct UpsertUserSettingCommand {
    pub user_id: UserId,
    pub key: String,
    pub value: serde_json::Value,
}

pub struct SettingsService {
    settings: Arc<dyn UserSettingRepository>,
    audit: Arc<AuditRecorder>,
    authz: Arc<dyn AuthzApi>,
}

impl SettingsService {
    pub fn new(
        settings: Arc<dyn UserSettingRepository>,
        audit: Arc<AuditRecorder>,
        authz: Arc<dyn AuthzApi>,
    ) -> Self {
        Self {
            settings,
            audit,
            authz,
        }
    }

    pub async fn get(&self, user_id: UserId, key: &str) -> Result<UserSetting, UsersError> {
        self.settings
            .get(user_id, key)
            .await?
            .ok_or(UsersError::NotFound("user_setting"))
    }

    pub async fn list(&self, user_id: UserId) -> Result<Vec<UserSetting>, UsersError> {
        self.settings.list(user_id).await
    }

    pub async fn upsert(
        &self,
        ctx: &ActingContext,
        cmd: UpsertUserSettingCommand,
    ) -> Result<UserSetting, UsersError> {
        authorize_self_or_permission(
            self.authz.as_ref(),
            ctx,
            permissions::SETTINGS_MANAGE,
            cmd.user_id,
        )
        .await?;

        validate_non_empty("key", &cmd.key)?;

        let setting = UserSetting {
            user_id: cmd.user_id,
            tenant_id: ctx.tenant_id,
            key: cmd.key.clone(),
            value: cmd.value,
            updated_at: Utc::now(),
        };
        self.settings.upsert(&setting).await?;

        self.audit
            .record(
                ctx,
                cmd.user_id,
                "user_setting_upserted",
                "user_setting",
                None,
                format!("Setting '{}' upserted", cmd.key),
                serde_json::json!({ "key": cmd.key }),
                UsersEvent::UserSettingUpserted {
                    tenant_id: ctx.tenant_id,
                    user_id: cmd.user_id,
                    key: cmd.key,
                },
            )
            .await?;

        Ok(setting)
    }
}
