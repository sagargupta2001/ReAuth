use crate::adapters::persistence::connection::Database;
use crate::domain::pagination::{PageRequest, PageResponse, SortDirection};
use crate::{
    domain::oidc::{AuthCode, OidcClient},
    error::{Error, Result},
    ports::oidc_repository::OidcRepository,
};
use async_trait::async_trait;
use chrono::Utc;
use sqlx::{QueryBuilder, Sqlite};
use uuid::Uuid;

///This repository handles the stateful, temporary data required by the OIDC protocol (Clients and Auth Codes).
pub struct SqliteOidcRepository {
    pool: Database,
}

impl SqliteOidcRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }

    /// Helper: Applies the standard filters (Realm ID + Search) to any query builder.
    /// This ensures `COUNT(*)` and `SELECT *` always use the same criteria.
    fn apply_filters<'a>(
        builder: &mut QueryBuilder<'a, Sqlite>,
        realm_id: &Uuid,
        q: &Option<String>,
    ) {
        // 1. Base Constraint
        builder.push(" WHERE realm_id = ");
        builder.push_bind(realm_id.to_string());

        // 2. Search Filter
        if let Some(query_text) = q {
            if !query_text.is_empty() {
                builder.push(" AND client_id LIKE ");
                builder.push_bind(format!("%{}%", query_text));
            }
        }
    }
}

#[async_trait]
impl OidcRepository for SqliteOidcRepository {
    // --- Client Management ---

    async fn find_client_by_id(
        &self,
        realm_id: &Uuid,
        client_id: &str,
    ) -> Result<Option<OidcClient>> {
        let client =
            sqlx::query_as("SELECT * FROM oidc_clients WHERE realm_id = ? AND client_id = ?")
                .bind(realm_id.to_string())
                .bind(client_id)
                .fetch_optional(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(client)
    }

    async fn create_client(&self, client: &OidcClient) -> Result<()> {
        sqlx::query(
            "INSERT INTO oidc_clients (id, realm_id, client_id, client_secret, redirect_uris, scopes, web_origins, managed_by_config)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
            .bind(client.id.to_string())
            .bind(client.realm_id.to_string())
            .bind(&client.client_id)
            .bind(&client.client_secret)
            .bind(&client.redirect_uris)
            .bind(&client.scopes)
            .bind(&client.web_origins)
            .bind(client.managed_by_config)
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn find_clients_by_realm(
        &self,
        realm_id: &Uuid,
        req: &PageRequest,
    ) -> Result<PageResponse<OidcClient>> {
        let limit = req.per_page.clamp(1, 100);
        let offset = (req.page - 1) * limit;

        // We create a builder just for counting
        let mut count_builder = QueryBuilder::new("SELECT COUNT(*) FROM oidc_clients");

        // Apply the shared filters
        Self::apply_filters(&mut count_builder, realm_id, &req.q);

        let total: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        // We create a new builder for the actual data
        let mut query_builder = QueryBuilder::new("SELECT * FROM oidc_clients");

        // Apply the SAME shared filters
        Self::apply_filters(&mut query_builder, realm_id, &req.q);

        // Apply Sorting
        // Whitelist sort columns to prevent SQL injection
        let sort_col = match req.sort_by.as_deref() {
            Some("client_id") => "client_id",
            // Add "created_at" here if you add that column to your DB schema later
            _ => "client_id",
        };

        let sort_dir = match req.sort_dir.unwrap_or(SortDirection::Asc) {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };

        query_builder.push(format!(" ORDER BY {} {}", sort_col, sort_dir));

        // Apply Pagination
        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        // Execute
        let clients: Vec<OidcClient> = query_builder
            .build_query_as()
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(PageResponse::new(clients, total, req.page, limit))
    }

    async fn find_client_by_uuid(&self, id: &Uuid) -> Result<Option<OidcClient>> {
        let client = sqlx::query_as("SELECT * FROM oidc_clients WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&*self.pool)
            .await
            .map_err(|_| Error::OidcClientNotFound(id.to_string()))?;
        Ok(client)
    }

    async fn update_client(&self, client: &OidcClient) -> Result<()> {
        sqlx::query(
            "UPDATE oidc_clients SET client_id = ?, redirect_uris = ?, scopes = ?, web_origins = ?, managed_by_config = ? WHERE id = ?",
        )
        .bind(&client.client_id)
        .bind(&client.redirect_uris)
        .bind(&client.scopes)
        .bind(&client.web_origins)
        .bind(client.managed_by_config)
        .bind(client.id.to_string())
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    // --- Auth Code Management ---

    async fn save_auth_code(&self, code: &AuthCode) -> Result<()> {
        sqlx::query(
            "INSERT INTO authorization_codes (code, user_id, client_id, redirect_uri, nonce, code_challenge, code_challenge_method, expires_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
            .bind(&code.code)
            .bind(code.user_id.to_string())
            .bind(&code.client_id)
            .bind(&code.redirect_uri)
            .bind(&code.nonce)
            .bind(&code.code_challenge)
            .bind(&code.code_challenge_method)
            .bind(code.expires_at)
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn find_auth_code_by_code(&self, code: &str) -> Result<Option<AuthCode>> {
        let code =
            sqlx::query_as("SELECT * FROM authorization_codes WHERE code = ? AND expires_at > ?")
                .bind(code)
                .bind(Utc::now())
                .fetch_optional(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(code)
    }

    async fn delete_auth_code(&self, code: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM authorization_codes WHERE code = ?")
            .bind(code)
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        // Best practice: Check rows affected to detect concurrent usage
        if result.rows_affected() == 0 {
            return Err(Error::Unexpected(anyhow::anyhow!(
                "Authorization code not found for deletion or already expired."
            )));
        }
        Ok(())
    }

    async fn is_origin_allowed(&self, origin: &str) -> Result<bool> {
        // We use a naive LIKE query to find the origin inside the JSON array string
        // For 'http://localhost:3000', we search for '%"http://localhost:3000"%'
        let pattern = format!("%\"{}\"%", origin);

        let result: Option<(i32,)> =
            sqlx::query_as("SELECT 1 FROM oidc_clients WHERE web_origins LIKE ? LIMIT 1")
                .bind(pattern)
                .fetch_optional(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(result.is_some())
    }
}
