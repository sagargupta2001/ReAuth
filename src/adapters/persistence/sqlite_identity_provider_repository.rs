use crate::adapters::persistence::connection::Database;
use crate::domain::identity_provider::{IdentityProvider, IdentityProviderProtocol};
use crate::error::{Error, Result};
use crate::ports::identity_provider_repository::IdentityProviderRepository;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tracing::instrument;
use uuid::Uuid;

pub struct SqliteIdentityProviderRepository {
    pool: Database,
}

impl SqliteIdentityProviderRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct IdentityProviderRow {
    id: String,
    realm_id: String,
    alias: String,
    display_name: String,
    protocol: String,
    preset_key: Option<String>,
    enabled: bool,
    client_id: String,
    client_secret: Option<String>,
    issuer: Option<String>,
    authorization_endpoint: Option<String>,
    token_endpoint: Option<String>,
    userinfo_endpoint: Option<String>,
    jwks_uri: Option<String>,
    scopes_json: String,
    claim_mapping_json: String,
    pkce_required: bool,
    allow_login: bool,
    allow_link: bool,
    allow_jit_provisioning: bool,
    allow_email_auto_link: bool,
    require_verified_email: bool,
    icon_ref: Option<String>,
    button_color: Option<String>,
    sort_order: i64,
    metadata_cached_at: Option<DateTime<Utc>>,
    metadata_cache_json: Option<String>,
    jwks_cached_at: Option<DateTime<Utc>>,
    jwks_cache_json: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<IdentityProviderRow> for IdentityProvider {
    type Error = Error;

    fn try_from(row: IdentityProviderRow) -> Result<Self> {
        Ok(Self {
            id: Uuid::parse_str(&row.id)
                .map_err(|_| Error::System("Invalid provider id".into()))?,
            realm_id: Uuid::parse_str(&row.realm_id)
                .map_err(|_| Error::System("Invalid provider realm id".into()))?,
            alias: row.alias,
            display_name: row.display_name,
            protocol: IdentityProviderProtocol::try_from(row.protocol).map_err(Error::System)?,
            preset_key: row.preset_key,
            enabled: row.enabled,
            client_id: row.client_id,
            client_secret: row.client_secret,
            issuer: row.issuer,
            authorization_endpoint: row.authorization_endpoint,
            token_endpoint: row.token_endpoint,
            userinfo_endpoint: row.userinfo_endpoint,
            jwks_uri: row.jwks_uri,
            scopes_json: row.scopes_json,
            claim_mapping_json: row.claim_mapping_json,
            pkce_required: row.pkce_required,
            allow_login: row.allow_login,
            allow_link: row.allow_link,
            allow_jit_provisioning: row.allow_jit_provisioning,
            allow_email_auto_link: row.allow_email_auto_link,
            require_verified_email: row.require_verified_email,
            icon_ref: row.icon_ref,
            button_color: row.button_color,
            sort_order: row.sort_order,
            metadata_cached_at: row.metadata_cached_at,
            metadata_cache_json: row.metadata_cache_json,
            jwks_cached_at: row.jwks_cached_at,
            jwks_cache_json: row.jwks_cache_json,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[async_trait]
impl IdentityProviderRepository for SqliteIdentityProviderRepository {
    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "identity_providers", db_op = "insert")
    )]
    async fn create(&self, provider: &IdentityProvider) -> Result<()> {
        sqlx::query(
            "INSERT INTO identity_providers (
                id, realm_id, alias, display_name, protocol, preset_key, enabled, client_id,
                client_secret, issuer, authorization_endpoint, token_endpoint, userinfo_endpoint,
                jwks_uri, scopes_json, claim_mapping_json, pkce_required, allow_login, allow_link,
                allow_jit_provisioning, allow_email_auto_link, require_verified_email, icon_ref,
                button_color, sort_order, metadata_cached_at, metadata_cache_json, jwks_cached_at,
                jwks_cache_json, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(provider.id.to_string())
        .bind(provider.realm_id.to_string())
        .bind(&provider.alias)
        .bind(&provider.display_name)
        .bind(provider.protocol.to_string())
        .bind(&provider.preset_key)
        .bind(provider.enabled)
        .bind(&provider.client_id)
        .bind(&provider.client_secret)
        .bind(&provider.issuer)
        .bind(&provider.authorization_endpoint)
        .bind(&provider.token_endpoint)
        .bind(&provider.userinfo_endpoint)
        .bind(&provider.jwks_uri)
        .bind(&provider.scopes_json)
        .bind(&provider.claim_mapping_json)
        .bind(provider.pkce_required)
        .bind(provider.allow_login)
        .bind(provider.allow_link)
        .bind(provider.allow_jit_provisioning)
        .bind(provider.allow_email_auto_link)
        .bind(provider.require_verified_email)
        .bind(&provider.icon_ref)
        .bind(&provider.button_color)
        .bind(provider.sort_order)
        .bind(provider.metadata_cached_at)
        .bind(&provider.metadata_cache_json)
        .bind(provider.jwks_cached_at)
        .bind(&provider.jwks_cache_json)
        .bind(provider.created_at)
        .bind(provider.updated_at)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "identity_providers", db_op = "update")
    )]
    async fn update(&self, provider: &IdentityProvider) -> Result<()> {
        sqlx::query(
            "UPDATE identity_providers SET
                alias = ?, display_name = ?, protocol = ?, preset_key = ?, enabled = ?, client_id = ?,
                client_secret = ?, issuer = ?, authorization_endpoint = ?, token_endpoint = ?,
                userinfo_endpoint = ?, jwks_uri = ?, scopes_json = ?, claim_mapping_json = ?,
                pkce_required = ?, allow_login = ?, allow_link = ?, allow_jit_provisioning = ?,
                allow_email_auto_link = ?, require_verified_email = ?, icon_ref = ?, button_color = ?,
                sort_order = ?, metadata_cached_at = ?, metadata_cache_json = ?, jwks_cached_at = ?,
                jwks_cache_json = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(&provider.alias)
        .bind(&provider.display_name)
        .bind(provider.protocol.to_string())
        .bind(&provider.preset_key)
        .bind(provider.enabled)
        .bind(&provider.client_id)
        .bind(&provider.client_secret)
        .bind(&provider.issuer)
        .bind(&provider.authorization_endpoint)
        .bind(&provider.token_endpoint)
        .bind(&provider.userinfo_endpoint)
        .bind(&provider.jwks_uri)
        .bind(&provider.scopes_json)
        .bind(&provider.claim_mapping_json)
        .bind(provider.pkce_required)
        .bind(provider.allow_login)
        .bind(provider.allow_link)
        .bind(provider.allow_jit_provisioning)
        .bind(provider.allow_email_auto_link)
        .bind(provider.require_verified_email)
        .bind(&provider.icon_ref)
        .bind(&provider.button_color)
        .bind(provider.sort_order)
        .bind(provider.metadata_cached_at)
        .bind(&provider.metadata_cache_json)
        .bind(provider.jwks_cached_at)
        .bind(&provider.jwks_cache_json)
        .bind(provider.updated_at)
        .bind(provider.id.to_string())
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "identity_providers", db_op = "select")
    )]
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<IdentityProvider>> {
        let row: Option<IdentityProviderRow> =
            sqlx::query_as("SELECT * FROM identity_providers WHERE id = ?")
                .bind(id.to_string())
                .fetch_optional(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        row.map(TryInto::try_into).transpose()
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "identity_providers", db_op = "select")
    )]
    async fn find_by_alias(
        &self,
        realm_id: &Uuid,
        alias: &str,
    ) -> Result<Option<IdentityProvider>> {
        let row: Option<IdentityProviderRow> =
            sqlx::query_as("SELECT * FROM identity_providers WHERE realm_id = ? AND alias = ?")
                .bind(realm_id.to_string())
                .bind(alias)
                .fetch_optional(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        row.map(TryInto::try_into).transpose()
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "identity_providers", db_op = "select")
    )]
    async fn list_by_realm(&self, realm_id: &Uuid) -> Result<Vec<IdentityProvider>> {
        let rows: Vec<IdentityProviderRow> = sqlx::query_as(
            "SELECT * FROM identity_providers WHERE realm_id = ? ORDER BY sort_order ASC, display_name ASC",
        )
        .bind(realm_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "identity_providers", db_op = "delete")
    )]
    async fn delete(&self, id: &Uuid) -> Result<()> {
        sqlx::query("DELETE FROM identity_providers WHERE id = ?")
            .bind(id.to_string())
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }
}
