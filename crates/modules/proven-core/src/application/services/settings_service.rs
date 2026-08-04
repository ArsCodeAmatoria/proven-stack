//! `SettingsResolver` — get/upsert scoped settings (CORE_DOMAIN.md §19.1).

use std::sync::Arc;

use chrono::Utc;
use proven_shared::{SettingKey, TenantId};
use uuid::Uuid;

use crate::application::ports::{EventPublisher, SettingsRepository};
use crate::domain::{CoreError, SettingEntry, SettingScopeType};
use crate::events::{ActorRef, CoreEvent, EventEnvelope, ResourceRef};

pub struct UpsertSettingCommand {
    pub tenant_id: TenantId,
    pub scope_type: SettingScopeType,
    pub scope_id: Option<Uuid>,
    pub key: SettingKey,
    pub value: serde_json::Value,
}

pub struct SettingsService {
    settings: Arc<dyn SettingsRepository>,
    outbox: Arc<dyn EventPublisher>,
}

impl SettingsService {
    pub fn new(settings: Arc<dyn SettingsRepository>, outbox: Arc<dyn EventPublisher>) -> Self {
        Self { settings, outbox }
    }

    pub async fn get(
        &self,
        tenant_id: TenantId,
        scope_type: SettingScopeType,
        scope_id: Option<Uuid>,
        key: &SettingKey,
    ) -> Result<Option<SettingEntry>, CoreError> {
        self.settings
            .get(tenant_id, scope_type, scope_id, key)
            .await
    }

    pub async fn upsert(&self, cmd: UpsertSettingCommand) -> Result<SettingEntry, CoreError> {
        let entry = SettingEntry {
            tenant_id: cmd.tenant_id,
            scope_type: cmd.scope_type,
            scope_id: cmd.scope_id,
            key: cmd.key.clone(),
            value: cmd.value,
            updated_at: Utc::now(),
        };
        self.settings.upsert(&entry).await?;

        self.outbox
            .publish(EventEnvelope::new(
                cmd.tenant_id,
                ActorRef::System,
                ResourceRef {
                    resource_type: "setting".to_string(),
                    resource_id: cmd.scope_id.unwrap_or(cmd.tenant_id.as_uuid()),
                },
                None,
                None,
                CoreEvent::SettingsChanged {
                    tenant_id: cmd.tenant_id,
                    key: cmd.key,
                },
            ))
            .await?;

        Ok(entry)
    }
}
