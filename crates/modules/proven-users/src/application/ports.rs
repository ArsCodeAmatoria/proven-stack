//! Repository / outbound ports. Implemented by `infrastructure::memory` (always) and, in a
//! future iteration, `infrastructure::postgres` against the `users` schema. Application services
//! depend only on these traits — never on a concrete storage engine (ADR-0006, mirrors ADR-0004
//! for Core / ADR-0005 for Companies).

use async_trait::async_trait;

use proven_shared::UserId;

use crate::domain::{
    AccessibilityPreferences, AuthenticationProfile, Avatar, DigitalSignatureProfile,
    EmergencyContact, EmergencyContactId, LocalePreferences, NotificationPreferences,
    ProfileAuditEntry, UserKind, UserKindAssignment, UserProfile, UserSetting, UsersError,
};
use crate::events::EventEnvelope;

#[async_trait]
pub trait UserProfileRepository: Send + Sync {
    async fn get(&self, user_id: UserId) -> Result<Option<UserProfile>, UsersError>;
    async fn upsert(&self, profile: &UserProfile) -> Result<(), UsersError>;
}

#[async_trait]
pub trait UserKindRepository: Send + Sync {
    async fn upsert(&self, assignment: &UserKindAssignment) -> Result<(), UsersError>;
    async fn get(
        &self,
        user_id: UserId,
        kind: UserKind,
    ) -> Result<Option<UserKindAssignment>, UsersError>;
    async fn list(&self, user_id: UserId) -> Result<Vec<UserKindAssignment>, UsersError>;
    async fn remove(&self, user_id: UserId, kind: UserKind) -> Result<(), UsersError>;
    /// Unsets `is_primary` on every assignment for `user_id` (used before promoting a new
    /// primary kind, so at most one assignment stays primary).
    async fn clear_primary(&self, user_id: UserId) -> Result<(), UsersError>;
}

#[async_trait]
pub trait AvatarRepository: Send + Sync {
    async fn get(&self, user_id: UserId) -> Result<Option<Avatar>, UsersError>;
    async fn upsert(&self, avatar: &Avatar) -> Result<(), UsersError>;
}

#[async_trait]
pub trait LocaleRepository: Send + Sync {
    async fn get(&self, user_id: UserId) -> Result<Option<LocalePreferences>, UsersError>;
    async fn upsert(&self, prefs: &LocalePreferences) -> Result<(), UsersError>;
}

#[async_trait]
pub trait AccessibilityRepository: Send + Sync {
    async fn get(&self, user_id: UserId) -> Result<Option<AccessibilityPreferences>, UsersError>;
    async fn upsert(&self, prefs: &AccessibilityPreferences) -> Result<(), UsersError>;
}

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn get(&self, user_id: UserId) -> Result<Option<NotificationPreferences>, UsersError>;
    async fn upsert(&self, prefs: &NotificationPreferences) -> Result<(), UsersError>;
}

#[async_trait]
pub trait AuthenticationProfileRepository: Send + Sync {
    async fn get(&self, user_id: UserId) -> Result<Option<AuthenticationProfile>, UsersError>;
    async fn upsert(&self, profile: &AuthenticationProfile) -> Result<(), UsersError>;
}

#[async_trait]
pub trait SignatureProfileRepository: Send + Sync {
    async fn get(&self, user_id: UserId) -> Result<Option<DigitalSignatureProfile>, UsersError>;
    async fn upsert(&self, profile: &DigitalSignatureProfile) -> Result<(), UsersError>;
}

#[async_trait]
pub trait EmergencyContactRepository: Send + Sync {
    async fn insert(&self, contact: &EmergencyContact) -> Result<(), UsersError>;
    async fn get(
        &self,
        user_id: UserId,
        id: EmergencyContactId,
    ) -> Result<Option<EmergencyContact>, UsersError>;
    async fn list(&self, user_id: UserId) -> Result<Vec<EmergencyContact>, UsersError>;
    async fn update(&self, contact: &EmergencyContact) -> Result<(), UsersError>;
    async fn remove(&self, user_id: UserId, id: EmergencyContactId) -> Result<(), UsersError>;
}

#[async_trait]
pub trait UserSettingRepository: Send + Sync {
    async fn get(&self, user_id: UserId, key: &str) -> Result<Option<UserSetting>, UsersError>;
    async fn list(&self, user_id: UserId) -> Result<Vec<UserSetting>, UsersError>;
    async fn upsert(&self, setting: &UserSetting) -> Result<(), UsersError>;
}

#[async_trait]
pub trait ProfileAuditRepository: Send + Sync {
    async fn append(&self, entry: &ProfileAuditEntry) -> Result<(), UsersError>;
    /// Most-recent-first history for `user_id`.
    async fn list(&self, user_id: UserId) -> Result<Vec<ProfileAuditEntry>, UsersError>;
}

/// Outbound event transport (in-memory buffer for tests; NATS/outbox in production, mirroring
/// `proven_core::application::ports::EventPublisher`).
#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, envelope: EventEnvelope) -> Result<(), UsersError>;
}
