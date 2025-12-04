use crate::adapters::persistence::connection::Database;
use crate::domain::pagination::{PageRequest, PageResponse};
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
            "INSERT INTO refresh_tokens
            (id, user_id, realm_id, client_id, expires_at, ip_address, user_agent, created_at, last_used_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
            .bind(token.id.to_string())
            .bind(token.user_id.to_string())
            .bind(token.realm_id.to_string())
            .bind(&token.client_id)
            .bind(token.expires_at)
            .bind(&token.ip_address)
            .bind(&token.user_agent)
            .bind(token.created_at)
            .bind(token.last_used_at)
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

    async fn list(&self, realm_id: &Uuid, req: &PageRequest) -> Result<PageResponse<RefreshToken>> {
        let limit = req.per_page.clamp(1, 100);
        let offset = (req.page - 1) * limit;

        /* -------------------------
           1. COUNT QUERY
        ------------------------- */

        let mut count_builder =
            sqlx::QueryBuilder::new("SELECT COUNT(*) FROM refresh_tokens WHERE realm_id = ");
        count_builder.push_bind(realm_id.to_string());

        // match user repo behavior — simple search on user_id
        if let Some(q) = &req.q {
            if !q.is_empty() {
                count_builder.push(" AND user_id LIKE ");
                count_builder.push_bind(format!("%{}%", q));
            }
        }

        let total: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        /* -------------------------
           2. SELECT QUERY
        ------------------------- */

        let mut query_builder =
            sqlx::QueryBuilder::new("SELECT * FROM refresh_tokens WHERE realm_id = ");
        query_builder.push_bind(realm_id.to_string());

        if let Some(q) = &req.q {
            if !q.is_empty() {
                query_builder.push(" AND user_id LIKE ");
                query_builder.push_bind(format!("%{}%", q));
            }
        }

        query_builder.push(" ORDER BY created_at DESC");

        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);

        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        let sessions: Vec<RefreshToken> = query_builder
            .build_query_as()
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        /* -------------------------
           3. Return paginated result
        ------------------------- */

        Ok(PageResponse::new(sessions, total, req.page, limit))
    }
}
