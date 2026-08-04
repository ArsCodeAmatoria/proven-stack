//! Integration-style tests exercising `UsersModule` through the public `UsersApi` trait only —
//! mirrors how other modules are expected to consume Users (ADR-0006).

use proven_shared::{PrincipalId, TenantId, UserId};
use proven_users::application::services::{
    AddEmergencyContactCommand, AssignUserKindCommand, UpdateEmergencyContactCommand,
    UpsertAccessibilityCommand, UpsertAuthenticationProfileCommand, UpsertLocaleCommand,
    UpsertNotificationPreferencesCommand, UpsertSignatureProfileCommand,
};
use proven_users::{
    ActingContext, DigestCadence, ProfileStatus, SignatureType, UserKind, UsersApi, UsersModule,
};

fn new_ctx() -> ActingContext {
    ActingContext::new(TenantId::new(), PrincipalId::new())
}

#[tokio::test]
async fn ensure_profile_creates_defaults() {
    let module = UsersModule::in_memory();
    let ctx = new_ctx();
    let user_id = UserId::new();

    let profile = module
        .services
        .ensure_profile(ctx, user_id, "Jamie Worker".to_string())
        .await
        .expect("ensure_profile should succeed");
    assert_eq!(profile.user_id, user_id);
    assert_eq!(profile.status, ProfileStatus::Active);
    assert_eq!(profile.display_name, "Jamie Worker");

    // Default preference rows must exist without a separate provisioning step.
    let locale = module
        .services
        .get_locale(user_id)
        .await
        .expect("locale defaults should exist");
    assert_eq!(locale.language_code, "en");

    let accessibility = module
        .services
        .get_accessibility(user_id)
        .await
        .expect("accessibility defaults should exist");
    assert!(!accessibility.reduce_motion);

    let notifications = module
        .services
        .get_notification_preferences(user_id)
        .await
        .expect("notification defaults should exist");
    assert!(notifications.email_enabled);

    let auth_profile = module
        .services
        .get_authentication_profile(user_id)
        .await
        .expect("auth profile defaults should exist");
    assert!(auth_profile.password_login_enabled);

    let signature_profile = module
        .services
        .get_signature_profile(user_id)
        .await
        .expect("signature profile defaults should exist");
    assert_eq!(
        signature_profile.default_signature_type,
        SignatureType::Drawn
    );

    // Idempotent: calling again returns the same profile rather than erroring or duplicating.
    let ensured_again = module
        .services
        .ensure_profile(new_ctx(), user_id, "Should Not Apply".to_string())
        .await
        .expect("ensure_profile should be idempotent");
    assert_eq!(ensured_again.display_name, profile.display_name);
    assert_eq!(ensured_again.version, profile.version);
}

#[tokio::test]
async fn assign_user_kinds_worker_and_supervisor() {
    let module = UsersModule::in_memory();
    let user_id = UserId::new();
    module
        .services
        .ensure_profile(new_ctx(), user_id, "Jamie Worker".to_string())
        .await
        .expect("ensure_profile should succeed");

    let worker = module
        .services
        .assign_kind(
            new_ctx(),
            AssignUserKindCommand {
                user_id,
                kind: UserKind::Worker,
                is_primary: true,
            },
        )
        .await
        .expect("assigning worker kind should succeed");
    assert!(worker.is_primary);

    let supervisor = module
        .services
        .assign_kind(
            new_ctx(),
            AssignUserKindCommand {
                user_id,
                kind: UserKind::Supervisor,
                is_primary: true,
            },
        )
        .await
        .expect("assigning supervisor kind should succeed");
    assert!(supervisor.is_primary);

    let kinds = module
        .services
        .list_kinds(user_id)
        .await
        .expect("list_kinds should succeed");
    assert_eq!(kinds.len(), 2);

    // Promoting supervisor to primary must demote worker — at most one primary kind.
    let primaries: Vec<_> = kinds.iter().filter(|k| k.is_primary).collect();
    assert_eq!(primaries.len(), 1);
    assert_eq!(primaries[0].kind, UserKind::Supervisor);

    module
        .services
        .remove_kind(new_ctx(), user_id, UserKind::Worker)
        .await
        .expect("removing worker kind should succeed");
    let kinds_after_removal = module
        .services
        .list_kinds(user_id)
        .await
        .expect("list_kinds should succeed");
    assert_eq!(kinds_after_removal.len(), 1);
    assert_eq!(kinds_after_removal[0].kind, UserKind::Supervisor);

    let remove_missing = module
        .services
        .remove_kind(new_ctx(), user_id, UserKind::Worker)
        .await;
    assert!(
        remove_missing.is_err(),
        "removing an already-removed kind must fail"
    );
}

