//! Project Place aggregate and participant entities (ADR-0009 skeleton).

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use proven_shared::{CompanyId, ProjectId, TenantId};

use super::{ParticipantId, ParticipantStatus, ParticipationRole, ProjectStatus};

/// Primary site locality for a project (embedded on the Project row in the skeleton).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectLocation {
    pub line1: String,
    pub line2: Option<String>,
    pub city: String,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country_code: String,
    pub timezone: Option<String>,
}

/// Construction undertaking / Place — SoR in this module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub tenant_id: TenantId,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub status: ProjectStatus,
    pub location: Option<ProjectLocation>,
    pub prime_contractor_company_id: CompanyId,
    pub client_company_id: Option<CompanyId>,
    pub planned_start: Option<NaiveDate>,
    pub planned_end: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

/// Company engagement on a project (Prime / Client / …).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectParticipant {
    pub id: ParticipantId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub company_id: CompanyId,
    pub role: ParticipationRole,
    pub status: ParticipantStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}
