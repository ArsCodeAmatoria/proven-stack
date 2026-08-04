//! `ProfileService` — account profile lifecycle + first-run default preference provisioning
//! (ADR-0006 §3).

use std::sync::Arc;

use chrono::Utc;
use proven_core::AuthzApi;
use proven_core::IdentityApi;
use proven_shared::{TenantId, UserId};

use crate::application::ports::{
    AccessibilityRepository, AuthenticationProfileRepository, LocaleRepository,
    NotificationRepository, SignatureProfileRepository, UserProfileRepository,
};
use crate::application::services::audit_recorder::AuditRecorder;
use crate::application::services::authz::{authorize, authorize_self_or_permission, ActingContext};
use crate::domain::permissions;
use crate::domain::{
    AccessibilityPreferences, AuthenticationProfile, DigitalSignatureProfile, LocalePreferences,
    NotificationPreferences, ProfileStatus, UserProfile, UsersError,
};
use crate::events::UsersEvent;

pub struct UpdateProfileCommand {
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub preferred_name: Option<String>,
    pub job_title: Option<String>,
    pub phone: Option<String>,
    pub bio: Option<String>,
}

pub struct ProfileService {
    profiles: Arc<dyn UserProfileRepository>,
    locale: Arc<dyn LocaleRepository>,
    accessibility: Arc<dyn AccessibilityRepository>,
    notification: Arc<dyn NotificationRepository>,
    auth_profile: Arc<dyn AuthenticationProfileRepository>,
    signature_profile: Arc<dyn SignatureProfileRepository>,
    audit: Arc<AuditRecorder>,
    authz: Arc<dyn AuthzApi>,
    identity: Option<Arc<dyn IdentityApi>>,
}

