use crate::domain::events::EventEnvelope;
use crate::error::Result;
use crate::ports::transaction_manager::Transaction;
use async_trait::async_trait;

#[async_trait]
pub trait OutboxRepository: Send + Sync {
    async fn insert(
        &self,
        envelope: &EventEnvelope,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
}
