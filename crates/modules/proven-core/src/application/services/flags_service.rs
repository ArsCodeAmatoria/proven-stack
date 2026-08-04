//! `FeatureFlagService` — resolve effective flag: override wins, then default (CORE_DOMAIN.md §19.2).

use std::sync::Arc;

use proven_shared::{FeatureFlagKey, TenantId, UserId};

use crate::application::ports::FlagsRepository;
use crate::domain::CoreError;

pub struct FlagsService {
    flags: Arc<dyn FlagsRepository>,
}

impl FlagsService {
    pub fn new(flags: Arc<dyn FlagsRepository>) -> Self {
        Self { flags }
    }

    pub async fn evaluate(
        &self,
        key: &FeatureFlagKey,
        tenant_id: Option<TenantId>,
        user_id: Option<UserId>,
    ) -> Result<bool, CoreError> {
        if let Some(uid) = user_id {
            if let Some(enabled) = self.flags.get_override(key, tenant_id, Some(uid)).await? {
                return Ok(enabled);
            }
        }
        if let Some(tid) = tenant_id {
            if let Some(enabled) = self.flags.get_override(key, Some(tid), None).await? {
                return Ok(enabled);
            }
        }
        Ok(self
            .flags
            .get_flag(key)
            .await?
            .map(|flag| flag.default_enabled)
            .unwrap_or(false))
    }
}
