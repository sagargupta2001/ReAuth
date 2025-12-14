use crate::adapters::persistence::connection::Database;
use crate::error::{Error, Result};
use crate::ports::transaction_manager::{Transaction, TransactionManager};
use async_trait::async_trait;
use sqlx::{Sqlite, Transaction as SqlxTx};
use std::any::Any;

pub struct SqliteTransaction {
    pub inner: SqlxTx<'static, Sqlite>,
}

impl Transaction for SqliteTransaction {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl SqliteTransaction {
    pub fn from_trait(tx: &mut dyn Transaction) -> Option<&mut SqlxTx<'static, Sqlite>> {
        tx.as_any()
            .downcast_mut::<SqliteTransaction>()
            .map(|t| &mut t.inner)
    }
}

pub struct SqliteTransactionManager {
    pool: Database,
}

impl SqliteTransactionManager {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TransactionManager for SqliteTransactionManager {
    async fn begin(&self) -> Result<Box<dyn Transaction>> {
        let tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(Box::new(SqliteTransaction { inner: tx }))
    }

    async fn commit(&self, tx: Box<dyn Transaction>) -> Result<()> {
        // Safe Downcast
        let sqlite_tx = tx
            .into_any()
            .downcast::<SqliteTransaction>()
            .map_err(|_| Error::Unexpected(anyhow::anyhow!("Transaction type mismatch")))?;

        sqlite_tx
            .inner
            .commit()
            .await
            .map_err(|e| Error::Unexpected(e.into()))
    }

    async fn rollback(&self, tx: Box<dyn Transaction>) -> Result<()> {
        let sqlite_tx = tx
            .into_any()
            .downcast::<SqliteTransaction>()
            .map_err(|_| Error::Unexpected(anyhow::anyhow!("Transaction type mismatch")))?;

        sqlite_tx
            .inner
            .rollback()
            .await
            .map_err(|e| Error::Unexpected(e.into()))
    }
}
