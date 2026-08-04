//! Projects module tests — create, update, archive, membership orchestration (ADR-0009).

use proven_core::application::services::{
    InviteUserCommand, ProvisionTenantCommand, RegisterCompanyCommand,
};
use proven_core::domain::CompanyType;
use proven_core::{CoreModule, IdentityApi};
use proven_projects::application::services::{
    AssignProjectMembershipCommand, CreateProjectCommand, UpdateProjectCommand,
};
use proven_projects::{
    ActingContext, ProjectLocation, ProjectStatus, ProjectsApi, ProjectsModule,
};
use proven_shared::{CompanyId, PrincipalId, RegionCode, TenantId, UserId};

fn ctx(tenant_id: TenantId) -> ActingContext {
    ActingContext::new(tenant_id, PrincipalId::new())
}

#[tokio::test]
async fn create_update_archive_project() {
    let module = ProjectsModule::in_memory();
    let tenant_id = TenantId::new();
    let prime = CompanyId::new();
    let client = CompanyId::new();

    let created = module
        .services
        .create_project(
            ctx(tenant_id),
            CreateProjectCommand {
                code: "JOB-100".to_string(),
                name: "River Bridge".to_string(),
                description: Some("Phase 1".to_string()),
                location: Some(ProjectLocation {
                    line1: "100 River Rd".to_string(),
                    line2: None,
                    city: "Vancouver".to_string(),
                    region: Some("BC".to_string()),
                    postal_code: Some("V6B1A1".to_string()),
                    country_code: "CA".to_string(),
                    timezone: Some("America/Vancouver".to_string()),
                }),
                prime_contractor_company_id: prime,
                client_company_id: Some(client),
                planned_start: None,
                planned_end: None,
            },
        )
        .await
        .expect("create should succeed");

    assert_eq!(created.status, ProjectStatus::Planning);
    assert_eq!(created.prime_contractor_company_id, prime);
    assert_eq!(created.client_company_id, Some(client));
    assert_eq!(
        created.location.as_ref().map(|l| l.city.as_str()),
        Some("Vancouver")
    );

    let participants = module
        .services
        .list_participants(created.id)
        .await
        .expect("list participants");
    assert_eq!(participants.len(), 2);

    let updated = module
        .services
        .update_project(
            ctx(tenant_id),
            UpdateProjectCommand {
                project_id: created.id,
                name: Some("River Bridge Rehab".to_string()),
                description: None,
                location: None,
                client_company_id: None,
                planned_start: None,
                planned_end: None,
            },
        )
        .await
        .expect("update should succeed");
    assert_eq!(updated.name, "River Bridge Rehab");
    assert_eq!(updated.version, 2);

    let archived = module
        .services
        .archive_project(ctx(tenant_id), created.id)
        .await
        .expect("archive should succeed");
    assert_eq!(archived.status, ProjectStatus::Archived);

    let listed = module
        .services
        .list_projects(tenant_id, false)
        .await
        .expect("list default excludes archived");
    assert!(listed.is_empty());

    let listed_all = module
        .services
        .list_projects(tenant_id, true)
        .await
        .expect("list including archived");
    assert_eq!(listed_all.len(), 1);

    let update_archived = module
        .services
        .update_project(
            ctx(tenant_id),
            UpdateProjectCommand {
                project_id: created.id,
                name: Some("Nope".to_string()),
                description: None,
                location: None,
                client_company_id: None,
                planned_start: None,
                planned_end: None,
            },
        )
        .await;
    assert!(update_archived.is_err());
}

#[tokio::test]
async fn duplicate_code_rejected() {
    let module = ProjectsModule::in_memory();
    let tenant_id = TenantId::new();
    let prime = CompanyId::new();
    let cmd = || CreateProjectCommand {
        code: "DUP-1".to_string(),
        name: "One".to_string(),
        description: None,
        location: None,
        prime_contractor_company_id: prime,
        client_company_id: None,
        planned_start: None,
        planned_end: None,
    };
    module
        .services
        .create_project(ctx(tenant_id), cmd())
        .await
        .expect("first create");
    let second = module.services.create_project(ctx(tenant_id), cmd()).await;
    assert!(second.is_err());
}

