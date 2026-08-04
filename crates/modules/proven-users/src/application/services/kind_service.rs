//! `KindService` — `UserKind` classification tag assignment (ADR-0006 §4). These are profile
//! tags for UX/directory only; see `domain::ownership` for why they are never consulted by
//! AuthZ. Administrator-only: unlike preferences, a user cannot self-assign their own kind.

use std::sync::Arc;

use chrono::Utc;
use proven_core::AuthzApi;
use proven_shared::UserId;

use crate::application::ports::{UserKindRepository, UserProfileRepository};
use crate::application::services::audit_recorder::AuditRecorder;
use crate::application::services::authz::{authorize, ActingContext};
use crate::domain::permissions;
use crate::domain::{UserKind, UserKindAssignment, UserKindAssignmentId, UsersError};
use crate::events::UsersEvent;

pub struct AssignUserKindCommand {
    pub user_id: UserId,
    pub kind: UserKind,
    pub is_primary: bool,
}

pub struct KindService {
    kinds: Arc<dyn UserKindRepository>,
    profiles: Arc<dyn UserProfileRepository>,
    audit: Arc<AuditRecorder>,
    authz: Arc<dyn AuthzApi>,
}

impl KindService {
    pub fn new(
        kinds: Arc<dyn UserKindRepository>,
        profiles: Arc<dyn UserProfileRepository>,
        audit: Arc<AuditRecorder>,
        authz: Arc<dyn AuthzApi>,
    ) -> Self {
        Self {
            kinds,
            profiles,
            audit,
            authz,
        }
    }

    pub async fn assign(
        &self,
        ctx: &ActingContext,
        cmd: AssignUserKindCommand,
    ) -> Result<UserKindAssignment, UsersError> {
        authorize(self.authz.as_ref(), ctx, permissions::KIND_MANAGE).await?;

        self.profiles
            .get(cmd.user_id)
            .await?
            .ok_or(UsersError::NotFound("user_profile"))?;

        if cmd.is_primary {
            self.kinds.clear_primary(cmd.user_id).await?;
        }

        let existing = self.kinds.get(cmd.user_id, cmd.kind).await?;
        let assignment = UserKindAssignment {
            id: existing
                .map(|a| a.id)
                .unwrap_or_else(UserKindAssignmentId::new),
            tenant_id: ctx.tenant_id,
            user_id: cmd.user_id,
            kind: cmd.kind,
            is_primary: cmd.is_primary,
            assigned_at: Utc::now(),
        };
        self.kinds.upsert(&assignment).await?;

        self.audit
            .record(
                ctx,
                cmd.user_id,
                "kind_assigned",
                "user_kind",
                Some(assignment.id.as_uuid()),
                format!("Assigned kind {}", cmd.kind.as_str()),
                serde_json::json!({ "kind": cmd.kind.as_str(), "is_primary": cmd.is_primary }),
                UsersEvent::UserKindAssigned {
                    tenant_id: ctx.tenant_id,
                    user_id: cmd.user_id,
                    assignment_id: assignment.id,
                    kind: cmd.kind,
                },
            )
            .await?;

        Ok(assignment)
    }

    pub async fn remove(
        &self,
        ctx: &ActingContext,
        user_id: UserId,
        kind: UserKind,
    ) -> Result<(), UsersError> {
        authorize(self.authz.as_ref(), ctx, permissions::KIND_MANAGE).await?;

        self.kinds
            .get(user_id, kind)
            .await?
            .ok_or(UsersError::NotFound("user_kind"))?;
        self.kinds.remove(user_id, kind).await?;

        self.audit
            .record(
                ctx,
                user_id,
                "kind_removed",
                "user_kind",
                None,
                format!("Removed kind {}", kind.as_str()),
                serde_json::json!({ "kind": kind.as_str() }),
                UsersEvent::UserKindRemoved {
                    tenant_id: ctx.tenant_id,
                    user_id,
                    kind,
                },
            )
            .await?;

        Ok(())
    }

    pub async fn list(&self, user_id: UserId) -> Result<Vec<UserKindAssignment>, UsersError> {
        self.kinds.list(user_id).await
    }
}
