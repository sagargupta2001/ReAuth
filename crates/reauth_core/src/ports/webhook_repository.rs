use crate::domain::webhook::{WebhookEndpoint, WebhookSubscription};
use crate::error::Result;
use crate::ports::transaction_manager::Transaction;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait WebhookRepository: Send + Sync {
    async fn create_endpoint(
        &self,
        endpoint: &WebhookEndpoint,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn update_endpoint(
        &self,
        endpoint: &WebhookEndpoint,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn delete_endpoint(
        &self,
        realm_id: &Uuid,
        endpoint_id: &Uuid,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn set_endpoint_status(
        &self,
        realm_id: &Uuid,
        endpoint_id: &Uuid,
        status: &str,
        reason: Option<&str>,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn find_endpoint(
        &self,
        realm_id: &Uuid,
        endpoint_id: &Uuid,
    ) -> Result<Option<WebhookEndpoint>>;
    async fn list_endpoints(&self, realm_id: &Uuid) -> Result<Vec<WebhookEndpoint>>;
    async fn upsert_subscriptions(
        &self,
        endpoint_id: &Uuid,
        event_types: &[String],
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn set_subscription_enabled(
        &self,
        endpoint_id: &Uuid,
        event_type: &str,
        enabled: bool,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn list_subscriptions(&self, endpoint_id: &Uuid) -> Result<Vec<WebhookSubscription>>;
}
