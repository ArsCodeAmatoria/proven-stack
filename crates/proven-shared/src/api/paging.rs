//! Cursor and offset paging primitives.

use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// Default list page size (REST_API.md §7).
pub const DEFAULT_PAGE_LIMIT: u32 = 25;
/// Maximum list page size for interactive APIs.
pub const MAX_PAGE_LIMIT: u32 = 100;

/// Cursor pagination request (`?limit=&cursor=`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CursorPageRequest {
    pub limit: u32,
    pub cursor: Option<String>,
}

impl Default for CursorPageRequest {
    fn default() -> Self {
        Self {
            limit: DEFAULT_PAGE_LIMIT,
            cursor: None,
        }
    }
}

impl CursorPageRequest {
    pub fn new(limit: Option<u32>, cursor: Option<String>) -> Result<Self, AppError> {
        let limit = limit.unwrap_or(DEFAULT_PAGE_LIMIT);
        if limit == 0 {
            return Err(AppError::BadRequest(
                "limit must be greater than zero".into(),
            ));
        }
        if limit > MAX_PAGE_LIMIT {
            return Err(AppError::BadRequest(format!(
                "limit must be at most {MAX_PAGE_LIMIT}"
            )));
        }
        Ok(Self { limit, cursor })
    }
}

/// Cursor page result (internal / repository shape).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CursorPage<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}

impl<T> CursorPage<T> {
    pub fn new(items: Vec<T>, next_cursor: Option<String>) -> Self {
        Self {
            items,
            next_cursor,
        }
    }

    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            next_cursor: None,
        }
    }
}

/// Offset pagination request — for admin/export/search SoR queries, not hot field lists.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OffsetPageRequest {
    pub limit: u32,
    pub offset: u32,
}

impl Default for OffsetPageRequest {
    fn default() -> Self {
        Self {
            limit: DEFAULT_PAGE_LIMIT,
            offset: 0,
        }
    }
}

impl OffsetPageRequest {
    pub fn new(limit: Option<u32>, offset: Option<u32>) -> Result<Self, AppError> {
        let limit = limit.unwrap_or(DEFAULT_PAGE_LIMIT);
        let offset = offset.unwrap_or(0);
        if limit == 0 {
            return Err(AppError::BadRequest(
                "limit must be greater than zero".into(),
            ));
        }
        if limit > MAX_PAGE_LIMIT {
            return Err(AppError::BadRequest(format!(
                "limit must be at most {MAX_PAGE_LIMIT}"
            )));
        }
        Ok(Self { limit, offset })
    }
}

/// Offset page result (repositories / audit search).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OffsetPage<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub limit: u32,
    pub offset: u32,
}
