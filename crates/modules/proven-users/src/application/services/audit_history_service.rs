//! `AuditHistoryService` — read access over the append-only profile audit log (ADR-0006 §8). Not
//! a substitute for Core's `AuditApi` — see `domain::ownership`.

use std::sync::Arc;

use proven_core::AuthzApi;
use proven_shared::UserId;

use crate::application::ports::ProfileAuditRepository;
use crate::application::services::authz::{authorize_self_or_permission, ActingContext};
use crate::domain::permissions;
use crate::domain::{ProfileAuditEntry, UsersError};

pub struct AuditHistoryService {
    audit: Arc<dyn ProfileAuditRepository>,
    authz: Arc<dyn AuthzApi>,
}

impl AuditHistoryService {
    pub fn new(audit: Arc<dyn ProfileAuditRepository>, authz: Arc<dyn AuthzApi>) -> Self {
        Self { audit, authz }
    }

    pub async fn list(
        &self,
        ctx: &ActingContext,
        user_id: UserId,
    ) -> Result<Vec<ProfileAuditEntry>, UsersError> {
        authorize_self_or_permission(self.authz.as_ref(), ctx, permissions::AUDIT_READ, user_id)
            .await?;
        self.audit.list(user_id).await
    }
}
