use crate::domain::pagination::{PageRequest, PageResponse};
use crate::domain::user::User;
use crate::error::Result;
use async_trait::async_trait;
use uuid::Uuid;

/// A "Port" defining the contract for user persistence operations.
///
/// The application layer uses this interface to talk to the database,
/// without knowing the specific database technology being used.
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>>;
    async fn save(&self, user: &User) -> Result<()>;
    async fn update(&self, user: &User) -> Result<()>;
    async fn list(&self, req: &PageRequest) -> Result<PageResponse<User>>;
}
