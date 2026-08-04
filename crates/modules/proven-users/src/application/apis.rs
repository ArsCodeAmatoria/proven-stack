//! In-process public interface (ADR-0006 §3). Every other module and Temporal activity talks to
//! Users exclusively through [`UsersApi`] — never through this module's schema.

use std::sync::Arc;

use async_trait::async_trait;
use proven_core::{AuthzApi, IdentityApi};
use proven_shared::UserId;

use crate::application::ports::{
    AccessibilityRepository, AuthenticationProfileRepository, AvatarRepository,
    EmergencyContactRepository, EventPublisher, LocaleRepository, NotificationRepository,
    ProfileAuditRepository, SignatureProfileRepository, UserKindRepository, UserProfileRepository,
    UserSettingRepository,
};
use crate::application::services::{
    AccessibilityService, ActingContext, AddEmergencyContactCommand, AllowAllAuthz,
    AssignUserKindCommand, AuditHistoryService, AuditRecorder, AuthProfileService, AvatarService,
    EmergencyContactService, KindService, LocaleService, NotificationService, ProfileService,
    SettingsService, SignatureService, UpdateEmergencyContactCommand, UpdateProfileCommand,
    UpsertAccessibilityCommand, UpsertAuthenticationProfileCommand, UpsertAvatarCommand,
    UpsertLocaleCommand, UpsertNotificationPreferencesCommand, UpsertSignatureProfileCommand,
    UpsertUserSettingCommand,
};
use crate::domain::{
    AccessibilityPreferences, AuthenticationProfile, Avatar, DigitalSignatureProfile,
    EmergencyContact, EmergencyContactId, LocalePreferences, NotificationPreferences,
    ProfileAuditEntry, UserKind, UserKindAssignment, UserProfile, UserSetting, UsersError,
};
use crate::infrastructure::memory::MemoryStore;
use crate::infrastructure::outbox::InMemoryOutbox;

/// Facade covering every Users capability (ADR-0006 §3). Mutations take an [`ActingContext`] so
/// implementations can enforce tenant scoping + AuthZ; reads are keyed by `UserId` alone.
#[async_trait]
pub trait UsersApi: Send + Sync {
    async fn ensure_profile(
        &self,
        ctx: ActingContext,
        user_id: UserId,
        display_name: String,
    ) -> Result<UserProfile, UsersError>;
    async fn get_profile(&self, user_id: UserId) -> Result<UserProfile, UsersError>;
    async fn update_profile(
        &self,
        ctx: ActingContext,
        cmd: UpdateProfileCommand,
    ) -> Result<UserProfile, UsersError>;
    async fn archive_profile(
        &self,
        ctx: ActingContext,
        user_id: UserId,
    ) -> Result<UserProfile, UsersError>;

    async fn assign_kind(
        &self,
        ctx: ActingContext,
        cmd: AssignUserKindCommand,
    ) -> Result<UserKindAssignment, UsersError>;
    async fn remove_kind(
        &self,
        ctx: ActingContext,
        user_id: UserId,
        kind: UserKind,
    ) -> Result<(), UsersError>;
    async fn list_kinds(&self, user_id: UserId) -> Result<Vec<UserKindAssignment>, UsersError>;

    async fn get_avatar(&self, user_id: UserId) -> Result<Avatar, UsersError>;
    async fn upsert_avatar(
        &self,
        ctx: ActingContext,
        cmd: UpsertAvatarCommand,
    ) -> Result<Avatar, UsersError>;

    async fn get_locale(&self, user_id: UserId) -> Result<LocalePreferences, UsersError>;
    async fn upsert_locale(
        &self,
        ctx: ActingContext,
        cmd: UpsertLocaleCommand,
    ) -> Result<LocalePreferences, UsersError>;

    async fn get_accessibility(
        &self,
        user_id: UserId,
    ) -> Result<AccessibilityPreferences, UsersError>;
    async fn upsert_accessibility(
        &self,
        ctx: ActingContext,
        cmd: UpsertAccessibilityCommand,
    ) -> Result<AccessibilityPreferences, UsersError>;

    async fn get_notification_preferences(
        &self,
        user_id: UserId,
    ) -> Result<NotificationPreferences, UsersError>;
    async fn upsert_notification_preferences(
        &self,
        ctx: ActingContext,
        cmd: UpsertNotificationPreferencesCommand,
    ) -> Result<NotificationPreferences, UsersError>;

