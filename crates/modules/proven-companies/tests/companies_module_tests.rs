//! Integration-style tests exercising `CompaniesModule` through the public `CompaniesApi` trait
//! only — mirrors how other modules are expected to consume Companies (ADR-0005).

use proven_companies::application::services::{
    AddContactCommand, CreateBusinessUnitCommand, UpsertBrandingCommand,
    UpsertRegionalSettingsCommand, UpsertSafetySettingsCommand,
};
use proven_companies::{ActingContext, CompaniesApi, CompaniesModule, ContactKind, ProfileStatus};
use proven_shared::{CompanyId, PrincipalId, TenantId};

fn new_ctx() -> ActingContext {
    ActingContext::new(TenantId::new(), PrincipalId::new())
}

#[tokio::test]
async fn ensure_profile_creates_defaults() {
    let module = CompaniesModule::in_memory();
    let ctx = new_ctx();
    let company_id = CompanyId::new();

    let profile = module
        .services
        .ensure_profile(ctx, company_id)
        .await
        .expect("ensure_profile should succeed");
    assert_eq!(profile.company_id, company_id);
    assert_eq!(profile.status, ProfileStatus::Active);

    // Default config rows must exist without a separate provisioning step.
    let safety = module
        .services
        .get_safety_settings(company_id)
        .await
        .expect("safety settings default should exist");
    assert!(safety.require_flha_before_work);

    let regional = module
        .services
        .get_regional_settings(company_id)
        .await
        .expect("regional settings default should exist");
    assert!(!regional.primary_region.is_empty());

    let notifications = module
        .services
        .get_notification_defaults(company_id)
        .await
        .expect("notification defaults should exist");
    assert!(notifications.email_enabled);

    let storage = module
        .services
        .get_storage_configuration(company_id)
        .await
        .expect("storage configuration should exist");
    assert!(storage.max_upload_bytes > 0);

    // Idempotent: calling again returns the same profile rather than erroring or duplicating.
    let ensured_again = module
        .services
        .ensure_profile(new_ctx(), company_id)
        .await
        .expect("ensure_profile should be idempotent");
    assert_eq!(ensured_again.company_id, profile.company_id);
    assert_eq!(ensured_again.version, profile.version);
}

#[tokio::test]
async fn business_unit_hierarchy_same_company() {
    let module = CompaniesModule::in_memory();
    let company_a = CompanyId::new();
    let company_b = CompanyId::new();

    let parent = module
        .services
        .create_business_unit(
            new_ctx(),
            CreateBusinessUnitCommand {
                company_id: company_a,
                name: "Western Region".to_string(),
                code: Some("WEST".to_string()),
                parent_id: None,
                org_unit_id: None,
            },
        )
        .await
        .expect("root business unit should be creatable");

    let child = module
        .services
        .create_business_unit(
            new_ctx(),
            CreateBusinessUnitCommand {
                company_id: company_a,
                name: "Vancouver Crew".to_string(),
                code: None,
                parent_id: Some(parent.id),
                org_unit_id: None,
            },
        )
        .await
        .expect("child under a same-company parent should succeed");
    assert_eq!(child.parent_id, Some(parent.id));

    let cross_company_attempt = module
        .services
        .create_business_unit(
            new_ctx(),
            CreateBusinessUnitCommand {
                company_id: company_b,
                name: "Should Fail".to_string(),
                code: None,
                parent_id: Some(parent.id),
                org_unit_id: None,
            },
        )
        .await;
    assert!(
        cross_company_attempt.is_err(),
        "a parent from a different company must be rejected"
    );

    let units = module
        .services
        .list_business_units(company_a)
        .await
        .expect("list should succeed");
    assert_eq!(units.len(), 2);
}