impl ProfileService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        profiles: Arc<dyn UserProfileRepository>,
        locale: Arc<dyn LocaleRepository>,
        accessibility: Arc<dyn AccessibilityRepository>,
        notification: Arc<dyn NotificationRepository>,
        auth_profile: Arc<dyn AuthenticationProfileRepository>,
        signature_profile: Arc<dyn SignatureProfileRepository>,
        audit: Arc<AuditRecorder>,
        authz: Arc<dyn AuthzApi>,
        identity: Option<Arc<dyn IdentityApi>>,
    ) -> Self {
        Self {
            profiles,
            locale,
            accessibility,
            notification,
            auth_profile,
            signature_profile,
            audit,
            authz,
            identity,
        }
    }

    /// Idempotently ensures a profile shell (+ default locale/accessibility/notification/
    /// auth/signature rows) exists for `user_id`. Returns the existing profile if one is already
    /// present. When an `IdentityApi` is wired, verifies the Core `User` exists first.
    pub async fn ensure_profile(
        &self,
        ctx: &ActingContext,
        user_id: UserId,
        display_name: String,
    ) -> Result<UserProfile, UsersError> {
        authorize(self.authz.as_ref(), ctx, permissions::PROFILE_MANAGE).await?;

        if let Some(identity) = &self.identity {
            identity
                .get_user(ctx.tenant_id, user_id)
                .await
                .map_err(|_| UsersError::not_found("core_user"))?;
        }

        if let Some(existing) = self.profiles.get(user_id).await? {
            return Ok(existing);
        }

        let now = Utc::now();
        let profile = UserProfile {
            user_id,
            tenant_id: ctx.tenant_id,
            status: ProfileStatus::Active,
            display_name,
            preferred_name: None,
            job_title: None,
            phone: None,
            company_id: None,
            person_id: None,
            bio: None,
            created_at: now,
            updated_at: now,
            version: 1,
        };
        self.profiles.upsert(&profile).await?;

        self.ensure_default_preferences(user_id, ctx.tenant_id, now)
            .await?;

        self.audit
            .record(
                ctx,
                user_id,
                "profile_ensured",
                "user_profile",
                None,
                "Account profile created",
                serde_json::json!({ "display_name": profile.display_name }),
                UsersEvent::UserProfileEnsured {
                    tenant_id: ctx.tenant_id,
                    user_id,
                },
            )
            .await?;

        Ok(profile)
    }

    async fn ensure_default_preferences(
        &self,
        user_id: UserId,
        tenant_id: TenantId,
        now: chrono::DateTime<Utc>,
    ) -> Result<(), UsersError> {
        if self.locale.get(user_id).await?.is_none() {
            self.locale
                .upsert(&LocalePreferences::defaults(user_id, tenant_id, now))
                .await?;
        }
        if self.accessibility.get(user_id).await?.is_none() {
            self.accessibility
                .upsert(&AccessibilityPreferences::defaults(user_id, tenant_id, now))
                .await?;
        }
        if self.notification.get(user_id).await?.is_none() {
            self.notification
                .upsert(&NotificationPreferences::defaults(user_id, tenant_id, now))
                .await?;
        }
        if self.auth_profile.get(user_id).await?.is_none() {
            self.auth_profile
                .upsert(&AuthenticationProfile::defaults(user_id, tenant_id, now))
                .await?;
        }
        if self.signature_profile.get(user_id).await?.is_none() {
            self.signature_profile
                .upsert(&DigitalSignatureProfile::defaults(user_id, tenant_id, now))
                .await?;
        }
        Ok(())
    }

    pub async fn get_profile(&self, user_id: UserId) -> Result<UserProfile, UsersError> {
        self.profiles
            .get(user_id)
            .await?
            .ok_or(UsersError::NotFound("user_profile"))
    }

    pub async fn update_profile(
        &self,
        ctx: &ActingContext,
        cmd: UpdateProfileCommand,
    ) -> Result<UserProfile, UsersError> {
        authorize_self_or_permission(
            self.authz.as_ref(),
            ctx,
            permissions::PROFILE_MANAGE,
            cmd.user_id,
        )
        .await?;

        let mut profile = self.get_profile(cmd.user_id).await?;
        if profile.status == ProfileStatus::Archived {
            return Err(UsersError::conflict("profile is archived"));
        }

        if let Some(display_name) = cmd.display_name {
            crate::domain::validation::validate_non_empty("display_name", &display_name)?;
            profile.display_name = display_name;
        }
        if let Some(preferred_name) = cmd.preferred_name {
            profile.preferred_name = Some(preferred_name);
        }
        if let Some(job_title) = cmd.job_title {
            profile.job_title = Some(job_title);
        }
        if let Some(phone) = cmd.phone {
            profile.phone = Some(phone);
        }
        if let Some(bio) = cmd.bio {
            profile.bio = Some(bio);
        }
        profile.updated_at = Utc::now();
        profile.version += 1;
        self.profiles.upsert(&profile).await?;

        self.audit
            .record(
                ctx,
                cmd.user_id,
                "profile_updated",
                "user_profile",
                None,
                "Account profile updated",
                serde_json::json!({}),
                UsersEvent::UserProfileUpdated {
                    tenant_id: ctx.tenant_id,
                    user_id: cmd.user_id,
                },
            )
            .await?;

        Ok(profile)
    }

    pub async fn archive_profile(
        &self,
        ctx: &ActingContext,
        user_id: UserId,
    ) -> Result<UserProfile, UsersError> {
        authorize(self.authz.as_ref(), ctx, permissions::PROFILE_MANAGE).await?;

        let mut profile = self.get_profile(user_id).await?;
        profile.status = ProfileStatus::Archived;
        profile.updated_at = Utc::now();
        profile.version += 1;
        self.profiles.upsert(&profile).await?;

        self.audit
            .record(
                ctx,
                user_id,
                "profile_archived",
                "user_profile",
                None,
                "Account profile archived",
                serde_json::json!({}),
                UsersEvent::UserProfileArchived {
                    tenant_id: ctx.tenant_id,
                    user_id,
                },
            )
            .await?;

        Ok(profile)
    }
}