    async fn get_authentication_profile(
        &self,
        user_id: UserId,
    ) -> Result<AuthenticationProfile, UsersError>;
    async fn upsert_authentication_profile(
        &self,
        ctx: ActingContext,
        cmd: UpsertAuthenticationProfileCommand,
    ) -> Result<AuthenticationProfile, UsersError>;

    async fn get_signature_profile(
        &self,
        user_id: UserId,
    ) -> Result<DigitalSignatureProfile, UsersError>;
    async fn upsert_signature_profile(
        &self,
        ctx: ActingContext,
        cmd: UpsertSignatureProfileCommand,
    ) -> Result<DigitalSignatureProfile, UsersError>;

    async fn add_emergency_contact(
        &self,
        ctx: ActingContext,
        cmd: AddEmergencyContactCommand,
    ) -> Result<EmergencyContact, UsersError>;
    async fn list_emergency_contacts(
        &self,
        user_id: UserId,
    ) -> Result<Vec<EmergencyContact>, UsersError>;
    async fn update_emergency_contact(
        &self,
        ctx: ActingContext,
        cmd: UpdateEmergencyContactCommand,
    ) -> Result<EmergencyContact, UsersError>;
    async fn remove_emergency_contact(
        &self,
        ctx: ActingContext,
        user_id: UserId,
        contact_id: EmergencyContactId,
    ) -> Result<(), UsersError>;

    async fn get_setting(&self, user_id: UserId, key: String) -> Result<UserSetting, UsersError>;
    async fn list_settings(&self, user_id: UserId) -> Result<Vec<UserSetting>, UsersError>;
    async fn upsert_setting(
        &self,
        ctx: ActingContext,
        cmd: UpsertUserSettingCommand,
    ) -> Result<UserSetting, UsersError>;

    async fn list_audit_history(
        &self,
        ctx: ActingContext,
        user_id: UserId,
    ) -> Result<Vec<ProfileAuditEntry>, UsersError>;
}

/// Bundle of repository ports used to construct [`UsersServices`]. Swap `infrastructure::memory`
/// for a future `infrastructure::postgres` adapter without touching application logic
/// (ADR-0006, mirrors ADR-0004 for Core / ADR-0005 for Companies).
pub struct UsersPorts {
    pub profiles: Arc<dyn UserProfileRepository>,
    pub kinds: Arc<dyn UserKindRepository>,
    pub avatars: Arc<dyn AvatarRepository>,
    pub locale: Arc<dyn LocaleRepository>,
    pub accessibility: Arc<dyn AccessibilityRepository>,
    pub notification: Arc<dyn NotificationRepository>,
    pub auth_profiles: Arc<dyn AuthenticationProfileRepository>,
    pub signature_profiles: Arc<dyn SignatureProfileRepository>,
    pub emergency_contacts: Arc<dyn EmergencyContactRepository>,
    pub settings: Arc<dyn UserSettingRepository>,
    pub audit: Arc<dyn ProfileAuditRepository>,
    pub outbox: Arc<dyn EventPublisher>,
}

impl UsersPorts {
    /// Wire every port to a single shared in-memory store (unit tests / no-DB mode).
    pub fn in_memory() -> Self {
        let store = Arc::new(MemoryStore::new());
        let outbox = Arc::new(InMemoryOutbox::new());
        Self {
            profiles: store.clone(),
            kinds: store.clone(),
            avatars: store.clone(),
            locale: store.clone(),
            accessibility: store.clone(),
            notification: store.clone(),
            auth_profiles: store.clone(),
            signature_profiles: store.clone(),
            emergency_contacts: store.clone(),
            settings: store.clone(),
            audit: store,
            outbox,
        }
    }
}

/// Facade implementing [`UsersApi`] — the seam other modules depend on.
pub struct UsersServices {
    profile: ProfileService,
    kinds: KindService,
    avatar: AvatarService,
    locale: LocaleService,
    accessibility: AccessibilityService,
    notification: NotificationService,
    auth_profile: AuthProfileService,
    signature: SignatureService,
    emergency_contacts: EmergencyContactService,
    settings: SettingsService,
    audit_history: AuditHistoryService,
}