#[tokio::test]
async fn locale_accessibility_notification_round_trip() {
    let module = UsersModule::in_memory();
    let user_id = UserId::new();
    module
        .services
        .ensure_profile(new_ctx(), user_id, "Jamie Worker".to_string())
        .await
        .expect("ensure_profile should succeed");

    let locale = module
        .services
        .upsert_locale(
            new_ctx(),
            UpsertLocaleCommand {
                user_id,
                language_code: Some("fr".to_string()),
                time_zone: Some("America/Vancouver".to_string()),
            },
        )
        .await
        .expect("locale upsert should succeed");
    assert_eq!(locale.language_code, "fr");
    assert_eq!(locale.time_zone, "America/Vancouver");

    let accessibility = module
        .services
        .upsert_accessibility(
            new_ctx(),
            UpsertAccessibilityCommand {
                user_id,
                reduce_motion: Some(true),
                high_contrast: None,
                large_text: None,
                screen_reader_hints: None,
            },
        )
        .await
        .expect("accessibility upsert should succeed");
    assert!(accessibility.reduce_motion);
    assert!(!accessibility.high_contrast);

    let notifications = module
        .services
        .upsert_notification_preferences(
            new_ctx(),
            UpsertNotificationPreferencesCommand {
                user_id,
                email_enabled: Some(false),
                push_enabled: None,
                sms_enabled: None,
                in_app_enabled: None,
                digest_cadence: Some(DigestCadence::Weekly),
                quiet_hours_start: Some("22:00".to_string()),
                quiet_hours_end: Some("06:00".to_string()),
            },
        )
        .await
        .expect("notification upsert should succeed");
    assert!(!notifications.email_enabled);
    assert_eq!(notifications.digest_cadence, DigestCadence::Weekly);
    assert_eq!(notifications.quiet_hours_start.as_deref(), Some("22:00"));

    // A malformed quiet-hours value must be rejected.
    let bad_quiet_hours = module
        .services
        .upsert_notification_preferences(
            new_ctx(),
            UpsertNotificationPreferencesCommand {
                user_id,
                email_enabled: None,
                push_enabled: None,
                sms_enabled: None,
                in_app_enabled: None,
                digest_cadence: None,
                quiet_hours_start: Some("not-a-time".to_string()),
                quiet_hours_end: None,
            },
        )
        .await;
    assert!(
        bad_quiet_hours.is_err(),
        "malformed quiet hours must be rejected"
    );

    // Round-trip through the getters too.
    let fetched_locale = module.services.get_locale(user_id).await.unwrap();
    assert_eq!(fetched_locale.language_code, "fr");
}

#[tokio::test]
async fn emergency_contact_validation() {
    let module = UsersModule::in_memory();
    let user_id = UserId::new();
    module
        .services
        .ensure_profile(new_ctx(), user_id, "Jamie Worker".to_string())
        .await
        .expect("ensure_profile should succeed");

    let missing_phone = module
        .services
        .add_emergency_contact(
            new_ctx(),
            AddEmergencyContactCommand {
                user_id,
                full_name: "Alex Family".to_string(),
                relationship: Some("Spouse".to_string()),
                phone: "".to_string(),
                email: None,
                is_primary: true,
            },
        )
        .await;
    assert!(missing_phone.is_err(), "empty phone must be rejected");

    let bad_email = module
        .services
        .add_emergency_contact(
            new_ctx(),
            AddEmergencyContactCommand {
                user_id,
                full_name: "Alex Family".to_string(),
                relationship: Some("Spouse".to_string()),
                phone: "555-0100".to_string(),
                email: Some("not-an-email".to_string()),
                is_primary: true,
            },
        )
        .await;
    assert!(bad_email.is_err(), "malformed email must be rejected");

    let contact = module
        .services
        .add_emergency_contact(
            new_ctx(),
            AddEmergencyContactCommand {
                user_id,
                full_name: "Alex Family".to_string(),
                relationship: Some("Spouse".to_string()),
                phone: "555-0100".to_string(),
                email: Some("alex@example.test".to_string()),
                is_primary: true,
            },
        )
        .await
        .expect("valid emergency contact should be accepted");
    assert_eq!(contact.full_name, "Alex Family");

    let updated = module
        .services
        .update_emergency_contact(
            new_ctx(),
            UpdateEmergencyContactCommand {
                user_id,
                contact_id: contact.id,
                full_name: None,
                relationship: None,
                phone: Some("555-0199".to_string()),
                email: None,
                is_primary: None,
            },
        )
        .await
        .expect("update should succeed");
    assert_eq!(updated.phone, "555-0199");

    let contacts = module
        .services
        .list_emergency_contacts(user_id)
        .await
        .expect("list should succeed");
    assert_eq!(contacts.len(), 1);

    module
        .services
        .remove_emergency_contact(new_ctx(), user_id, contact.id)
        .await
        .expect("remove should succeed");
    let contacts_after_removal = module
        .services
        .list_emergency_contacts(user_id)
        .await
        .expect("list should succeed");
    assert!(contacts_after_removal.is_empty());
}

