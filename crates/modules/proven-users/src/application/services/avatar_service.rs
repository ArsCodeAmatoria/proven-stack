//! `AvatarService` — user avatar pointer (ADR-0006 §3). Absent until explicitly set.

use std::sync::Arc;

use chrono::Utc;
use proven_core::AuthzApi;
use proven_shared::{FileObjectId, UserId};

use crate::application::ports::AvatarRepository;
use crate::application::services::audit_recorder::AuditRecorder;
use crate::application::services::authz::{authorize_self_or_permission, ActingContext};
use crate::domain::permissions;
use crate::domain::{Avatar, UsersError};
use crate::events::UsersEvent;

pub struct UpsertAvatarCommand {
    pub user_id: UserId,
    pub file_object_id: Option<FileObjectId>,
    pub avatar_url: Option<String>,
}

pub struct AvatarService {
    avatars: Arc<dyn AvatarRepository>,
    audit: Arc<AuditRecorder>,
    authz: Arc<dyn AuthzApi>,
}

impl AvatarService {
    pub fn new(
        avatars: Arc<dyn AvatarRepository>,
        audit: Arc<AuditRecorder>,
        authz: Arc<dyn AuthzApi>,
    ) -> Self {
        Self {
            avatars,
            audit,
            authz,
        }
    }

    pub async fn get(&self, user_id: UserId) -> Result<Avatar, UsersError> {
        self.avatars
            .get(user_id)
            .await?
            .ok_or(UsersError::NotFound("avatar"))
    }

    pub async fn upsert(
        &self,
        ctx: &ActingContext,
        cmd: UpsertAvatarCommand,
    ) -> Result<Avatar, UsersError> {
        authorize_self_or_permission(
            self.authz.as_ref(),
            ctx,
            permissions::AVATAR_MANAGE,
            cmd.user_id,
        )
        .await?;

        let avatar = Avatar {
            user_id: cmd.user_id,
            tenant_id: ctx.tenant_id,
            file_object_id: cmd.file_object_id,
            avatar_url: cmd.avatar_url,
            updated_at: Utc::now(),
        };
        self.avatars.upsert(&avatar).await?;

        self.audit
            .record(
                ctx,
                cmd.user_id,
                "avatar_updated",
                "avatar",
                None,
                "Avatar updated",
                serde_json::json!({}),
                UsersEvent::AvatarUpdated {
                    tenant_id: ctx.tenant_id,
                    user_id: cmd.user_id,
                    file_object_id: avatar.file_object_id,
                },
            )
            .await?;

        Ok(avatar)
    }
}
