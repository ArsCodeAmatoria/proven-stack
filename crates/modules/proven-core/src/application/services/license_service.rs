//! `LicenseEnforcementService` — module gating and entitlement lookups (CORE_DOMAIN.md §19.3).

use std::sync::Arc;

use proven_shared::{ModuleKey, TenantId};

use crate::application::ports::LicenseRepository;
use crate::domain::{CoreError, License};

pub struct LicenseService {
    licenses: Arc<dyn LicenseRepository>,
}

impl LicenseService {
    pub fn new(licenses: Arc<dyn LicenseRepository>) -> Self {
        Self { licenses }
    }

    pub async fn get_current(&self, tenant_id: TenantId) -> Result<License, CoreError> {
        self.licenses
            .get_current(tenant_id)
            .await?
            .ok_or(CoreError::NotFound("license"))
    }

    /// Fail closed: unlicensed, expired, or unlisted modules are disabled.
    pub async fn is_module_enabled(
        &self,
        tenant_id: TenantId,
        module: &ModuleKey,
    ) -> Result<bool, CoreError> {
        let license = match self.licenses.get_current(tenant_id).await? {
            Some(l) => l,
            None => return Ok(false),
        };
        if !license.is_usable() {
            return Ok(false);
        }
        let entitlements = self.licenses.get_entitlements(license.id).await?;
        Ok(entitlements
            .into_iter()
            .any(|e| &e.module_key == module && e.enabled))
    }
}
