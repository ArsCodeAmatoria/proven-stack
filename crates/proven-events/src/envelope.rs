//! Shared event envelope (EVENT_CATALOG.md §3, ADR-0011).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use proven_shared::{CausationId, CorrelationId, TenantId};

use crate::naming::{event_subject, SubjectParts};

/// Who caused the event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ActorRef {
    User { user_id: Uuid },
    Principal { principal_id: Uuid },
    System,
}

/// Primary resource the event is about.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceRef {
    pub resource_type: String,
    pub resource_id: Uuid,
}

/// Transport envelope for all Proven NATS events.
///
/// `event_version` is a **payload schema** semver string (`1.0.0`). The **subject** major
/// (`proven.<module>.v1.…`) is the transport/name major and is derived separately.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventEnvelope {
    pub event_id: Uuid,
    /// PascalCase past-tense name (e.g. `ProjectCreated`).
    pub event_name: String,
    /// Payload schema version — additive within major; bump major on breaking changes.
    pub event_version: String,
    pub occurred_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published_at: Option<DateTime<Utc>>,
    pub tenant_id: TenantId,
    pub actor: ActorRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<CorrelationId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causation_id: Option<CausationId>,
    pub resource: ResourceRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<Uuid>,
    /// Module key used in the subject (`core`, `companies`, `projects`, …).
    pub module: String,
    /// Transport subject major version (usually `1`).
    pub subject_major: u32,
    pub payload: serde_json::Value,
}

impl EventEnvelope {
    pub fn new(
        module: impl Into<String>,
        event_name: impl Into<String>,
        tenant_id: TenantId,
        actor: ActorRef,
        resource: ResourceRef,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_name: event_name.into(),
            event_version: "1.0.0".to_string(),
            occurred_at: Utc::now(),
            published_at: None,
            tenant_id,
            actor,
            correlation_id: None,
            causation_id: None,
            resource,
            project_id: None,
            module: module.into(),
            subject_major: 1,
            payload,
        }
    }

    pub fn with_event_version(mut self, version: impl Into<String>) -> Self {
        self.event_version = version.into();
        self
    }

    pub fn with_subject_major(mut self, major: u32) -> Self {
        self.subject_major = major;
        self
    }

    pub fn with_correlation(mut self, id: CorrelationId) -> Self {
        self.correlation_id = Some(id);
        self
    }

    pub fn with_causation(mut self, id: CausationId) -> Self {
        self.causation_id = Some(id);
        self
    }

    pub fn with_project(mut self, project_id: Uuid) -> Self {
        self.project_id = Some(project_id);
        self
    }

    pub fn mark_published(mut self) -> Self {
        self.published_at = Some(Utc::now());
        self
    }

    /// NATS subject for this envelope.
    pub fn subject(&self) -> String {
        event_subject(&self.module, self.subject_major, &self.event_name)
    }

    pub fn subject_parts(&self) -> SubjectParts {
        SubjectParts {
            module: self.module.clone(),
            major: self.subject_major,
            event_name: self.event_name.clone(),
        }
    }

    /// Major component of `event_version` (payload schema).
    pub fn payload_major(&self) -> u32 {
        self.event_version
            .split('.')
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, crate::error::EventError> {
        Ok(serde_json::to_vec(self)?)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, crate::error::EventError> {
        Ok(serde_json::from_slice(bytes)?)
    }
}
