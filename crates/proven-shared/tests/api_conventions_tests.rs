//! REST API convention unit tests (ADR-0013).

use proven_shared::{
    parse_multi_value, require_known_filters, AppError, CursorPageRequest, DataEnvelope,
    ErrorResponse, ListEnvelope, ListQuery, PaginationMeta, SortDirection, API_V1_PREFIX,
    DEFAULT_PAGE_LIMIT, MAX_PAGE_LIMIT,
};

#[test]
fn cursor_page_limits() {
    assert!(CursorPageRequest::new(Some(0), None).is_err());
    assert!(CursorPageRequest::new(Some(MAX_PAGE_LIMIT + 1), None).is_err());
    let ok = CursorPageRequest::new(None, None).unwrap();
    assert_eq!(ok.limit, DEFAULT_PAGE_LIMIT);
}

#[test]
fn list_query_sort_whitelist() {
    let q = ListQuery::parse(
        Some(10),
        None,
        Some("created_at:desc,name:asc"),
        Some("  site  ".into()),
        &["created_at", "name"],
    )
    .unwrap();
    assert_eq!(q.page.limit, 10);
    assert_eq!(q.sort.len(), 2);
    assert_eq!(q.sort[0].direction, SortDirection::Desc);
    assert_eq!(q.q.as_deref(), Some("site"));

    let err = ListQuery::parse(None, None, Some("hack:asc"), None, &["name"]).unwrap_err();
    assert_eq!(err.error_code(), "validation_failed");
    assert_eq!(err.status_code(), 422);
}

#[test]
fn strict_filters_reject_unknown() {
    let err = require_known_filters(&["status", "evil"], &["status", "q"]).unwrap_err();
    assert_eq!(err.error_code(), "validation_failed");
    assert!(parse_multi_value(Some("a, b ,c")).len() == 3);
}

#[test]
fn envelopes_serialize() {
    let single = DataEnvelope::new(serde_json::json!({"id": 1}));
    let v = serde_json::to_value(&single).unwrap();
    assert!(v.get("data").is_some());

    let list = ListEnvelope::new(
        vec!["a".to_string()],
        PaginationMeta::from_cursor(Some("c1".into())),
    );
    let v = serde_json::to_value(&list).unwrap();
    assert_eq!(v["pagination"]["has_more"], true);
    assert_eq!(v["pagination"]["next_cursor"], "c1");
}

#[test]
fn error_envelope_nested() {
    let err = AppError::Validation {
        message: "bad".into(),
        details: vec![proven_shared::FieldError::new("name", "required", "required")],
    };
    let body = ErrorResponse::from_app_error(&err, Some("corr-1".into()));
    let v = serde_json::to_value(&body).unwrap();
    assert_eq!(v["error"]["code"], "validation_failed");
    assert_eq!(v["error"]["correlation_id"], "corr-1");
    assert!(v["error"]["details"].as_array().unwrap().len() == 1);
    assert!(v.get("title").is_none()); // not flat ProblemDetails
}

#[test]
fn versioning_constants() {
    assert_eq!(API_V1_PREFIX, "/api/v1");
}
