use crate::domain::user_email::UserEmail;
use crate::error::Result;
use crate::ports::transaction_manager::Transaction;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait UserEmailRepository: Send + Sync {
    /// All emails for a user, primary first.
    async fn find_by_user_id(&self, user_id: &Uuid) -> Result<Vec<UserEmail>>;

    /// Find the user_email row matching a normalised address within a realm.
    async fn find_by_email(
        &self,
        realm_id: &Uuid,
        email_normalized: &str,
    ) -> Result<Option<UserEmail>>;

    /// The single primary email for a user.
    async fn find_primary(&self, user_id: &Uuid) -> Result<Option<UserEmail>>;

    async fn save(&self, email: &UserEmail, tx: Option<&mut dyn Transaction>) -> Result<()>;

    /// Atomically mark one row as primary (the trigger demotes any previous primary).
    async fn set_primary(
        &self,
        user_id: &Uuid,
        email_id: &Uuid,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;

    async fn set_verified(
        &self,
        email_id: &Uuid,
        is_verified: bool,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;

    async fn delete(&self, email_id: &Uuid, tx: Option<&mut dyn Transaction>) -> Result<()>;
}
