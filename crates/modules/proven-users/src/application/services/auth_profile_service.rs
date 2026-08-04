//! `AuthProfileService` — authentication *preference* mirror flags (ADR-0006 §6). Never stores
//! password hashes or credentials; Core remains the authentication System of Record — see
//! `domain::ownership`.

use std::sync::Arc;

use chrono::Utc;
use proven_core::AuthzApi;
use proven_shared::UserId;

use crate::application::ports::AuthenticationProfileRepository;
use crate::application::services::audit_recorder::AuditRecorder;
use crate::application::services::authz::{authorize_self_or_permission, ActingContext};
use crate::domain::permissions;
use crate::domain::{AuthenticationProfile, UsersError};
use crate::events::UsersEvent;

pub struct UpsertAuthenticationProfileCommand {
    pub user_id: UserId,
    pub mfa_preferred: Option<bool>,
    pub password_login_enabled: Option<bool>,
    pub oauth_google_linked: Option<bool>,
    pub oauth_microsoft_linked: Option<bool>,
    pub magic_link_preferred: Option<bool>,
    pub last_auth_method: Option<String>,
}

pub struct AuthProfileService {
    profiles: Arc<dyn AuthenticationProfileRepository>,
    audit: Arc<AuditRecorder>,
    authz: Arc<dyn AuthzApi>,
}

impl AuthProfileService {
    pub fn new(
        profiles: Arc<dyn AuthenticationProfileRepository>,
        audit: Arc<AuditRecorder>,
        authz: Arc<dyn AuthzApi>,
    ) -> Self {
        Self {
            profiles,
            audit,
            authz,
        }
    }

    pub async fn get(&self, user_id: UserId) -> Result<AuthenticationProfile, UsersError> {
        self.profiles
            .get(user_id)
            .await?
            .ok_or(UsersError::NotFound("authentication_profile"))
    }

    pub async fn upsert(
        &self,
        ctx: &ActingContext,
        cmd: UpsertAuthenticationProfileCommand,
    ) -> Result<AuthenticationProfile, UsersError> {
        authorize_self_or_permission(
            self.authz.as_ref(),
            ctx,
            permissions::AUTH_PROFILE_MANAGE,
            cmd.user_id,
        )
        .await?;

        let mut profile = self.profiles.get(cmd.user_id).await?.unwrap_or_else(|| {
            AuthenticationProfile::defaults(cmd.user_id, ctx.tenant_id, Utc::now())
        });

        if let Some(mfa_preferred) = cmd.mfa_preferred {
            profile.mfa_preferred = mfa_preferred;
        }
        if let Some(password_login_enabled) = cmd.password_login_enabled {
            profile.password_login_enabled = password_login_enabled;
        }
        if let Some(oauth_google_linked) = cmd.oauth_google_linked {
            profile.oauth_google_linked = oauth_google_linked;
        }
        if let Some(oauth_microsoft_linked) = cmd.oauth_microsoft_linked {
            profile.oauth_microsoft_linked = oauth_microsoft_linked;
        }
        if let Some(magic_link_preferred) = cmd.magic_link_preferred {
            profile.magic_link_preferred = magic_link_preferred;
        }
        if cmd.last_auth_method.is_some() {
            profile.last_auth_method = cmd.last_auth_method;
        }
        profile.updated_at = Utc::now();
        self.profiles.upsert(&profile).await?;

        self.audit
            .record(
                ctx,
                cmd.user_id,
                "authentication_profile_updated",
                "authentication_profile",
                None,
                "Authentication preference flags updated",
                serde_json::json!({}),
                UsersEvent::AuthenticationProfileUpdated {
                    tenant_id: ctx.tenant_id,
                    user_id: cmd.user_id,
                },
            )
            .await?;

        Ok(profile)
    }
}
