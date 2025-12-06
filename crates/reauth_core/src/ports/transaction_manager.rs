use crate::error::Result;
use async_trait::async_trait;
use std::any::Any;

pub trait Transaction: Send + Any {
    fn as_any(&mut self) -> &mut dyn Any;
    // Add this to allow consuming the Box for commit/rollback
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

#[async_trait]
pub trait TransactionManager: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn Transaction>>;
    async fn commit(&self, tx: Box<dyn Transaction>) -> Result<()>;
    async fn rollback(&self, tx: Box<dyn Transaction>) -> Result<()>;
}
