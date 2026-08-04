//! `BrandingService` — company visual branding (ADR-0005 §3).

use std::sync::Arc;

use chrono::Utc;
use proven_core::AuthzApi;
use proven_shared::{CompanyId, FileObjectId};

use crate::application::ports::{BrandingRepository, EventPublisher};
use crate::application::services::authz::{authorize, ActingContext};
use crate::domain::validation::validate_hex_color;
use crate::domain::{permissions, CompaniesError, CompanyBranding};
use crate::events::{ActorRef, CompaniesEvent, EventEnvelope, ResourceRef};

pub struct UpsertBrandingCommand {
    pub company_id: CompanyId,
    pub logo_file_id: Option<FileObjectId>,
    pub wordmark_file_id: Option<FileObjectId>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub accent_color: Option<String>,
    pub favicon_file_id: Option<FileObjectId>,
}

pub struct BrandingService {
    branding: Arc<dyn BrandingRepository>,
    outbox: Arc<dyn EventPublisher>,
    authz: Arc<dyn AuthzApi>,
}

impl BrandingService {
    pub fn new(
        branding: Arc<dyn BrandingRepository>,
        outbox: Arc<dyn EventPublisher>,
        authz: Arc<dyn AuthzApi>,
    ) -> Self {
        Self {
            branding,
            outbox,
            authz,
        }
    }

    pub async fn get(&self, company_id: CompanyId) -> Result<CompanyBranding, CompaniesError> {
        self.branding
            .get(company_id)
            .await?
            .ok_or(CompaniesError::NotFound("company_branding"))
    }

    pub async fn upsert(
        &self,
        ctx: &ActingContext,
        cmd: UpsertBrandingCommand,
    ) -> Result<CompanyBranding, CompaniesError> {
        authorize(
            self.authz.as_ref(),
            ctx,
            permissions::BRANDING_MANAGE,
            cmd.company_id,
        )
        .await?;
        if let Some(color) = &cmd.primary_color {
            validate_hex_color(color)?;
        }
        if let Some(color) = &cmd.secondary_color {
            validate_hex_color(color)?;
        }
        if let Some(color) = &cmd.accent_color {
            validate_hex_color(color)?;
        }

        let now = Utc::now();
        let mut branding = self
            .branding
            .get(cmd.company_id)
            .await?
            .unwrap_or_else(|| CompanyBranding::defaults(cmd.company_id, ctx.tenant_id, now));

        if cmd.logo_file_id.is_some() {
            branding.logo_file_id = cmd.logo_file_id;
        }
        if cmd.wordmark_file_id.is_some() {
            branding.wordmark_file_id = cmd.wordmark_file_id;
        }
        if cmd.primary_color.is_some() {
            branding.primary_color = cmd.primary_color;
        }
        if cmd.secondary_color.is_some() {
            branding.secondary_color = cmd.secondary_color;
        }
        if cmd.accent_color.is_some() {
            branding.accent_color = cmd.accent_color;
        }
        if cmd.favicon_file_id.is_some() {
            branding.favicon_file_id = cmd.favicon_file_id;
        }
        branding.updated_at = now;
        self.branding.upsert(&branding).await?;

        self.outbox
            .publish(EventEnvelope::new(
                ctx.tenant_id,
                ActorRef::Principal {
                    principal_id: ctx.principal,
                },
                ResourceRef {
                    resource_type: "company_branding".to_string(),
                    resource_id: cmd.company_id.as_uuid(),
                },
                None,
                None,
                CompaniesEvent::BrandingUpdated {
                    tenant_id: ctx.tenant_id,
                    company_id: cmd.company_id,
                    logo_file_id: branding.logo_file_id,
                },
            ))
            .await?;

        Ok(branding)
    }
}
