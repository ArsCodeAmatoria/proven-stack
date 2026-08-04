//! Integration events published by Companies (ADR-0005 §6). Each variant's subject follows
//! `proven.companies.v1.<EventName>` (e.g. `proven.companies.v1.CompanyProfileEnsured`),
//! mirroring the `proven.core.v1.*` convention used by `proven-core`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use proven_shared::{CompanyId, CausationId, CorrelationId, FileObjectId, PrincipalId, TenantId};

use crate::domain::{AddressId, BusinessUnitId, ContactId, DefaultTemplateId, TemplateKind};

/// Who performed the action that produced this event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "actor_type", rename_all = "snake_case")]
pub enum ActorRef {
    Principal { principal_id: PrincipalId },
    System,
}

/// What the event is about.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRef {
    pub resource_type: String,
    pub resource_id: Uuid,
}

/// Domain events published by Companies (ADR-0005 §6).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum CompaniesEvent {
    CompanyProfileEnsured {
        tenant_id: TenantId,
        company_id: CompanyId,
    },
    CompanyProfileUpdated {
        tenant_id: TenantId,
        company_id: CompanyId,
    },
    CompanyProfileArchived {
        tenant_id: TenantId,
        company_id: CompanyId,
    },
    BusinessUnitCreated {
        tenant_id: TenantId,
        company_id: CompanyId,
        business_unit_id: BusinessUnitId,
    },
    BusinessUnitUpdated {
        tenant_id: TenantId,
        company_id: CompanyId,
        business_unit_id: BusinessUnitId,
    },
    BusinessUnitArchived {
        tenant_id: TenantId,
        company_id: CompanyId,
        business_unit_id: BusinessUnitId,
    },
    AddressAdded {
        tenant_id: TenantId,
        company_id: CompanyId,
        address_id: AddressId,
    },
    AddressUpdated {
        tenant_id: TenantId,
        company_id: CompanyId,
        address_id: AddressId,
    },
    AddressRemoved {
        tenant_id: TenantId,
        company_id: CompanyId,
        address_id: AddressId,
    },
    ContactAdded {
        tenant_id: TenantId,
        company_id: CompanyId,
        contact_id: ContactId,
    },
    ContactUpdated {
        tenant_id: TenantId,
        company_id: CompanyId,
        contact_id: ContactId,
    },
    ContactRemoved {
        tenant_id: TenantId,
        company_id: CompanyId,
        contact_id: ContactId,
    },
    BrandingUpdated {
        tenant_id: TenantId,
        company_id: CompanyId,
        logo_file_id: Option<FileObjectId>,
    },
    SafetySettingsUpdated {
        tenant_id: TenantId,
        company_id: CompanyId,
    },
    RegionalSettingsUpdated {
        tenant_id: TenantId,
        company_id: CompanyId,
    },
    DefaultTemplateUpserted {
        tenant_id: TenantId,
        company_id: CompanyId,
        template_id: DefaultTemplateId,
        kind: TemplateKind,
    },
    NotificationDefaultsUpdated {
        tenant_id: TenantId,
        company_id: CompanyId,
    },
    StorageConfigurationUpdated {
        tenant_id: TenantId,
        company_id: CompanyId,
    },
}

impl CompaniesEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::CompanyProfileEnsured { .. } => "CompanyProfileEnsured",
            Self::CompanyProfileUpdated { .. } => "CompanyProfileUpdated",
            Self::CompanyProfileArchived { .. } => "CompanyProfileArchived",
            Self::BusinessUnitCreated { .. } => "BusinessUnitCreated",
            Self::BusinessUnitUpdated { .. } => "BusinessUnitUpdated",
            Self::BusinessUnitArchived { .. } => "BusinessUnitArchived",
            Self::AddressAdded { .. } => "AddressAdded",
            Self::AddressUpdated { .. } => "AddressUpdated",
            Self::AddressRemoved { .. } => "AddressRemoved",
            Self::ContactAdded { .. } => "ContactAdded",
            Self::ContactUpdated { .. } => "ContactUpdated",
            Self::ContactRemoved { .. } => "ContactRemoved",
            Self::BrandingUpdated { .. } => "BrandingUpdated",
            Self::SafetySettingsUpdated { .. } => "SafetySettingsUpdated",
            Self::RegionalSettingsUpdated { .. } => "RegionalSettingsUpdated",
            Self::DefaultTemplateUpserted { .. } => "DefaultTemplateUpserted",
            Self::NotificationDefaultsUpdated { .. } => "NotificationDefaultsUpdated",
            Self::StorageConfigurationUpdated { .. } => "StorageConfigurationUpdated",
        }
    }

    /// NATS-style subject this event is published on, e.g.
    /// `proven.companies.v1.CompanyProfileEnsured`.
    pub fn subject(&self) -> String {
        format!("proven.companies.v1.{}", self.event_type())
    }
}

/// Standard Companies event envelope, structurally aligned with `proven_core::events::EventEnvelope`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: Uuid,
    pub event_type: String,
    pub event_version: u32,
    pub occurred_at: DateTime<Utc>,
    pub tenant_id: TenantId,
    pub actor: ActorRef,
    pub correlation_id: Option<CorrelationId>,
    pub causation_id: Option<CausationId>,
    pub resource: ResourceRef,
    pub payload: CompaniesEvent,
}

impl EventEnvelope {
    pub fn new(
        tenant_id: TenantId,
        actor: ActorRef,
        resource: ResourceRef,
        correlation_id: Option<CorrelationId>,
        causation_id: Option<CausationId>,
        payload: CompaniesEvent,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type: payload.event_type().to_string(),
            event_version: 1,
            occurred_at: Utc::now(),
            tenant_id,
            actor,
            correlation_id,
            causation_id,
            resource,
            payload,
        }
    }
}
