use crate::domain::user_phone_number::UserPhoneNumber;
use crate::error::Result;
use crate::ports::transaction_manager::Transaction;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait UserPhoneNumberRepository: Send + Sync {
    /// All phone numbers for a user, primary first.
    async fn find_by_user_id(&self, user_id: &Uuid) -> Result<Vec<UserPhoneNumber>>;

    /// Find the phone-number row matching a normalised number within a realm.
    async fn find_by_phone_number(
        &self,
        realm_id: &Uuid,
        phone_number_normalized: &str,
    ) -> Result<Option<UserPhoneNumber>>;

    /// The single primary phone number for a user.
    async fn find_primary(&self, user_id: &Uuid) -> Result<Option<UserPhoneNumber>>;

    async fn save(
        &self,
        phone_number: &UserPhoneNumber,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;

    /// Atomically mark one row as primary (the trigger demotes any previous primary).
    async fn set_primary(
        &self,
        user_id: &Uuid,
        phone_number_id: &Uuid,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;

    async fn set_verified(
        &self,
        phone_number_id: &Uuid,
        is_verified: bool,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;

    async fn delete(&self, phone_number_id: &Uuid, tx: Option<&mut dyn Transaction>) -> Result<()>;
}