#[tokio::test]
async fn archive_profile() {
    let module = UsersModule::in_memory();
    let user_id = UserId::new();
    module
        .services
        .ensure_profile(new_ctx(), user_id, "Jamie Worker".to_string())
        .await
        .expect("ensure_profile should succeed");

    let archived = module
        .services
        .archive_profile(new_ctx(), user_id)
        .await
        .expect("archive_profile should succeed");
    assert_eq!(archived.status, ProfileStatus::Archived);

    let fetched = module
        .services
        .get_profile(user_id)
        .await
        .expect("archived profile should still be fetchable");
    assert_eq!(fetched.status, ProfileStatus::Archived);

    // Mutating an archived profile is a conflict, not a silent success.
    let update_attempt = module
        .services
        .update_profile(
            new_ctx(),
            proven_users::application::services::UpdateProfileCommand {
                user_id,
                display_name: Some("Should Not Apply".to_string()),
                preferred_name: None,
                job_title: None,
                phone: None,
                bio: None,
            },
        )
        .await;
    assert!(
        update_attempt.is_err(),
        "updating an archived profile must fail"
    );

    // The profile audit trail should have recorded ensure + archive.
    let history = module
        .services
        .list_audit_history(new_ctx(), user_id)
        .await
        .expect("audit history should be readable");
    assert!(history.len() >= 2);
}

#[tokio::test]
async fn signature_and_auth_profile_round_trip() {
    let module = UsersModule::in_memory();
    let user_id = UserId::new();
    module
        .services
        .ensure_profile(new_ctx(), user_id, "Jamie Worker".to_string())
        .await
        .expect("ensure_profile should succeed");

    let auth_profile = module
        .services
        .upsert_authentication_profile(
            new_ctx(),
            UpsertAuthenticationProfileCommand {
                user_id,
                mfa_preferred: Some(true),
                password_login_enabled: Some(false),
                oauth_google_linked: Some(true),
                oauth_microsoft_linked: None,
                magic_link_preferred: None,
                last_auth_method: Some("oauth_google".to_string()),
            },
        )
        .await
        .expect("auth profile upsert should succeed");
    assert!(auth_profile.mfa_preferred);
    assert!(!auth_profile.password_login_enabled);
    assert!(auth_profile.oauth_google_linked);
    assert_eq!(
        auth_profile.last_auth_method.as_deref(),
        Some("oauth_google")
    );

    let signature_profile = module
        .services
        .upsert_signature_profile(
            new_ctx(),
            UpsertSignatureProfileCommand {
                user_id,
                default_signature_type: Some(SignatureType::Typed),
                typed_name_default: Some("Jamie Worker".to_string()),
                signature_image_file_id: None,
                require_reauth_to_sign: Some(true),
            },
        )
        .await
        .expect("signature profile upsert should succeed");
    assert_eq!(
        signature_profile.default_signature_type,
        SignatureType::Typed
    );
    assert_eq!(
        signature_profile.typed_name_default.as_deref(),
        Some("Jamie Worker")
    );
    assert!(signature_profile.require_reauth_to_sign);

    // Round-trip through the getters too.
    let fetched_auth = module
        .services
        .get_authentication_profile(user_id)
        .await
        .expect("auth profile should round-trip");
    assert_eq!(fetched_auth.last_auth_method, auth_profile.last_auth_method);

    let fetched_signature = module
        .services
        .get_signature_profile(user_id)
        .await
        .expect("signature profile should round-trip");
    assert_eq!(
        fetched_signature.default_signature_type,
        signature_profile.default_signature_type
    );
}
