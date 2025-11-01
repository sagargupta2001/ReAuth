use crate::{
    domain::{group::Group, role::Role},
    error::{Error, Result},
    ports::rbac_repository::RbacRepository,
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct CreateRolePayload {
    pub name: String,
    pub description: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct CreateGroupPayload {
    pub name: String,
    pub description: Option<String>,
}

pub struct RbacService {
    rbac_repo: Arc<dyn RbacRepository>,
}

impl RbacService {
    pub fn new(rbac_repo: Arc<dyn RbacRepository>) -> Self {
        Self { rbac_repo }
    }

    pub async fn create_role(&self, payload: CreateRolePayload) -> Result<Role> {
        if self.rbac_repo.find_role_by_name(&payload.name).await?.is_some() {
            return Err(Error::RoleAlreadyExists);
        }
        let role = Role {
            id: Uuid::new_v4(),
            name: payload.name,
            description: payload.description,
        };
        self.rbac_repo.create_role(&role).await?;
        Ok(role)
    }

    pub async fn create_group(&self, payload: CreateGroupPayload) -> Result<Group> {
        if self.rbac_repo.find_group_by_name(&payload.name).await?.is_some() {
            return Err(Error::GroupAlreadyExists); 
        }
        let group = Group {
            id: Uuid::new_v4(),
            name: payload.name,
            description: payload.description,
        };
        self.rbac_repo.create_group(&group).await?;
        Ok(group)
    }
}