#[tokio::test]
async fn membership_orchestrates_core() {
    use proven_core::{AuthzApi, MembershipApi, TenancyApi};
    use proven_projects::application::services::AllowAllAuthz;
    use proven_projects::{ProjectsModule, ProjectsPorts};
    use std::sync::Arc;

    let core = CoreModule::in_memory();
    // Real Membership + Tenancy, stub AuthZ — focuses the test on orchestration invariants
    // (license gating of projects.* is covered by Core AuthZ tests).
    let authz: Arc<dyn AuthzApi> = Arc::new(AllowAllAuthz);
    let membership: Arc<dyn MembershipApi> = core.services.clone();
    let tenancy: Arc<dyn TenancyApi> = core.services.clone();
    let projects = ProjectsModule::from_ports(
        ProjectsPorts::in_memory(),
        authz,
        Some(membership),
        Some(tenancy),
    );

    let provisioned = core
        .services
        .provision_tenant(ProvisionTenantCommand {
            slug: format!("acme-{}", uuid::Uuid::new_v4()),
            display_name: "Acme Construction".into(),
            region_code: RegionCode::new("CA"),
            owner_company_name: "Acme GC Ltd".into(),
            owner_company_type: CompanyType::Prime,
            admin_email: format!("admin-{}@acme.test", uuid::Uuid::new_v4()),
            admin_display_name: "Acme Admin".into(),
            seats_limit: 25,
        })
        .await
        .expect("tenant");

    let company = core
        .services
        .register_company(RegisterCompanyCommand {
            tenant_id: provisioned.tenant.id,
            legal_name: "Acme Prime Ltd".into(),
            display_name: "Acme Prime".into(),
            company_type: CompanyType::Prime,
        })
        .await
        .expect("company");

    let user = core
        .services
        .invite_user(InviteUserCommand {
            tenant_id: provisioned.tenant.id,
            email: format!("worker-{}@example.com", uuid::Uuid::new_v4()),
            display_name: "Worker One".into(),
            invited_by: Some(provisioned.admin_user.id),
        })
        .await
        .expect("user");

    let acting = ActingContext::new(
        provisioned.tenant.id,
        PrincipalId::from_uuid(provisioned.admin_user.id.as_uuid()),
    );

    let project = projects
        .services
        .create_project(
            acting,
            CreateProjectCommand {
                code: "MEM-1".to_string(),
                name: "Membership Site".to_string(),
                description: None,
                location: None,
                prime_contractor_company_id: company.id,
                client_company_id: None,
                planned_start: None,
                planned_end: None,
            },
        )
        .await
        .expect("create project with Core companies check");

    let membership = projects
        .services
        .assign_membership(
            acting,
            AssignProjectMembershipCommand {
                project_id: project.id,
                user_id: user.id,
                membership_role: "worker".to_string(),
                granted_by: Some(provisioned.admin_user.id),
            },
        )
        .await
        .expect("assign membership via Core");

    assert_eq!(membership.project_id, project.id);
    assert_eq!(membership.user_id, Some(user.id));

    let is_member = projects
        .services
        .is_member(
            provisioned.tenant.id,
            project.id,
            PrincipalId::from_uuid(user.id.as_uuid()),
        )
        .await
        .expect("is_member");
    assert!(is_member);

    let mine = projects
        .services
        .list_principal_projects(
            provisioned.tenant.id,
            PrincipalId::from_uuid(user.id.as_uuid()),
        )
        .await
        .expect("list mine");
    assert_eq!(mine, vec![project.id]);

    let via_core = core
        .services
        .is_project_member(
            provisioned.tenant.id,
            project.id,
            PrincipalId::from_uuid(user.id.as_uuid()),
        )
        .await
        .expect("core is_member");
    assert!(via_core);

    projects
        .services
        .archive_project(acting, project.id)
        .await
        .expect("archive");
    let blocked = projects
        .services
        .assign_membership(
            acting,
            AssignProjectMembershipCommand {
                project_id: project.id,
                user_id: UserId::new(),
                membership_role: "worker".to_string(),
                granted_by: Some(provisioned.admin_user.id),
            },
        )
        .await;
    assert!(blocked.is_err());
}

#[tokio::test]
async fn membership_requires_core_wiring() {
    let module = ProjectsModule::in_memory();
    let tenant_id = TenantId::new();
    let project = module
        .services
        .create_project(
            ctx(tenant_id),
            CreateProjectCommand {
                code: "NO-CORE".to_string(),
                name: "No Core".to_string(),
                description: None,
                location: None,
                prime_contractor_company_id: CompanyId::new(),
                client_company_id: None,
                planned_start: None,
                planned_end: None,
            },
        )
        .await
        .expect("create");

    let err = module
        .services
        .assign_membership(
            ctx(tenant_id),
            AssignProjectMembershipCommand {
                project_id: project.id,
                user_id: UserId::new(),
                membership_role: "worker".to_string(),
                granted_by: None,
            },
        )
        .await;
    assert!(err.is_err());
}