#[tokio::test]
async fn branding_and_settings_round_trip() {
    let module = CompaniesModule::in_memory();
    let company_id = CompanyId::new();
    module
        .services
        .ensure_profile(new_ctx(), company_id)
        .await
        .expect("ensure_profile should succeed");

    let branding = module
        .services
        .upsert_branding(
            new_ctx(),
            UpsertBrandingCommand {
                company_id,
                logo_file_id: None,
                wordmark_file_id: None,
                primary_color: Some("#1A2B3C".to_string()),
                secondary_color: None,
                accent_color: None,
                favicon_file_id: None,
            },
        )
        .await
        .expect("branding upsert should succeed");
    assert_eq!(branding.primary_color.as_deref(), Some("#1A2B3C"));

    let fetched_branding = module
        .services
        .get_branding(company_id)
        .await
        .expect("branding should round-trip");
    assert_eq!(fetched_branding.primary_color, branding.primary_color);

    let safety = module
        .services
        .upsert_safety_settings(
            new_ctx(),
            UpsertSafetySettingsCommand {
                company_id,
                require_flha_before_work: Some(false),
                require_toolbox_talk_weekly: None,
                incident_notify_emails: Some(vec!["safety@acme.test".to_string()]),
                default_risk_matrix: None,
                allow_offline_safety_submit: None,
            },
        )
        .await
        .expect("safety settings upsert should succeed");
    assert!(!safety.require_flha_before_work);
    assert_eq!(
        safety.incident_notify_emails,
        vec!["safety@acme.test".to_string()]
    );

    let regional = module
        .services
        .upsert_regional_settings(
            new_ctx(),
            UpsertRegionalSettingsCommand {
                company_id,
                primary_region: Some("CA-BC".to_string()),
                locales: Some(vec!["en-CA".to_string()]),
                timezone: Some("America/Vancouver".to_string()),
                measurement_system: None,
                currency_code: Some("CAD".to_string()),
            },
        )
        .await
        .expect("regional settings upsert should succeed");
    assert_eq!(regional.primary_region, "CA-BC");
    assert_eq!(regional.currency_code, "CAD");

    // A malformed currency code must be rejected.
    let bad_currency = module
        .services
        .upsert_regional_settings(
            new_ctx(),
            UpsertRegionalSettingsCommand {
                company_id,
                primary_region: None,
                locales: None,
                timezone: None,
                measurement_system: None,
                currency_code: Some("Canadian-Dollar".to_string()),
            },
        )
        .await;
    assert!(
        bad_currency.is_err(),
        "non-3-letter currency codes must be rejected"
    );
}

#[tokio::test]
async fn archive_profile() {
    let module = CompaniesModule::in_memory();
    let company_id = CompanyId::new();
    module
        .services
        .ensure_profile(new_ctx(), company_id)
        .await
        .expect("ensure_profile should succeed");

    let archived = module
        .services
        .archive_profile(new_ctx(), company_id)
        .await
        .expect("archive_profile should succeed");
    assert_eq!(archived.status, ProfileStatus::Archived);

    let fetched = module
        .services
        .get_profile(company_id)
        .await
        .expect("archived profile should still be fetchable");
    assert_eq!(fetched.status, ProfileStatus::Archived);

    // Mutating an archived profile is a conflict, not a silent success.
    let update_attempt = module
        .services
        .update_profile(
            new_ctx(),
            proven_companies::application::services::UpdateProfileCommand {
                company_id,
                trade_name: Some("Should Not Apply".to_string()),
                website: None,
                notes: None,
            },
        )
        .await;
    assert!(
        update_attempt.is_err(),
        "updating an archived profile must fail"
    );
}

#[tokio::test]
async fn contact_email_validation_rejects_bad_email() {
    let module = CompaniesModule::in_memory();
    let company_id = CompanyId::new();

    let bad_contact = module
        .services
        .add_contact(
            new_ctx(),
            AddContactCommand {
                company_id,
                business_unit_id: None,
                kind: ContactKind::Safety,
                full_name: "Jamie Safety Lead".to_string(),
                title: None,
                email: Some("not-an-email".to_string()),
                phone: None,
                user_id: None,
                is_primary: true,
            },
        )
        .await;
    assert!(bad_contact.is_err(), "malformed email must be rejected");

    let good_contact = module
        .services
        .add_contact(
            new_ctx(),
            AddContactCommand {
                company_id,
                business_unit_id: None,
                kind: ContactKind::Safety,
                full_name: "Jamie Safety Lead".to_string(),
                title: None,
                email: Some("jamie@acme.test".to_string()),
                phone: None,
                user_id: None,
                is_primary: true,
            },
        )
        .await
        .expect("valid email should be accepted");
    assert_eq!(good_contact.email.as_deref(), Some("jamie@acme.test"));

    let contacts = module
        .services
        .list_contacts(company_id)
        .await
        .expect("list should succeed");
    assert_eq!(contacts.len(), 1);
}
