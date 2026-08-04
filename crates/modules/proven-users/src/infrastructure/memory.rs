//! Full in-memory store implementing every repository port. Used for unit tests and any
//! no-Postgres deployment mode (mirrors `proven_core::infrastructure::memory` /
//! `proven_companies::infrastructure::memory` — ADR-0006 has no SQL adapter yet; the in-memory
//! store is authoritative for now and is safe for production no-DB deployment modes).

use std::collections::HashMap;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use async_trait::async_trait;
use proven_shared::UserId;
use uuid::Uuid;

use crate::application::ports::{
    AccessibilityRepository, AuthenticationProfileRepository, AvatarRepository,
    EmergencyContactRepository, LocaleRepository, NotificationRepository, ProfileAuditRepository,
    SignatureProfileRepository, UserKindRepository, UserProfileRepository, UserSettingRepository,
};
use crate::domain::{
    AccessibilityPreferences, AuthenticationProfile, Avatar, DigitalSignatureProfile,
    EmergencyContact, EmergencyContactId, LocalePreferences, NotificationPreferences,
    ProfileAuditEntry, UserKind, UserKindAssignment, UserProfile, UserSetting, UsersError,
};

#[derive(Default)]
struct MemoryState {
    profiles: HashMap<Uuid, UserProfile>,
    kinds: HashMap<Uuid, UserKindAssignment>,
    avatars: HashMap<Uuid, Avatar>,
    locale: HashMap<Uuid, LocalePreferences>,
    accessibility: HashMap<Uuid, AccessibilityPreferences>,
    notification: HashMap<Uuid, NotificationPreferences>,
    auth_profiles: HashMap<Uuid, AuthenticationProfile>,
    signature_profiles: HashMap<Uuid, DigitalSignatureProfile>,
    emergency_contacts: HashMap<Uuid, EmergencyContact>,
    settings: HashMap<(Uuid, String), UserSetting>,
    audit: Vec<ProfileAuditEntry>,
}

/// Shared, thread-safe in-memory backing store for every Users port.
#[derive(Default)]
pub struct MemoryStore {
    state: RwLock<MemoryState>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(MemoryState::default()),
        }
    }

    fn read(&self) -> Result<RwLockReadGuard<'_, MemoryState>, UsersError> {
        self.state
            .read()
            .map_err(|_| UsersError::Internal("memory store lock poisoned".into()))
    }

    fn write(&self) -> Result<RwLockWriteGuard<'_, MemoryState>, UsersError> {
        self.state
            .write()
            .map_err(|_| UsersError::Internal("memory store lock poisoned".into()))
    }
}

#[async_trait]
impl UserProfileRepository for MemoryStore {
    async fn get(&self, user_id: UserId) -> Result<Option<UserProfile>, UsersError> {
        Ok(self.read()?.profiles.get(&user_id.as_uuid()).cloned())
    }

    async fn upsert(&self, profile: &UserProfile) -> Result<(), UsersError> {
        self.write()?
            .profiles
            .insert(profile.user_id.as_uuid(), profile.clone());
        Ok(())
    }
}

#[async_trait]
impl UserKindRepository for MemoryStore {
    async fn upsert(&self, assignment: &UserKindAssignment) -> Result<(), UsersError> {
        self.write()?
            .kinds
            .insert(assignment.id.as_uuid(), assignment.clone());
        Ok(())
    }

    async fn get(
        &self,
        user_id: UserId,
        kind: UserKind,
    ) -> Result<Option<UserKindAssignment>, UsersError> {
        Ok(self
            .read()?
            .kinds
            .values()
            .find(|a| a.user_id == user_id && a.kind == kind)
            .cloned())
    }

