use crate::adapters::persistence::connection::Database;
use crate::adapters::persistence::transaction::SqliteTransaction;
use crate::domain::user_email::UserEmail;
use crate::error::{Error, Result};
use crate::ports::transaction_manager::Transaction;
use crate::ports::user_email_repository::UserEmailRepository;
use async_trait::async_trait;
use tracing::instrument;
use uuid::Uuid;

pub struct SqliteUserEmailRepository {
    pool: Database,
}

impl SqliteUserEmailRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserEmailRepository for SqliteUserEmailRepository {
    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "user_emails", db_op = "select")
    )]
    async fn find_by_user_id(&self, user_id: &Uuid) -> Result<Vec<UserEmail>> {
        let rows = sqlx::query_as(
            "SELECT * FROM user_emails WHERE user_id = ? ORDER BY is_primary DESC, created_at ASC",
        )
        .bind(user_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(rows)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "user_emails", db_op = "select")
    )]
    async fn find_by_email(
        &self,
        realm_id: &Uuid,
        email_normalized: &str,
    ) -> Result<Option<UserEmail>> {
        let row =
            sqlx::query_as("SELECT * FROM user_emails WHERE realm_id = ? AND email_normalized = ?")
                .bind(realm_id.to_string())
                .bind(email_normalized)
                .fetch_optional(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(row)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "user_emails", db_op = "select")
    )]
    async fn find_primary(&self, user_id: &Uuid) -> Result<Option<UserEmail>> {
        let row = sqlx::query_as(
            "SELECT * FROM user_emails WHERE user_id = ? AND is_primary = 1 LIMIT 1",
        )
        .bind(user_id.to_string())
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(row)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "user_emails", db_op = "insert")
    )]
    async fn save(&self, email: &UserEmail, tx: Option<&mut dyn Transaction>) -> Result<()> {
        let query = sqlx::query(
            "INSERT INTO user_emails (id, user_id, realm_id, email, email_normalized, is_primary, is_verified, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(email.id.to_string())
        .bind(email.user_id.to_string())
        .bind(email.realm_id.to_string())
        .bind(&email.email)
        .bind(&email.email_normalized)
        .bind(email.is_primary)
        .bind(email.is_verified)
        .bind(email.created_at)
        .bind(email.updated_at);

        match tx {
            Some(tx) => {
                let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX type");
                query
                    .execute(&mut **sql_tx)
                    .await
                    .map_err(|e| Error::Unexpected(e.into()))?;
            }
            None => {
                query
                    .execute(&*self.pool)
                    .await
                    .map_err(|e| Error::Unexpected(e.into()))?;
            }
        }
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "user_emails", db_op = "update")
    )]
    async fn set_primary(
        &self,
        user_id: &Uuid,
        email_id: &Uuid,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        // The DB trigger handles demoting the old primary automatically.
        let query = sqlx::query(
            "UPDATE user_emails SET is_primary = 1, updated_at = CURRENT_TIMESTAMP
             WHERE id = ? AND user_id = ?",
        )
        .bind(email_id.to_string())
        .bind(user_id.to_string());

        match tx {
            Some(tx) => {
                let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX type");
                query
                    .execute(&mut **sql_tx)
                    .await
                    .map_err(|e| Error::Unexpected(e.into()))?;
            }
            None => {
                query
                    .execute(&*self.pool)
                    .await
                    .map_err(|e| Error::Unexpected(e.into()))?;
            }
        }
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "user_emails", db_op = "update")
    )]
    async fn set_verified(
        &self,
        email_id: &Uuid,
        is_verified: bool,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = sqlx::query(
            "UPDATE user_emails SET is_verified = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        )
        .bind(is_verified)
        .bind(email_id.to_string());

        match tx {
            Some(tx) => {
                let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX type");
                query
                    .execute(&mut **sql_tx)
                    .await
                    .map_err(|e| Error::Unexpected(e.into()))?;
            }
            None => {
                query
                    .execute(&*self.pool)
                    .await
                    .map_err(|e| Error::Unexpected(e.into()))?;
            }
        }
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "user_emails", db_op = "delete")
    )]
    async fn delete(&self, email_id: &Uuid, tx: Option<&mut dyn Transaction>) -> Result<()> {
        let query = sqlx::query("DELETE FROM user_emails WHERE id = ?").bind(email_id.to_string());

        match tx {
            Some(tx) => {
                let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX type");
                query
                    .execute(&mut **sql_tx)
                    .await
                    .map_err(|e| Error::Unexpected(e.into()))?;
            }
            None => {
                query
                    .execute(&*self.pool)
                    .await
                    .map_err(|e| Error::Unexpected(e.into()))?;
            }
        }
        Ok(())
    }
}
