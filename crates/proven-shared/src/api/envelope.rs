//! Success response envelopes (`{ data }` / `{ data, pagination }`).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::paging::CursorPage;

/// Single-resource or action success body.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataEnvelope<T> {
    pub data: T,
}

impl<T> DataEnvelope<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

/// Cursor pagination metadata (REST_API.md §7).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PaginationMeta {
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

impl PaginationMeta {
    pub fn none() -> Self {
        Self {
            next_cursor: None,
            has_more: false,
        }
    }

    pub fn from_cursor(next_cursor: Option<String>) -> Self {
        let has_more = next_cursor.is_some();
        Self {
            next_cursor,
            has_more,
        }
    }
}

/// List success body.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListEnvelope<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

impl<T> ListEnvelope<T> {
    pub fn new(data: Vec<T>, pagination: PaginationMeta) -> Self {
        Self { data, pagination }
    }

    pub fn from_cursor_page(page: CursorPage<T>) -> Self {
        Self {
            data: page.items,
            pagination: PaginationMeta::from_cursor(page.next_cursor),
        }
    }

    pub fn empty() -> Self {
        Self {
            data: Vec::new(),
            pagination: PaginationMeta::none(),
        }
    }
}