    async fn list(&self, user_id: UserId) -> Result<Vec<UserKindAssignment>, UsersError> {
        Ok(self
            .read()?
            .kinds
            .values()
            .filter(|a| a.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn remove(&self, user_id: UserId, kind: UserKind) -> Result<(), UsersError> {
        let mut state = self.write()?;
        let id = state
            .kinds
            .values()
            .find(|a| a.user_id == user_id && a.kind == kind)
            .map(|a| a.id.as_uuid());
        match id {
            Some(id) => {
                state.kinds.remove(&id);
                Ok(())
            }
            None => Err(UsersError::NotFound("user_kind")),
        }
    }

    async fn clear_primary(&self, user_id: UserId) -> Result<(), UsersError> {
        let mut state = self.write()?;
        for assignment in state.kinds.values_mut() {
            if assignment.user_id == user_id {
                assignment.is_primary = false;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl AvatarRepository for MemoryStore {
    async fn get(&self, user_id: UserId) -> Result<Option<Avatar>, UsersError> {
        Ok(self.read()?.avatars.get(&user_id.as_uuid()).cloned())
    }

    async fn upsert(&self, avatar: &Avatar) -> Result<(), UsersError> {
        self.write()?
            .avatars
            .insert(avatar.user_id.as_uuid(), avatar.clone());
        Ok(())
    }
}

#[async_trait]
impl LocaleRepository for MemoryStore {
    async fn get(&self, user_id: UserId) -> Result<Option<LocalePreferences>, UsersError> {
        Ok(self.read()?.locale.get(&user_id.as_uuid()).cloned())
    }

    async fn upsert(&self, prefs: &LocalePreferences) -> Result<(), UsersError> {
        self.write()?
            .locale
            .insert(prefs.user_id.as_uuid(), prefs.clone());
        Ok(())
    }
}

#[async_trait]
impl AccessibilityRepository for MemoryStore {
    async fn get(&self, user_id: UserId) -> Result<Option<AccessibilityPreferences>, UsersError> {
        Ok(self.read()?.accessibility.get(&user_id.as_uuid()).cloned())
    }

    async fn upsert(&self, prefs: &AccessibilityPreferences) -> Result<(), UsersError> {
        self.write()?
            .accessibility
            .insert(prefs.user_id.as_uuid(), prefs.clone());
        Ok(())
    }
}

#[async_trait]
impl NotificationRepository for MemoryStore {
    async fn get(&self, user_id: UserId) -> Result<Option<NotificationPreferences>, UsersError> {
        Ok(self.read()?.notification.get(&user_id.as_uuid()).cloned())
    }

    async fn upsert(&self, prefs: &NotificationPreferences) -> Result<(), UsersError> {
        self.write()?
            .notification
            .insert(prefs.user_id.as_uuid(), prefs.clone());
        Ok(())
    }
}

#[async_trait]
impl AuthenticationProfileRepository for MemoryStore {
    async fn get(&self, user_id: UserId) -> Result<Option<AuthenticationProfile>, UsersError> {
        Ok(self.read()?.auth_profiles.get(&user_id.as_uuid()).cloned())
    }

    async fn upsert(&self, profile: &AuthenticationProfile) -> Result<(), UsersError> {
        self.write()?
            .auth_profiles
            .insert(profile.user_id.as_uuid(), profile.clone());
        Ok(())
    }
}

#[async_trait]
impl SignatureProfileRepository for MemoryStore {
    async fn get(&self, user_id: UserId) -> Result<Option<DigitalSignatureProfile>, UsersError> {
        Ok(self
            .read()?
            .signature_profiles
            .get(&user_id.as_uuid())
            .cloned())
    }

    async fn upsert(&self, profile: &DigitalSignatureProfile) -> Result<(), UsersError> {
        self.write()?
            .signature_profiles
            .insert(profile.user_id.as_uuid(), profile.clone());
        Ok(())
    }
}

#[async_trait]
impl EmergencyContactRepository for MemoryStore {
    async fn insert(&self, contact: &EmergencyContact) -> Result<(), UsersError> {
        self.write()?
            .emergency_contacts
            .insert(contact.id.as_uuid(), contact.clone());
        Ok(())
    }

    async fn get(
        &self,
        user_id: UserId,
        id: EmergencyContactId,
    ) -> Result<Option<EmergencyContact>, UsersError> {
        Ok(self
            .read()?
            .emergency_contacts
            .get(&id.as_uuid())
            .filter(|c| c.user_id == user_id)
            .cloned())
    }

    async fn list(&self, user_id: UserId) -> Result<Vec<EmergencyContact>, UsersError> {
        Ok(self
            .read()?
            .emergency_contacts
            .values()
            .filter(|c| c.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn update(&self, contact: &EmergencyContact) -> Result<(), UsersError> {
        self.write()?
            .emergency_contacts
            .insert(contact.id.as_uuid(), contact.clone());
        Ok(())
    }

    async fn remove(&self, user_id: UserId, id: EmergencyContactId) -> Result<(), UsersError> {
        let mut state = self.write()?;
        match state.emergency_contacts.get(&id.as_uuid()) {
            Some(c) if c.user_id == user_id => {
                state.emergency_contacts.remove(&id.as_uuid());
                Ok(())
            }
            _ => Err(UsersError::NotFound("emergency_contact")),
        }
    }
}

#[async_trait]
impl UserSettingRepository for MemoryStore {
    async fn get(&self, user_id: UserId, key: &str) -> Result<Option<UserSetting>, UsersError> {
        Ok(self
            .read()?
            .settings
            .get(&(user_id.as_uuid(), key.to_string()))
            .cloned())
    }

    async fn list(&self, user_id: UserId) -> Result<Vec<UserSetting>, UsersError> {
        Ok(self
            .read()?
            .settings
            .values()
            .filter(|s| s.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn upsert(&self, setting: &UserSetting) -> Result<(), UsersError> {
        self.write()?.settings.insert(
            (setting.user_id.as_uuid(), setting.key.clone()),
            setting.clone(),
        );
        Ok(())
    }
}

#[async_trait]
impl ProfileAuditRepository for MemoryStore {
    async fn append(&self, entry: &ProfileAuditEntry) -> Result<(), UsersError> {
        self.write()?.audit.push(entry.clone());
        Ok(())
    }

    async fn list(&self, user_id: UserId) -> Result<Vec<ProfileAuditEntry>, UsersError> {
        let mut entries: Vec<ProfileAuditEntry> = self
            .read()?
            .audit
            .iter()
            .filter(|e| e.user_id == user_id)
            .cloned()
            .collect();
        entries.sort_by(|a, b| b.occurred_at.cmp(&a.occurred_at));
        Ok(entries)
    }
}
