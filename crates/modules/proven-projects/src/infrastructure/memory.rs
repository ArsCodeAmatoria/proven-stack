//! In-memory store for Projects ports (authoritative until Postgres adapter lands).

use std::collections::HashMap;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use async_trait::async_trait;
use proven_shared::{ProjectId, TenantId};
use uuid::Uuid;

use crate::application::ports::{ParticipantRepository, ProjectRepository};
use crate::domain::{Project, ProjectParticipant, ProjectStatus, ProjectsError};

#[derive(Default)]
struct MemoryState {
    projects: HashMap<Uuid, Project>,
    participants: HashMap<Uuid, ProjectParticipant>,
}

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

    fn read(&self) -> Result<RwLockReadGuard<'_, MemoryState>, ProjectsError> {
        self.state
            .read()
            .map_err(|_| ProjectsError::Internal("memory store lock poisoned".into()))
    }

    fn write(&self) -> Result<RwLockWriteGuard<'_, MemoryState>, ProjectsError> {
        self.state
            .write()
            .map_err(|_| ProjectsError::Internal("memory store lock poisoned".into()))
    }
}

#[async_trait]
impl ProjectRepository for MemoryStore {
    async fn insert(&self, project: &Project) -> Result<(), ProjectsError> {
        self.write()?
            .projects
            .insert(project.id.as_uuid(), project.clone());
        Ok(())
    }

    async fn get(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> Result<Option<Project>, ProjectsError> {
        Ok(self
            .read()?
            .projects
            .get(&project_id.as_uuid())
            .filter(|p| p.tenant_id == tenant_id)
            .cloned())
    }

    async fn get_by_code(
        &self,
        tenant_id: TenantId,
        code: &str,
    ) -> Result<Option<Project>, ProjectsError> {
        let needle = code.trim().to_ascii_lowercase();
        Ok(self
            .read()?
            .projects
            .values()
            .find(|p| {
                p.tenant_id == tenant_id && p.code.to_ascii_lowercase() == needle
            })
            .cloned())
    }

    async fn update(&self, project: &Project) -> Result<(), ProjectsError> {
        let mut state = self.write()?;
        if !state.projects.contains_key(&project.id.as_uuid()) {
            return Err(ProjectsError::not_found("project"));
        }
        state
            .projects
            .insert(project.id.as_uuid(), project.clone());
        Ok(())
    }

    async fn list(
        &self,
        tenant_id: TenantId,
        include_archived: bool,
    ) -> Result<Vec<Project>, ProjectsError> {
        let mut items: Vec<_> = self
            .read()?
            .projects
            .values()
            .filter(|p| p.tenant_id == tenant_id)
            .filter(|p| include_archived || !matches!(p.status, ProjectStatus::Archived))
            .cloned()
            .collect();
        items.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(items)
    }
}

#[async_trait]
impl ParticipantRepository for MemoryStore {
    async fn insert(&self, participant: &ProjectParticipant) -> Result<(), ProjectsError> {
        self.write()?
            .participants
            .insert(participant.id.as_uuid(), participant.clone());
        Ok(())
    }

    async fn list_for_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<ProjectParticipant>, ProjectsError> {
        Ok(self
            .read()?
            .participants
            .values()
            .filter(|p| p.project_id == project_id)
            .cloned()
            .collect())
    }
}
