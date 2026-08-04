//! Request DTOs for Projects HTTP handlers.

use chrono::NaiveDate;
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::ProjectLocation;

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub location: Option<ProjectLocation>,
    pub prime_contractor_company_id: Uuid,
    pub client_company_id: Option<Uuid>,
    pub planned_start: Option<NaiveDate>,
    pub planned_end: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub location: Option<ProjectLocation>,
    /// Omit to leave unchanged; JSON `null` clears the client.
    #[serde(default, deserialize_with = "crate::api::dto::deserialize_optional_uuid")]
    pub client_company_id: Option<Option<Uuid>>,
    #[serde(default)]
    pub planned_start: Option<Option<NaiveDate>>,
    #[serde(default)]
    pub planned_end: Option<Option<NaiveDate>>,
}

#[derive(Debug, Deserialize)]
pub struct AssignMembershipRequest {
    pub user_id: Uuid,
    pub membership_role: String,
}

#[derive(Debug, Deserialize)]
pub struct ListProjectsQuery {
    #[serde(default)]
    pub include_archived: bool,
}

fn deserialize_optional_uuid<'de, D>(deserializer: D) -> Result<Option<Option<Uuid>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Some(Option::<Uuid>::deserialize(deserializer)?))
}
