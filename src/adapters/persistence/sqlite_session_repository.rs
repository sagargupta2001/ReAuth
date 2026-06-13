use crate::adapters::persistence::connection::Database;
use crate::domain::pagination::{PageRequest, PageResponse};
use crate::{
    domain::session::{RefreshToken, SessionListFilter},
    error::{Error, Result},
    ports::session_repository::SessionRepository,
};
use async_trait::async_trait;
use chrono::Utc;
use tracing::instrument;
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
    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "refresh_tokens", db_op = "insert")
    )]
    async fn save(&self, token: &RefreshToken) -> Result<()> {
        sqlx::query(
            "INSERT INTO refresh_tokens
            (id, family_id, user_id, realm_id, client_id, expires_at, ip_address, user_agent, created_at, last_used_at, revoked_at, replaced_by)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
            .bind(token.id.to_string())
            .bind(token.family_id.to_string())
            .bind(token.user_id.to_string())
            .bind(token.realm_id.to_string())
            .bind(&token.client_id)
            .bind(token.expires_at)
            .bind(&token.ip_address)
            .bind(&token.user_agent)
            .bind(token.created_at)
            .bind(token.last_used_at)
            .bind(token.revoked_at)
            .bind(token.replaced_by.map(|id| id.to_string()))
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "refresh_tokens", db_op = "select")
    )]
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<RefreshToken>> {
        Ok(
            sqlx::query_as(
                "SELECT * FROM refresh_tokens WHERE id = ? AND expires_at > ? AND revoked_at IS NULL AND replaced_by IS NULL",
            )
                .bind(id.to_string())
                .bind(Utc::now()) // Automatically check for expiry
                .fetch_optional(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?,
        )
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "refresh_tokens", db_op = "select")
    )]
    async fn find_by_id_any(&self, id: &Uuid) -> Result<Option<RefreshToken>> {
        Ok(sqlx::query_as("SELECT * FROM refresh_tokens WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "refresh_tokens", db_op = "delete")
    )]
    async fn delete_by_id(&self, id: &Uuid) -> Result<()> {
        let result = sqlx::query("UPDATE refresh_tokens SET revoked_at = ? WHERE id = ?")
            .bind(Utc::now())
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

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "refresh_tokens", db_op = "update")
    )]
    async fn mark_replaced(&self, old_id: &Uuid, new_id: &Uuid) -> Result<()> {
        let result =
            sqlx::query("UPDATE refresh_tokens SET replaced_by = ?, revoked_at = ? WHERE id = ?")
                .bind(new_id.to_string())
                .bind(Utc::now())
                .bind(old_id.to_string())
                .execute(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;

        if result.rows_affected() == 0 {
            return Err(Error::InvalidRefreshToken);
        }
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "refresh_tokens", db_op = "update")
    )]
    async fn revoke_family(&self, family_id: &Uuid) -> Result<()> {
        sqlx::query("UPDATE refresh_tokens SET revoked_at = ? WHERE family_id = ?")
            .bind(Utc::now())
            .bind(family_id.to_string())
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "refresh_tokens", db_op = "update")
    )]
    async fn revoke_all_for_user(&self, realm_id: &Uuid, user_id: &Uuid) -> Result<()> {
        sqlx::query("UPDATE refresh_tokens SET revoked_at = ? WHERE realm_id = ? AND user_id = ?")
            .bind(Utc::now())
            .bind(realm_id.to_string())
            .bind(user_id.to_string())
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "refresh_tokens", db_op = "update")
    )]
    async fn revoke_many(&self, realm_id: &Uuid, ids: &[Uuid]) -> Result<u64> {
        if ids.is_empty() {
            return Ok(0);
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        let now = Utc::now();
        let mut affected: u64 = 0;
        for id in ids {
            let result = sqlx::query(
                "UPDATE refresh_tokens SET revoked_at = ? WHERE id = ? AND realm_id = ? AND revoked_at IS NULL",
            )
            .bind(now)
            .bind(id.to_string())
            .bind(realm_id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
            affected += result.rows_affected();
        }

        tx.commit().await.map_err(|e| Error::Unexpected(e.into()))?;
        Ok(affected)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "refresh_tokens", db_op = "update")
    )]
    async fn revoke_others_for_user(
        &self,
        realm_id: &Uuid,
        user_id: &Uuid,
        except_id: &Uuid,
    ) -> Result<u64> {
        let result = sqlx::query(
            "UPDATE refresh_tokens SET revoked_at = ? WHERE realm_id = ? AND user_id = ? AND id != ? AND revoked_at IS NULL",
        )
        .bind(Utc::now())
        .bind(realm_id.to_string())
        .bind(user_id.to_string())
        .bind(except_id.to_string())
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(result.rows_affected())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "refresh_tokens", db_op = "update")
    )]
    async fn revoke_user_sessions(&self, realm_id: &Uuid, user_id: &Uuid) -> Result<u64> {
        let result = sqlx::query(
            "UPDATE refresh_tokens SET revoked_at = ? WHERE realm_id = ? AND user_id = ? AND revoked_at IS NULL",
        )
        .bind(Utc::now())
        .bind(realm_id.to_string())
        .bind(user_id.to_string())
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(result.rows_affected())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "refresh_tokens", db_op = "update")
    )]
    async fn request_step_up(&self, realm_id: &Uuid, id: &Uuid) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE refresh_tokens SET step_up_at = ? WHERE id = ? AND realm_id = ? AND revoked_at IS NULL AND replaced_by IS NULL",
        )
        .bind(Utc::now())
        .bind(id.to_string())
        .bind(realm_id.to_string())
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(result.rows_affected() > 0)
    }

    async fn revoke_by_user_and_client(
        &self,
        realm_id: &Uuid,
        user_id: &Uuid,
        client_id: &str,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE refresh_tokens SET revoked_at = ? WHERE realm_id = ? AND user_id = ? AND client_id = ? AND revoked_at IS NULL",
        )
        .bind(Utc::now())
        .bind(realm_id.to_string())
        .bind(user_id.to_string())
        .bind(client_id)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn revoke_root_tokens_for_user(&self, realm_id: &Uuid, user_id: &Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE refresh_tokens SET revoked_at = ? WHERE realm_id = ? AND user_id = ? AND client_id IS NULL AND revoked_at IS NULL",
        )
        .bind(Utc::now())
        .bind(realm_id.to_string())
        .bind(user_id.to_string())
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "refresh_tokens", db_op = "select")
    )]
    async fn list(
        &self,
        realm_id: &Uuid,
        req: &PageRequest,
        filter: &SessionListFilter,
    ) -> Result<PageResponse<RefreshToken>> {
        let limit = req.per_page.clamp(1, 100);
        let offset = (req.page - 1) * limit;

        /* -------------------------
           1. COUNT QUERY
        ------------------------- */

        let mut count_builder = sqlx::QueryBuilder::new(
            "SELECT COUNT(*) FROM refresh_tokens \
             LEFT JOIN users ON users.id = refresh_tokens.user_id \
             WHERE refresh_tokens.realm_id = ",
        );
        count_builder.push_bind(realm_id.to_string());
        count_builder.push(
            " AND refresh_tokens.revoked_at IS NULL AND refresh_tokens.replaced_by IS NULL \
             AND refresh_tokens.expires_at > ",
        );
        count_builder.push_bind(Utc::now());

        // Search matches the user id or the owning user's username.
        if let Some(q) = &req.q {
            if !q.is_empty() {
                let pattern = format!("%{}%", q);
                count_builder.push(" AND (refresh_tokens.user_id LIKE ");
                count_builder.push_bind(pattern.clone());
                count_builder.push(" OR users.username LIKE ");
                count_builder.push_bind(pattern);
                count_builder.push(")");
            }
        }

        if let Some(from) = filter.started_from {
            count_builder.push(" AND refresh_tokens.created_at >= ");
            count_builder.push_bind(from);
        }
        if let Some(to) = filter.started_to_exclusive {
            count_builder.push(" AND refresh_tokens.created_at < ");
            count_builder.push_bind(to);
        }

        let total: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        /* -------------------------
           2. SELECT QUERY
        ------------------------- */

        let mut query_builder = sqlx::QueryBuilder::new(
            "SELECT refresh_tokens.* FROM refresh_tokens \
             LEFT JOIN users ON users.id = refresh_tokens.user_id \
             WHERE refresh_tokens.realm_id = ",
        );
        query_builder.push_bind(realm_id.to_string());
        query_builder.push(
            " AND refresh_tokens.revoked_at IS NULL AND refresh_tokens.replaced_by IS NULL \
             AND refresh_tokens.expires_at > ",
        );
        query_builder.push_bind(Utc::now());

        if let Some(q) = &req.q {
            if !q.is_empty() {
                let pattern = format!("%{}%", q);
                query_builder.push(" AND (refresh_tokens.user_id LIKE ");
                query_builder.push_bind(pattern.clone());
                query_builder.push(" OR users.username LIKE ");
                query_builder.push_bind(pattern);
                query_builder.push(")");
            }
        }

        if let Some(from) = filter.started_from {
            query_builder.push(" AND refresh_tokens.created_at >= ");
            query_builder.push_bind(from);
        }
        if let Some(to) = filter.started_to_exclusive {
            query_builder.push(" AND refresh_tokens.created_at < ");
            query_builder.push_bind(to);
        }

        query_builder.push(" ORDER BY refresh_tokens.created_at DESC");

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
