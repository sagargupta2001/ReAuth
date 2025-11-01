use crate::domain::{group::Group, role::Role};
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait RbacRepository: Send + Sync {
    async fn create_role(&self, role: &Role) -> Result<()>;
    async fn find_role_by_name(&self, name: &str) -> Result<Option<Role>>;

    async fn create_group(&self, group: &Group) -> Result<()>;
    async fn find_group_by_name(&self, name: &str) -> Result<Option<Group>>;

}