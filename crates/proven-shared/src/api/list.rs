//! List query helpers — sort, filter whitelist, search `q`.

use serde::{Deserialize, Serialize};

use crate::error::{AppError, FieldError};

use super::paging::{CursorPageRequest, DEFAULT_PAGE_LIMIT};

/// Sort direction for `?sort=field:asc|desc`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SortField {
    pub field: String,
    pub direction: SortDirection,
}

impl SortField {
    pub fn new(field: impl Into<String>, direction: SortDirection) -> Self {
        Self {
            field: field.into(),
            direction,
        }
    }
}

/// Parsed list query conventions used by collection endpoints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListQuery {
    pub page: CursorPageRequest,
    pub sort: Vec<SortField>,
    pub q: Option<String>,
}

impl ListQuery {
    /// Parse common list params. `allowed_sort` is a whitelist (empty = no sort allowed).
    pub fn parse(
        limit: Option<u32>,
        cursor: Option<String>,
        sort: Option<&str>,
        q: Option<String>,
        allowed_sort: &[&str],
    ) -> Result<Self, AppError> {
        let page = CursorPageRequest::new(limit, cursor)?;
        let sort = parse_sort(sort, allowed_sort)?;
        let q = q
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        if let Some(ref query) = q {
            if query.chars().count() > 200 {
                return Err(AppError::Validation {
                    message: "search query is too long (max 200 characters)".into(),
                    details: vec![FieldError::new("q", "too_long", "max 200 characters")],
                });
            }
        }
        Ok(Self { page, sort, q })
    }

    pub fn limit_or_default(limit: Option<u32>) -> u32 {
        limit
            .unwrap_or(DEFAULT_PAGE_LIMIT)
            .min(super::paging::MAX_PAGE_LIMIT)
    }
}

/// Parse `sort=created_at:desc,name:asc`. Unknown fields → validation error (strict).
pub fn parse_sort(raw: Option<&str>, allowed: &[&str]) -> Result<Vec<SortField>, AppError> {
    let Some(raw) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(Vec::new());
    };

    let mut out = Vec::new();
    let mut details = Vec::new();

    for part in raw.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (field, direction) = match part.split_once(':') {
            Some((f, d)) => (f.trim(), d.trim()),
            None => (part, "asc"),
        };
        if field.is_empty() {
            details.push(FieldError::new("sort", "invalid", "empty sort field"));
            continue;
        }
        if !allowed.is_empty() && !allowed.iter().any(|a| *a == field) {
            details.push(FieldError::new(
                "sort",
                "unknown_field",
                format!("sort field '{field}' is not allowed"),
            ));
            continue;
        }
        let direction = match direction.to_ascii_lowercase().as_str() {
            "asc" => SortDirection::Asc,
            "desc" => SortDirection::Desc,
            other => {
                details.push(FieldError::new(
                    "sort",
                    "invalid_direction",
                    format!("direction '{other}' must be asc or desc"),
                ));
                continue;
            }
        };
        out.push(SortField::new(field, direction));
    }

    if !details.is_empty() {
        return Err(AppError::Validation {
            message: "invalid sort parameter".into(),
            details,
        });
    }
    Ok(out)
}

/// Strict filter mode: reject unknown query filter keys (REST_API.md §8).
pub fn require_known_filters(
    provided: &[impl AsRef<str>],
    allowed: &[&str],
) -> Result<(), AppError> {
    let mut details = Vec::new();
    for key in provided {
        let key = key.as_ref();
        if !allowed.iter().any(|a| *a == key) {
            details.push(FieldError::new(
                key,
                "unknown_filter",
                format!("filter '{key}' is not supported"),
            ));
        }
    }
    if details.is_empty() {
        Ok(())
    } else {
        Err(AppError::Validation {
            message: "unknown filter parameter(s)".into(),
            details,
        })
    }
}

/// Split multi-value filters (`status=active,on_hold` or CSV).
pub fn parse_multi_value(raw: Option<&str>) -> Vec<String> {
    raw.map(|s| {
        s.split(',')
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .map(str::to_string)
            .collect()
    })
    .unwrap_or_default()
}
