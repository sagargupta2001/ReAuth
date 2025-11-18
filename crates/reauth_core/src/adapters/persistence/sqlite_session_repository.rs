use crate::adapters::persistence::connection::Database;
use crate::{
    domain::session::RefreshToken,
    error::{Error, Result},
    ports::session_repository::SessionRepository,
};
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

pub struct SqliteSessionRepository {
    pool: Database,
}
impl SqliteSessionRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepository for SqliteSessionRepository {
    async fn save(&self, token: &RefreshToken) -> Result<()> {
        sqlx::query(
            "INSERT INTO refresh_tokens (id, user_id, realm_id, expires_at) VALUES (?, ?, ?, ?)",
        )
        .bind(token.id.to_string())
        .bind(token.user_id.to_string())
        .bind(token.realm_id.to_string())
        .bind(token.expires_at)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<RefreshToken>> {
        Ok(
            sqlx::query_as("SELECT * FROM refresh_tokens WHERE id = ? AND expires_at > ?")
                .bind(id.to_string())
                .bind(Utc::now()) // Automatically check for expiry
                .fetch_optional(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?,
        )
    }
    async fn delete_by_id(&self, id: &Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM refresh_tokens WHERE id = ?")
            .bind(id.to_string())
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        if result.rows_affected() == 0 {
            // We map this to InvalidRefreshToken so the service knows to stop.
            return Err(Error::InvalidRefreshToken);
        }
        Ok(())
    }
}