impl UsersServices {
    /// Build `UsersServices` from ports plus an AuthZ decision path, optionally wiring an
    /// `IdentityApi` so `ensure_profile` can verify the Core user exists first.
    pub fn new(
        ports: UsersPorts,
        authz: Arc<dyn AuthzApi>,
        identity: Option<Arc<dyn IdentityApi>>,
    ) -> Self {
        let audit = Arc::new(AuditRecorder::new(
            ports.audit.clone(),
            ports.outbox.clone(),
        ));

        Self {
            profile: ProfileService::new(
                ports.profiles.clone(),
                ports.locale.clone(),
                ports.accessibility.clone(),
                ports.notification.clone(),
                ports.auth_profiles.clone(),
                ports.signature_profiles.clone(),
                audit.clone(),
                authz.clone(),
                identity,
            ),
            kinds: KindService::new(ports.kinds, ports.profiles, audit.clone(), authz.clone()),
            avatar: AvatarService::new(ports.avatars, audit.clone(), authz.clone()),
            locale: LocaleService::new(ports.locale, audit.clone(), authz.clone()),
            accessibility: AccessibilityService::new(
                ports.accessibility,
                audit.clone(),
                authz.clone(),
            ),
            notification: NotificationService::new(
                ports.notification,
                audit.clone(),
                authz.clone(),
            ),
            auth_profile: AuthProfileService::new(
                ports.auth_profiles,
                audit.clone(),
                authz.clone(),
            ),
            signature: SignatureService::new(
                ports.signature_profiles,
                audit.clone(),
                authz.clone(),
            ),
            emergency_contacts: EmergencyContactService::new(
                ports.emergency_contacts,
                audit.clone(),
                authz.clone(),
            ),
            settings: SettingsService::new(ports.settings, audit.clone(), authz.clone()),
            audit_history: AuditHistoryService::new(ports.audit, authz),
        }
    }

    /// In-memory ports + a stub Allow-all AuthZ, no `IdentityApi` wired (so `ensure_profile`
    /// skips the Core user existence check). Intended for this crate's own unit tests.
    pub fn in_memory_unchecked() -> Self {
        Self::new(UsersPorts::in_memory(), Arc::new(AllowAllAuthz), None)
    }

    /// Wire real `proven-core` `AuthzApi` + `IdentityApi` (`CoreServices` implements both) over
    /// in-memory ports. Use this path outside of unit tests.
    pub fn with_core<C>(ports: UsersPorts, core: Arc<C>) -> Self
    where
        C: AuthzApi + IdentityApi + Send + Sync + 'static,
    {
        let authz: Arc<dyn AuthzApi> = core.clone();
        let identity: Arc<dyn IdentityApi> = core;
        Self::new(ports, authz, Some(identity))
    }
}

#[async_trait]
impl UsersApi for UsersServices {
    async fn ensure_profile(
        &self,
        ctx: ActingContext,
        user_id: UserId,
        display_name: String,
    ) -> Result<UserProfile, UsersError> {
        self.profile
            .ensure_profile(&ctx, user_id, display_name)
            .await
    }

    async fn get_profile(&self, user_id: UserId) -> Result<UserProfile, UsersError> {
        self.profile.get_profile(user_id).await
    }

    async fn update_profile(
        &self,
        ctx: ActingContext,
        cmd: UpdateProfileCommand,
    ) -> Result<UserProfile, UsersError> {
        self.profile.update_profile(&ctx, cmd).await
    }

    async fn archive_profile(
        &self,
        ctx: ActingContext,
        user_id: UserId,
    ) -> Result<UserProfile, UsersError> {
        self.profile.archive_profile(&ctx, user_id).await
    }

    async fn assign_kind(
        &self,
        ctx: ActingContext,
        cmd: AssignUserKindCommand,
    ) -> Result<UserKindAssignment, UsersError> {
        self.kinds.assign(&ctx, cmd).await
    }

    async fn remove_kind(
        &self,
        ctx: ActingContext,
        user_id: UserId,
        kind: UserKind,
    ) -> Result<(), UsersError> {
        self.kinds.remove(&ctx, user_id, kind).await
    }

    async fn list_kinds(&self, user_id: UserId) -> Result<Vec<UserKindAssignment>, UsersError> {
        self.kinds.list(user_id).await
    }

