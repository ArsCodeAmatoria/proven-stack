//! `TemplatesService` — default document template pointers, one per `TemplateKind`
//! (ADR-0005 §3). The template artifact itself is owned elsewhere (`domain::ownership`); this
//! service only stores/retrieves the pointer (`template_ref`), never dereferencing it.

use std::sync::Arc;

use chrono::Utc;
use proven_core::AuthzApi;
use proven_shared::CompanyId;

use crate::application::ports::{DefaultTemplateRepository, EventPublisher};
use crate::application::services::authz::{authorize, ActingContext};
use crate::domain::validation::validate_non_empty;
use crate::domain::{permissions, CompaniesError, DefaultTemplate, DefaultTemplateId, TemplateKind};
use crate::events::{ActorRef, CompaniesEvent, EventEnvelope, ResourceRef};

pub struct UpsertDefaultTemplateCommand {
    pub company_id: CompanyId,
    pub kind: TemplateKind,
    pub template_ref: String,
    pub label: Option<String>,
}

pub struct TemplatesService {
    templates: Arc<dyn DefaultTemplateRepository>,
    outbox: Arc<dyn EventPublisher>,
    authz: Arc<dyn AuthzApi>,
}

impl TemplatesService {
    pub fn new(
        templates: Arc<dyn DefaultTemplateRepository>,
        outbox: Arc<dyn EventPublisher>,
        authz: Arc<dyn AuthzApi>,
    ) -> Self {
        Self {
            templates,
            outbox,
            authz,
        }
    }

    pub async fn list(&self, company_id: CompanyId) -> Result<Vec<DefaultTemplate>, CompaniesError> {
        self.templates.list(company_id).await
    }

    /// Upserts the default template pointer for a single `TemplateKind` (PUT semantics: one
    /// kind per call, matching the HTTP surface and the `default_templates_kind_uidx` partial
    /// unique index in `db/migrations/companies`).
    pub async fn upsert(
        &self,
        ctx: &ActingContext,
        cmd: UpsertDefaultTemplateCommand,
    ) -> Result<DefaultTemplate, CompaniesError> {
        authorize(
            self.authz.as_ref(),
            ctx,
            permissions::TEMPLATES_MANAGE,
            cmd.company_id,
        )
        .await?;
        validate_non_empty("template_ref", &cmd.template_ref)?;

        let now = Utc::now();
        let existing = self.templates.get_by_kind(cmd.company_id, cmd.kind).await?;
        let template = DefaultTemplate {
            id: existing.as_ref().map(|t| t.id).unwrap_or_else(DefaultTemplateId::new),
            company_id: cmd.company_id,
            tenant_id: ctx.tenant_id,
            kind: cmd.kind,
            template_ref: cmd.template_ref,
            label: cmd.label,
            is_default: true,
            created_at: existing.map(|t| t.created_at).unwrap_or(now),
            updated_at: now,
        };
        self.templates.upsert(&template).await?;

        self.outbox
            .publish(EventEnvelope::new(
                ctx.tenant_id,
                ActorRef::Principal {
                    principal_id: ctx.principal,
                },
                ResourceRef {
                    resource_type: "default_template".to_string(),
                    resource_id: template.id.as_uuid(),
                },
                None,
                None,
                CompaniesEvent::DefaultTemplateUpserted {
                    tenant_id: ctx.tenant_id,
                    company_id: cmd.company_id,
                    template_id: template.id,
                    kind: cmd.kind,
                },
            ))
            .await?;

        Ok(template)
    }
}
