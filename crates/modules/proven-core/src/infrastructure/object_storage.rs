//! Object storage adapters for Core FileApi (ADR-0010).
//!
//! **Pending integration:** a production Cloudflare R2 signer (AWS SigV4 against the R2 S3
//! endpoint) is not wired yet. [`PlaceholderObjectStorage`] issues non-functional but
//! structurally correct URL descriptors so the upload/download API shape can be tested.
//! See `docs/development/FILE_MANAGEMENT.md`.

use async_trait::async_trait;
use chrono::{Duration, Utc};

use crate::application::ports::ObjectStoragePort;
use crate::domain::{CoreError, PresignedUrl};

/// Dev/test storage adapter — no network calls; marks URLs as `placeholder: true`.
#[derive(Debug, Default, Clone)]
pub struct PlaceholderObjectStorage {
    pub public_base_url: String,
}

impl PlaceholderObjectStorage {
    pub fn new() -> Self {
        Self {
            public_base_url: "https://r2.placeholder.local".to_string(),
        }
    }

    pub fn with_base_url(base: impl Into<String>) -> Self {
        Self {
            public_base_url: base.into(),
        }
    }
}

#[async_trait]
impl ObjectStoragePort for PlaceholderObjectStorage {
    async fn presign_put(
        &self,
        key: &str,
        content_type: &str,
        ttl_secs: u64,
    ) -> Result<PresignedUrl, CoreError> {
        let expires_at = Utc::now() + Duration::seconds(ttl_secs as i64);
        Ok(PresignedUrl {
            url: format!("{}/upload/{}", self.public_base_url.trim_end_matches('/'), key),
            method: "PUT".to_string(),
            expires_at,
            headers: serde_json::json!({
                "Content-Type": content_type,
                "x-proven-storage": "placeholder",
            }),
            placeholder: true,
        })
    }

    async fn presign_get(
        &self,
        key: &str,
        ttl_secs: u64,
        filename: Option<&str>,
    ) -> Result<PresignedUrl, CoreError> {
        let expires_at = Utc::now() + Duration::seconds(ttl_secs as i64);
        let mut headers = serde_json::Map::new();
        headers.insert(
            "x-proven-storage".to_string(),
            serde_json::Value::String("placeholder".into()),
        );
        if let Some(name) = filename {
            headers.insert(
                "Content-Disposition".to_string(),
                serde_json::Value::String(format!("attachment; filename=\"{name}\"")),
            );
        }
        Ok(PresignedUrl {
            url: format!("{}/download/{}", self.public_base_url.trim_end_matches('/'), key),
            method: "GET".to_string(),
            expires_at,
            headers: serde_json::Value::Object(headers),
            placeholder: true,
        })
    }

    async fn delete_object(&self, _key: &str) -> Result<(), CoreError> {
        // No-op for placeholder — real R2 adapter will delete the object.
        Ok(())
    }
}

/// Configuration for a future Cloudflare R2 (S3-compatible) adapter.
///
/// When `configured()` is false, callers should keep using [`PlaceholderObjectStorage`].
/// When true, production must wire an SigV4 signer — **not implemented in this crate yet**.
#[derive(Debug, Clone, Default)]
pub struct R2StorageConfig {
    pub account_id: Option<String>,
    pub bucket: Option<String>,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub endpoint: Option<String>,
    pub public_base_url: Option<String>,
}

impl R2StorageConfig {
    pub fn configured(&self) -> bool {
        self.account_id.as_ref().is_some_and(|s| !s.is_empty())
            && self.bucket.as_ref().is_some_and(|s| !s.is_empty())
            && self.access_key_id.as_ref().is_some_and(|s| !s.is_empty())
            && self
                .secret_access_key
                .as_ref()
                .is_some_and(|s| !s.is_empty())
    }

    /// Build the S3-compatible API endpoint for this account when not overridden.
    pub fn resolved_endpoint(&self) -> Option<String> {
        if let Some(endpoint) = &self.endpoint {
            return Some(endpoint.clone());
        }
        self.account_id
            .as_ref()
            .map(|id| format!("https://{id}.r2.cloudflarestorage.com"))
    }
}

/// Stub that refuses to sign until the Cloudflare SDK / SigV4 signer is integrated.
/// Prefer [`PlaceholderObjectStorage`] in development; use this only to fail closed when
/// operators set `R2_*` but the signer is still pending.
#[derive(Debug, Clone)]
pub struct PendingR2ObjectStorage {
    pub config: R2StorageConfig,
}

#[async_trait]
impl ObjectStoragePort for PendingR2ObjectStorage {
    async fn presign_put(
        &self,
        _key: &str,
        _content_type: &str,
        _ttl_secs: u64,
    ) -> Result<PresignedUrl, CoreError> {
        Err(CoreError::Internal(
            "Cloudflare R2 signer is not wired yet — see docs/development/FILE_MANAGEMENT.md"
                .into(),
        ))
    }

    async fn presign_get(
        &self,
        _key: &str,
        _ttl_secs: u64,
        _filename: Option<&str>,
    ) -> Result<PresignedUrl, CoreError> {
        Err(CoreError::Internal(
            "Cloudflare R2 signer is not wired yet — see docs/development/FILE_MANAGEMENT.md"
                .into(),
        ))
    }

    async fn delete_object(&self, _key: &str) -> Result<(), CoreError> {
        Err(CoreError::Internal(
            "Cloudflare R2 signer is not wired yet — see docs/development/FILE_MANAGEMENT.md"
                .into(),
        ))
    }
}
