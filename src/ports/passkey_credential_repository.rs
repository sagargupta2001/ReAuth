use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::passkey_credential::PasskeyCredential;
use crate::error::Result;

#[async_trait]
pub trait PasskeyCredentialRepository: Send + Sync {
    async fn create(&self, credential: &PasskeyCredential) -> Result<()>;
    async fn find_by_realm_and_credential_id(
        &self,
        realm_id: &Uuid,
        credential_id_b64url: &str,
    ) -> Result<Option<PasskeyCredential>>;
    async fn list_by_user(&self, realm_id: &Uuid, user_id: &Uuid)
        -> Result<Vec<PasskeyCredential>>;
    async fn touch_assertion_state(
        &self,
        credential_id: &Uuid,
        observed_sign_count: i64,
        backed_up: bool,
        last_used_at: DateTime<Utc>,
    ) -> Result<bool>;
    async fn count_by_realm(&self, realm_id: &Uuid) -> Result<u64>;
    async fn count_created_since(&self, realm_id: &Uuid, since: DateTime<Utc>) -> Result<u64>;
    async fn count_active_since(&self, realm_id: &Uuid, since: DateTime<Utc>) -> Result<u64>;
    async fn delete_by_id_for_user(
        &self,
        realm_id: &Uuid,
        user_id: &Uuid,
        credential_id: &Uuid,
    ) -> Result<bool>;
    async fn update_friendly_name_for_user(
        &self,
        realm_id: &Uuid,
        user_id: &Uuid,
        credential_id: &Uuid,
        friendly_name: Option<String>,
    ) -> Result<bool>;
}
