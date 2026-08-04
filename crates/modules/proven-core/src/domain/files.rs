//! File management domain types (ADR-0010, R2_STORAGE_ARCHITECTURE.md).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use proven_shared::{FileObjectId, TenantId, UserId};

/// Logical content class — maps to R2 prefix (`images`, `pdfs`, …).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileObjectClass {
    Photo,
    Pdf,
    Video,
    Certificate,
    Drawing,
    Attachment,
}

impl FileObjectClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Photo => "photo",
            Self::Pdf => "pdf",
            Self::Video => "video",
            Self::Certificate => "certificate",
            Self::Drawing => "drawing",
            Self::Attachment => "attachment",
        }
    }

    /// R2 key prefix segment (plural folder name).
    pub fn storage_prefix(self) -> &'static str {
        match self {
            Self::Photo => "images",
            Self::Pdf => "pdfs",
            Self::Video => "videos",
            Self::Certificate => "certs",
            Self::Drawing => "drawings",
            Self::Attachment => "attachments",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "photo" | "image" | "images" => Some(Self::Photo),
            "pdf" | "pdfs" => Some(Self::Pdf),
            "video" | "videos" => Some(Self::Video),
            "certificate" | "cert" | "certs" => Some(Self::Certificate),
            "drawing" | "drawings" => Some(Self::Drawing),
            "attachment" | "attachments" => Some(Self::Attachment),
            _ => None,
        }
    }

    /// MIME allowlist for the class (server-side gate before intent).
    pub fn allows_content_type(self, content_type: &str) -> bool {
        let ct = content_type.to_ascii_lowercase();
        let base = ct.split(';').next().unwrap_or(&ct).trim();
        match self {
            Self::Photo => {
                matches!(base, "image/jpeg" | "image/png" | "image/webp" | "image/heic")
            }
            Self::Pdf | Self::Certificate => base == "application/pdf",
            Self::Video => {
                matches!(base, "video/mp4" | "video/quicktime" | "video/webm")
            }
            Self::Drawing => {
                matches!(
                    base,
                    "application/pdf"
                        | "image/png"
                        | "image/jpeg"
                        | "image/vnd.dwg"
                        | "application/acad"
                        | "application/octet-stream"
                )
            }
            Self::Attachment => !base.is_empty(),
        }
    }
}

impl Default for FileObjectClass {
    fn default() -> Self {
        Self::Attachment
    }
}

/// Virus / malware scan outcome recorded on the FileObject.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VirusScanStatus {
    #[default]
    NotScanned,
    Pending,
    Clean,
    Infected,
    Error,
}

impl VirusScanStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotScanned => "not_scanned",
            Self::Pending => "pending",
            Self::Clean => "clean",
            Self::Infected => "infected",
            Self::Error => "error",
        }
    }
}

/// Share / download link kind. Public shares still resolve through the API — never public R2 ACLs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileLinkKind {
    /// Short-lived authenticated download (presigned GET).
    Private,
    /// Longer-lived share token resolved via Core HTTP (bucket remains private).
    PublicShare,
}

impl FileLinkKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Private => "private",
            Self::PublicShare => "public_share",
        }
    }
}

/// Presigned URL descriptor returned to clients (R2 or placeholder).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresignedUrl {
    pub url: String,
    pub method: String,
    pub expires_at: DateTime<Utc>,
    pub headers: serde_json::Value,
    /// When true, URL is a local/dev placeholder — Cloudflare signing is not wired.
    pub placeholder: bool,
}

/// Result of creating an upload intent (metadata + PUT target).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadIntent {
    pub file: crate::domain::FileObject,
    pub upload: PresignedUrl,
}

/// Result of authorizing a download.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadLink {
    pub file_id: FileObjectId,
    pub download: PresignedUrl,
    pub link_kind: FileLinkKind,
}

/// Share token row for public (API-mediated) links.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileShareLink {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub file_id: FileObjectId,
    pub token: String,
    pub kind: FileLinkKind,
    pub expires_at: DateTime<Utc>,
    pub created_by: Option<UserId>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub max_downloads: Option<i32>,
    pub download_count: i32,
}

impl FileShareLink {
    pub fn is_usable(&self, now: DateTime<Utc>) -> bool {
        self.revoked_at.is_none()
            && self.expires_at > now
            && self
                .max_downloads
                .map(|max| self.download_count < max)
                .unwrap_or(true)
    }
}

/// Input to the virus-scan hook after upload complete.
#[derive(Debug, Clone)]
pub struct VirusScanRequest {
    pub tenant_id: TenantId,
    pub file_id: FileObjectId,
    pub storage_key: String,
    pub content_type: Option<String>,
    pub checksum_sha256: String,
    pub byte_size: i64,
    pub object_class: FileObjectClass,
}

/// Hook outcome — `Pending` means a worker will call back later.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VirusScanOutcome {
    Clean { detail: Option<String> },
    Infected { detail: Option<String> },
    Pending { detail: Option<String> },
    Error { detail: String },
}
