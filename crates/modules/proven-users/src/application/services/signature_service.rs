//! `SignatureService` — digital signing preferences/assurance hints (ADR-0006 §7). Never stores
//! executed signature packages — those belong to the (future) Signatures module, see
//! `domain::ownership`.

use std::sync::Arc;

use chrono::Utc;
use proven_core::AuthzApi;
use proven_shared::{FileObjectId, UserId};

use crate::application::ports::SignatureProfileRepository;
use crate::application::services::audit_recorder::AuditRecorder;
use crate::application::services::authz::{authorize_self_or_permission, ActingContext};
use crate::domain::permissions;
use crate::domain::{DigitalSignatureProfile, SignatureType, UsersError};
use crate::events::UsersEvent;

pub struct UpsertSignatureProfileCommand {
    pub user_id: UserId,
    pub default_signature_type: Option<SignatureType>,
    pub typed_name_default: Option<String>,
    pub signature_image_file_id: Option<FileObjectId>,
    pub require_reauth_to_sign: Option<bool>,
}

pub struct SignatureService {
    profiles: Arc<dyn SignatureProfileRepository>,
    audit: Arc<AuditRecorder>,
    authz: Arc<dyn AuthzApi>,
}

impl SignatureService {
    pub fn new(
        profiles: Arc<dyn SignatureProfileRepository>,
        audit: Arc<AuditRecorder>,
        authz: Arc<dyn AuthzApi>,
    ) -> Self {
        Self {
            profiles,
            audit,
            authz,
        }
    }

    pub async fn get(&self, user_id: UserId) -> Result<DigitalSignatureProfile, UsersError> {
        self.profiles
            .get(user_id)
            .await?
            .ok_or(UsersError::NotFound("digital_signature_profile"))
    }

    pub async fn upsert(
        &self,
        ctx: &ActingContext,
        cmd: UpsertSignatureProfileCommand,
    ) -> Result<DigitalSignatureProfile, UsersError> {
        authorize_self_or_permission(
            self.authz.as_ref(),
            ctx,
            permissions::SIGNATURE_PROFILE_MANAGE,
            cmd.user_id,
        )
        .await?;

        let mut profile = self.profiles.get(cmd.user_id).await?.unwrap_or_else(|| {
            DigitalSignatureProfile::defaults(cmd.user_id, ctx.tenant_id, Utc::now())
        });

        if let Some(default_signature_type) = cmd.default_signature_type {
            profile.default_signature_type = default_signature_type;
        }
        if cmd.typed_name_default.is_some() {
            profile.typed_name_default = cmd.typed_name_default;
        }
        if cmd.signature_image_file_id.is_some() {
            profile.signature_image_file_id = cmd.signature_image_file_id;
        }
        if let Some(require_reauth_to_sign) = cmd.require_reauth_to_sign {
            profile.require_reauth_to_sign = require_reauth_to_sign;
        }
        profile.updated_at = Utc::now();
        self.profiles.upsert(&profile).await?;

        self.audit
            .record(
                ctx,
                cmd.user_id,
                "digital_signature_profile_updated",
                "digital_signature_profile",
                None,
                "Digital signature profile updated",
                serde_json::json!({}),
                UsersEvent::DigitalSignatureProfileUpdated {
                    tenant_id: ctx.tenant_id,
                    user_id: cmd.user_id,
                },
            )
            .await?;

        Ok(profile)
    }
}
