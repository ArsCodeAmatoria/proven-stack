//! Companies enumerations / states — mirrors the CHECK constraints in
//! `db/migrations/companies/20260803210000_companies_schema.sql` (ADR-0005).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileStatus {
    Active,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BusinessUnitStatus {
    Active,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AddressKind {
    HeadOffice,
    Billing,
    Site,
    Mailing,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContactKind {
    Primary,
    Billing,
    Safety,
    Hr,
    Operations,
    Other,
}

/// Unit system a company reports quantities in (default templates, safety forms, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeasurementSystem {
    Metric,
    Imperial,
}

/// How often a company's rolled-up notification digest is sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DigestCadence {
    Realtime,
    Hourly,
    Daily,
    Weekly,
    Off,
}

/// Kind of document template a company can point a default at (the template artifact itself
/// is owned by Documents/Training/Projects/Safety — this module only stores the pointer, see
/// `domain::ownership`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateKind {
    Project,
    Flha,
    Inspection,
    Toolbox,
    Document,
    Training,
    Notification,
    Other,
}
