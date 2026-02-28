use crate::domain::pagination::{PageRequest, PageResponse};
use crate::domain::user::User;
use crate::error::Result;
use crate::ports::transaction_manager::Transaction;
use async_trait::async_trait;
use uuid::Uuid;

/// A "Port" defining the contract for user persistence operations.
///
/// The application layer uses this interface to talk to the database,
/// without knowing the specific database technology being used.
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_username(&self, realm_id: &Uuid, username: &str) -> Result<Option<User>>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>>;
    async fn save(&self, user: &User, tx: Option<&mut dyn Transaction>) -> Result<()>;
    async fn update(&self, user: &User, tx: Option<&mut dyn Transaction>) -> Result<()>;
    async fn list(&self, realm_id: &Uuid, req: &PageRequest) -> Result<PageResponse<User>>;
}
