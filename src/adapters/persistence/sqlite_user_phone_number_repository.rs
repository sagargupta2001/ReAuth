use crate::adapters::persistence::connection::Database;
use crate::adapters::persistence::transaction::SqliteTransaction;
use crate::domain::user_phone_number::UserPhoneNumber;
use crate::error::{Error, Result};
use crate::ports::transaction_manager::Transaction;
use crate::ports::user_phone_number_repository::UserPhoneNumberRepository;
use async_trait::async_trait;
use tracing::instrument;
use uuid::Uuid;

pub struct SqliteUserPhoneNumberRepository {
    pool: Database,
}

impl SqliteUserPhoneNumberRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserPhoneNumberRepository for SqliteUserPhoneNumberRepository {
    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "user_phone_numbers", db_op = "select")
    )]
    async fn find_by_user_id(&self, user_id: &Uuid) -> Result<Vec<UserPhoneNumber>> {
        let rows = sqlx::query_as(
            "SELECT * FROM user_phone_numbers WHERE user_id = ? ORDER BY is_primary DESC, created_at ASC",
        )
        .bind(user_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(rows)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "user_phone_numbers", db_op = "select")
    )]
    async fn find_by_phone_number(
        &self,
        realm_id: &Uuid,
        phone_number_normalized: &str,
    ) -> Result<Option<UserPhoneNumber>> {
        let row = sqlx::query_as(
            "SELECT * FROM user_phone_numbers WHERE realm_id = ? AND phone_number_normalized = ?",
        )
        .bind(realm_id.to_string())
        .bind(phone_number_normalized)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(row)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "user_phone_numbers", db_op = "select")
    )]
    async fn find_primary(&self, user_id: &Uuid) -> Result<Option<UserPhoneNumber>> {
        let row = sqlx::query_as(
            "SELECT * FROM user_phone_numbers WHERE user_id = ? AND is_primary = 1 LIMIT 1",
        )
        .bind(user_id.to_string())
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(row)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "user_phone_numbers", db_op = "insert")
    )]
    async fn save(
        &self,
        phone_number: &UserPhoneNumber,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = sqlx::query(
            "INSERT INTO user_phone_numbers (
                id, user_id, realm_id, phone_number, phone_number_normalized,
                is_primary, is_verified, created_at, updated_at
            )
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(phone_number.id.to_string())
        .bind(phone_number.user_id.to_string())
        .bind(phone_number.realm_id.to_string())
        .bind(&phone_number.phone_number)
        .bind(&phone_number.phone_number_normalized)
        .bind(phone_number.is_primary)
        .bind(phone_number.is_verified)
        .bind(phone_number.created_at)
        .bind(phone_number.updated_at);

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
        fields(telemetry = "span", db_table = "user_phone_numbers", db_op = "update")
    )]
    async fn set_primary(
        &self,
        user_id: &Uuid,
        phone_number_id: &Uuid,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = sqlx::query(
            "UPDATE user_phone_numbers SET is_primary = 1, updated_at = CURRENT_TIMESTAMP
             WHERE id = ? AND user_id = ?",
        )
        .bind(phone_number_id.to_string())
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
        fields(telemetry = "span", db_table = "user_phone_numbers", db_op = "update")
    )]
    async fn set_verified(
        &self,
        phone_number_id: &Uuid,
        is_verified: bool,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = sqlx::query(
            "UPDATE user_phone_numbers
             SET is_verified = ?, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
        )
        .bind(is_verified)
        .bind(phone_number_id.to_string());

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
        fields(telemetry = "span", db_table = "user_phone_numbers", db_op = "delete")
    )]
    async fn delete(&self, phone_number_id: &Uuid, tx: Option<&mut dyn Transaction>) -> Result<()> {
        let query = sqlx::query("DELETE FROM user_phone_numbers WHERE id = ?")
            .bind(phone_number_id.to_string());

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
