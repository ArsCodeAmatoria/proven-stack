//! `AccessibilityService` — accessibility preferences (ADR-0006 §3).

use std::sync::Arc;

use chrono::Utc;
use proven_core::AuthzApi;
use proven_shared::UserId;

use crate::application::ports::AccessibilityRepository;
use crate::application::services::audit_recorder::AuditRecorder;
use crate::application::services::authz::{authorize_self_or_permission, ActingContext};
use crate::domain::permissions;
use crate::domain::{AccessibilityPreferences, UsersError};
use crate::events::UsersEvent;

pub struct UpsertAccessibilityCommand {
    pub user_id: UserId,
    pub reduce_motion: Option<bool>,
    pub high_contrast: Option<bool>,
    pub large_text: Option<bool>,
    pub screen_reader_hints: Option<bool>,
}

pub struct AccessibilityService {
    accessibility: Arc<dyn AccessibilityRepository>,
    audit: Arc<AuditRecorder>,
    authz: Arc<dyn AuthzApi>,
}

impl AccessibilityService {
    pub fn new(
        accessibility: Arc<dyn AccessibilityRepository>,
        audit: Arc<AuditRecorder>,
        authz: Arc<dyn AuthzApi>,
    ) -> Self {
        Self {
            accessibility,
            audit,
            authz,
        }
    }

    pub async fn get(&self, user_id: UserId) -> Result<AccessibilityPreferences, UsersError> {
        self.accessibility
            .get(user_id)
            .await?
            .ok_or(UsersError::NotFound("accessibility_preferences"))
    }

    pub async fn upsert(
        &self,
        ctx: &ActingContext,
        cmd: UpsertAccessibilityCommand,
    ) -> Result<AccessibilityPreferences, UsersError> {
        authorize_self_or_permission(
            self.authz.as_ref(),
            ctx,
            permissions::PREFERENCES_MANAGE,
            cmd.user_id,
        )
        .await?;

        let mut prefs = self
            .accessibility
            .get(cmd.user_id)
            .await?
            .unwrap_or_else(|| {
                AccessibilityPreferences::defaults(cmd.user_id, ctx.tenant_id, Utc::now())
            });

        if let Some(reduce_motion) = cmd.reduce_motion {
            prefs.reduce_motion = reduce_motion;
        }
        if let Some(high_contrast) = cmd.high_contrast {
            prefs.high_contrast = high_contrast;
        }
        if let Some(large_text) = cmd.large_text {
            prefs.large_text = large_text;
        }
        if let Some(screen_reader_hints) = cmd.screen_reader_hints {
            prefs.screen_reader_hints = screen_reader_hints;
        }
        prefs.updated_at = Utc::now();
        self.accessibility.upsert(&prefs).await?;

        self.audit
            .record(
                ctx,
                cmd.user_id,
                "accessibility_updated",
                "accessibility_preferences",
                None,
                "Accessibility preferences updated",
                serde_json::json!({}),
                UsersEvent::AccessibilityUpdated {
                    tenant_id: ctx.tenant_id,
                    user_id: cmd.user_id,
                },
            )
            .await?;

        Ok(prefs)
    }
}
