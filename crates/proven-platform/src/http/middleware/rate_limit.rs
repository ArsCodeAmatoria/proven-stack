//! In-process rate limiting middleware (REST_API.md §11).
//!
//! Sliding-window counter keyed by client identity (principal header or peer IP).
//! Redis-backed quotas can replace this later without changing response headers.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, Request};
use axum::http::{HeaderName, HeaderValue};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use proven_shared::AppError;

use crate::http::ApiError;

const RATE_LIMIT_HEADER: HeaderName = HeaderName::from_static("x-ratelimit-limit");
const RATE_REMAINING_HEADER: HeaderName = HeaderName::from_static("x-ratelimit-remaining");
const RATE_RESET_HEADER: HeaderName = HeaderName::from_static("x-ratelimit-reset");

#[derive(Clone)]
pub struct RateLimitState {
    inner: Arc<Mutex<RateLimitInner>>,
    pub limit_per_minute: u32,
    pub enabled: bool,
}

struct RateLimitInner {
    windows: HashMap<String, Window>,
}

struct Window {
    count: u32,
    reset_at: Instant,
}

impl RateLimitState {
    pub fn new(limit_per_minute: u32, enabled: bool) -> Self {
        Self {
            inner: Arc::new(Mutex::new(RateLimitInner {
                windows: HashMap::new(),
            })),
            limit_per_minute: limit_per_minute.max(1),
            enabled,
        }
    }

    fn check(&self, key: &str) -> Result<RateLimitSnapshot, AppError> {
        let now = Instant::now();
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| AppError::Internal("rate limit lock poisoned".into()))?;

        // Opportunistic prune of expired windows.
        guard.windows.retain(|_, w| w.reset_at > now);

        let window = guard.windows.entry(key.to_string()).or_insert_with(|| Window {
            count: 0,
            reset_at: now + Duration::from_secs(60),
        });

        if window.reset_at <= now {
            window.count = 0;
            window.reset_at = now + Duration::from_secs(60);
        }

        let reset_secs = window
            .reset_at
            .saturating_duration_since(now)
            .as_secs()
            .max(1);

        if window.count >= self.limit_per_minute {
            return Err(AppError::RateLimited {
                retry_after_secs: reset_secs,
                limit: self.limit_per_minute,
            });
        }

        window.count += 1;
        let remaining = self.limit_per_minute.saturating_sub(window.count);
        Ok(RateLimitSnapshot {
            limit: self.limit_per_minute,
            remaining,
            reset_secs,
        })
    }
}

struct RateLimitSnapshot {
    limit: u32,
    remaining: u32,
    reset_secs: u64,
}

fn client_key(request: &Request) -> String {
    if let Some(user) = request
        .headers()
        .get("x-proven-user-id")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
    {
        return format!("user:{user}");
    }
    if let Some(auth) = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .filter(|s| s.len() > 12)
    {
        // Coarse key from bearer prefix — not a secret log; truncated.
        return format!("auth:{}", &auth[7..std::cmp::min(auth.len(), 23)]);
    }
    if let Some(ConnectInfo(addr)) = request.extensions().get::<ConnectInfo<SocketAddr>>() {
        return format!("ip:{}", addr.ip());
    }
    "anon".into()
}

fn is_exempt(path: &str) -> bool {
    matches!(
        path,
        "/health"
            | "/healthz"
            | "/readyz"
            | "/metrics"
            | "/docs"
            | "/redoc"
            | "/api-docs/openapi.json"
            | "/api/v1/openapi.json"
            | "/api/v1/health"
    ) || path.starts_with("/docs/")
}

pub async fn rate_limit_layer(request: Request, next: Next) -> Response {
    let path = request.uri().path().to_string();
    if is_exempt(&path) {
        return next.run(request).await;
    }

    let Some(state) = request.extensions().get::<RateLimitState>().cloned() else {
        return next.run(request).await;
    };

    if !state.enabled {
        return next.run(request).await;
    }

    let key = client_key(&request);
    match state.check(&key) {
        Ok(snap) => {
            let mut response = next.run(request).await;
            insert_rate_headers(response.headers_mut(), &snap);
            response
        }
        Err(err) => {
            let snap = RateLimitSnapshot {
                limit: state.limit_per_minute,
                remaining: 0,
                reset_secs: match &err {
                    AppError::RateLimited {
                        retry_after_secs, ..
                    } => *retry_after_secs,
                    _ => 60,
                },
            };
            let mut response = ApiError::from(err).into_response();
            insert_rate_headers(response.headers_mut(), &snap);
            response
        }
    }
}

fn insert_rate_headers(headers: &mut axum::http::HeaderMap, snap: &RateLimitSnapshot) {
    if let Ok(v) = HeaderValue::from_str(&snap.limit.to_string()) {
        headers.insert(RATE_LIMIT_HEADER, v);
    }
    if let Ok(v) = HeaderValue::from_str(&snap.remaining.to_string()) {
        headers.insert(RATE_REMAINING_HEADER, v);
    }
    if let Ok(v) = HeaderValue::from_str(&snap.reset_secs.to_string()) {
        headers.insert(RATE_RESET_HEADER, v);
    }
}