    async fn get_avatar(&self, user_id: UserId) -> Result<Avatar, UsersError> {
        self.avatar.get(user_id).await
    }

    async fn upsert_avatar(
        &self,
        ctx: ActingContext,
        cmd: UpsertAvatarCommand,
    ) -> Result<Avatar, UsersError> {
        self.avatar.upsert(&ctx, cmd).await
    }

    async fn get_locale(&self, user_id: UserId) -> Result<LocalePreferences, UsersError> {
        self.locale.get(user_id).await
    }

    async fn upsert_locale(
        &self,
        ctx: ActingContext,
        cmd: UpsertLocaleCommand,
    ) -> Result<LocalePreferences, UsersError> {
        self.locale.upsert(&ctx, cmd).await
    }

    async fn get_accessibility(
        &self,
        user_id: UserId,
    ) -> Result<AccessibilityPreferences, UsersError> {
        self.accessibility.get(user_id).await
    }

    async fn upsert_accessibility(
        &self,
        ctx: ActingContext,
        cmd: UpsertAccessibilityCommand,
    ) -> Result<AccessibilityPreferences, UsersError> {
        self.accessibility.upsert(&ctx, cmd).await
    }

    async fn get_notification_preferences(
        &self,
        user_id: UserId,
    ) -> Result<NotificationPreferences, UsersError> {
        self.notification.get(user_id).await
    }

    async fn upsert_notification_preferences(
        &self,
        ctx: ActingContext,
        cmd: UpsertNotificationPreferencesCommand,
    ) -> Result<NotificationPreferences, UsersError> {
        self.notification.upsert(&ctx, cmd).await
    }

    async fn get_authentication_profile(
        &self,
        user_id: UserId,
    ) -> Result<AuthenticationProfile, UsersError> {
        self.auth_profile.get(user_id).await
    }

    async fn upsert_authentication_profile(
        &self,
        ctx: ActingContext,
        cmd: UpsertAuthenticationProfileCommand,
    ) -> Result<AuthenticationProfile, UsersError> {
        self.auth_profile.upsert(&ctx, cmd).await
    }

    async fn get_signature_profile(
        &self,
        user_id: UserId,
    ) -> Result<DigitalSignatureProfile, UsersError> {
        self.signature.get(user_id).await
    }

    async fn upsert_signature_profile(
        &self,
        ctx: ActingContext,
        cmd: UpsertSignatureProfileCommand,
    ) -> Result<DigitalSignatureProfile, UsersError> {
        self.signature.upsert(&ctx, cmd).await
    }

    async fn add_emergency_contact(
        &self,
        ctx: ActingContext,
        cmd: AddEmergencyContactCommand,
    ) -> Result<EmergencyContact, UsersError> {
        self.emergency_contacts.add(&ctx, cmd).await
    }

    async fn list_emergency_contacts(
        &self,
        user_id: UserId,
    ) -> Result<Vec<EmergencyContact>, UsersError> {
        self.emergency_contacts.list(user_id).await
    }

    async fn update_emergency_contact(
        &self,
        ctx: ActingContext,
        cmd: UpdateEmergencyContactCommand,
    ) -> Result<EmergencyContact, UsersError> {
        self.emergency_contacts.update(&ctx, cmd).await
    }

    async fn remove_emergency_contact(
        &self,
        ctx: ActingContext,
        user_id: UserId,
        contact_id: EmergencyContactId,
    ) -> Result<(), UsersError> {
        self.emergency_contacts
            .remove(&ctx, user_id, contact_id)
            .await
    }

    async fn get_setting(&self, user_id: UserId, key: String) -> Result<UserSetting, UsersError> {
        self.settings.get(user_id, &key).await
    }

    async fn list_settings(&self, user_id: UserId) -> Result<Vec<UserSetting>, UsersError> {
        self.settings.list(user_id).await
    }

    async fn upsert_setting(
        &self,
        ctx: ActingContext,
        cmd: UpsertUserSettingCommand,
    ) -> Result<UserSetting, UsersError> {
        self.settings.upsert(&ctx, cmd).await
    }

    async fn list_audit_history(
        &self,
        ctx: ActingContext,
        user_id: UserId,
    ) -> Result<Vec<ProfileAuditEntry>, UsersError> {
        self.audit_history.list(&ctx, user_id).await
    }
}
