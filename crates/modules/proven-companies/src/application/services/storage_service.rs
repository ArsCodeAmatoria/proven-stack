//! `StorageService` — company-wide file upload/storage policy (ADR-0005 §3).

use std::sync::Arc;

use chrono::Utc;
use proven_core::AuthzApi;
use proven_shared::CompanyId;

use crate::application::ports::{EventPublisher, StorageConfigurationRepository};
use crate::application::services::authz::{authorize, ActingContext};
use crate::domain::validation::{validate_max_upload_bytes, validate_non_empty};
use crate::domain::{permissions, CompaniesError, StorageConfiguration};
use crate::events::{ActorRef, CompaniesEvent, EventEnvelope, ResourceRef};

pub struct UpsertStorageConfigurationCommand {
    pub company_id: CompanyId,
    pub object_prefix: Option<String>,
    pub max_upload_bytes: Option<i64>,
    pub allowed_content_types: Option<Vec<String>>,
    pub retention_class_default: Option<String>,
    pub quarantine_enabled: Option<bool>,
}

pub struct StorageService {
    config: Arc<dyn StorageConfigurationRepository>,
    outbox: Arc<dyn EventPublisher>,
    authz: Arc<dyn AuthzApi>,
}

impl StorageService {
    pub fn new(
        config: Arc<dyn StorageConfigurationRepository>,
        outbox: Arc<dyn EventPublisher>,
        authz: Arc<dyn AuthzApi>,
    ) -> Self {
        Self {
            config,
            outbox,
            authz,
        }
    }

    pub async fn get(&self, company_id: CompanyId) -> Result<StorageConfiguration, CompaniesError> {
        self.config
            .get(company_id)
            .await?
            .ok_or(CompaniesError::NotFound("storage_configuration"))
    }

    pub async fn upsert(
        &self,
        ctx: &ActingContext,
        cmd: UpsertStorageConfigurationCommand,
    ) -> Result<StorageConfiguration, CompaniesError> {
        authorize(
            self.authz.as_ref(),
            ctx,
            permissions::STORAGE_MANAGE,
            cmd.company_id,
        )
        .await?;
        if let Some(max_upload_bytes) = cmd.max_upload_bytes {
            validate_max_upload_bytes(max_upload_bytes)?;
        }
        if let Some(object_prefix) = &cmd.object_prefix {
            validate_non_empty("object_prefix", object_prefix)?;
        }
        if let Some(retention_class_default) = &cmd.retention_class_default {
            validate_non_empty("retention_class_default", retention_class_default)?;
        }

        let now = Utc::now();
        let mut config = self
            .config
            .get(cmd.company_id)
            .await?
            .unwrap_or_else(|| StorageConfiguration::defaults(cmd.company_id, ctx.tenant_id, now));

        if let Some(object_prefix) = cmd.object_prefix {
            config.object_prefix = object_prefix;
        }
        if let Some(max_upload_bytes) = cmd.max_upload_bytes {
            config.max_upload_bytes = max_upload_bytes;
        }
        if let Some(allowed_content_types) = cmd.allowed_content_types {
            config.allowed_content_types = allowed_content_types;
        }
        if let Some(retention_class_default) = cmd.retention_class_default {
            config.retention_class_default = retention_class_default;
        }
        if let Some(quarantine_enabled) = cmd.quarantine_enabled {
            config.quarantine_enabled = quarantine_enabled;
        }
        config.updated_at = now;
        self.config.upsert(&config).await?;

        self.outbox
            .publish(EventEnvelope::new(
                ctx.tenant_id,
                ActorRef::Principal {
                    principal_id: ctx.principal,
                },
                ResourceRef {
                    resource_type: "storage_configuration".to_string(),
                    resource_id: cmd.company_id.as_uuid(),
                },
                None,
                None,
                CompaniesEvent::StorageConfigurationUpdated {
                    tenant_id: ctx.tenant_id,
                    company_id: cmd.company_id,
                },
            ))
            .await?;

        Ok(config)
    }
}
