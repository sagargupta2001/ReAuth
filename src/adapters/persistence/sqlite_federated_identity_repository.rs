use crate::adapters::persistence::connection::Database;
use crate::domain::identity_provider::FederatedIdentity;
use crate::error::{Error, Result};
use crate::ports::federated_identity_repository::FederatedIdentityRepository;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tracing::instrument;
use uuid::Uuid;

pub struct SqliteFederatedIdentityRepository {
    pool: Database,
}

impl SqliteFederatedIdentityRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct FederatedIdentityRow {
    id: String,
    realm_id: String,
    provider_id: String,
    user_id: String,
    subject: String,
    external_username: Option<String>,
    external_email: Option<String>,
    raw_claims_json: Option<String>,
    linked_via: String,
    last_login_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<FederatedIdentityRow> for FederatedIdentity {
    type Error = Error;

    fn try_from(row: FederatedIdentityRow) -> Result<Self> {
        Ok(Self {
            id: Uuid::parse_str(&row.id)
                .map_err(|_| Error::System("Invalid federated identity id".into()))?,
            realm_id: Uuid::parse_str(&row.realm_id)
                .map_err(|_| Error::System("Invalid federated identity realm id".into()))?,
            provider_id: Uuid::parse_str(&row.provider_id)
                .map_err(|_| Error::System("Invalid federated identity provider id".into()))?,
            user_id: Uuid::parse_str(&row.user_id)
                .map_err(|_| Error::System("Invalid federated identity user id".into()))?,
            subject: row.subject,
            external_username: row.external_username,
            external_email: row.external_email,
            raw_claims_json: row.raw_claims_json,
            linked_via: row.linked_via,
            last_login_at: row.last_login_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[async_trait]
impl FederatedIdentityRepository for SqliteFederatedIdentityRepository {
    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            db_table = "federated_identities",
            db_op = "insert"
        )
    )]
    async fn create(&self, identity: &FederatedIdentity) -> Result<()> {
        sqlx::query(
            "INSERT INTO federated_identities (
                id, realm_id, provider_id, user_id, subject, external_username, external_email,
                raw_claims_json, linked_via, last_login_at, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(identity.id.to_string())
        .bind(identity.realm_id.to_string())
        .bind(identity.provider_id.to_string())
        .bind(identity.user_id.to_string())
        .bind(&identity.subject)
        .bind(&identity.external_username)
        .bind(&identity.external_email)
        .bind(&identity.raw_claims_json)
        .bind(&identity.linked_via)
        .bind(identity.last_login_at)
        .bind(identity.created_at)
        .bind(identity.updated_at)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            db_table = "federated_identities",
            db_op = "update"
        )
    )]
    async fn update(&self, identity: &FederatedIdentity) -> Result<()> {
        sqlx::query(
            "UPDATE federated_identities
             SET external_username = ?, external_email = ?, raw_claims_json = ?, linked_via = ?,
                 last_login_at = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(&identity.external_username)
        .bind(&identity.external_email)
        .bind(&identity.raw_claims_json)
        .bind(&identity.linked_via)
        .bind(identity.last_login_at)
        .bind(identity.updated_at)
        .bind(identity.id.to_string())
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            db_table = "federated_identities",
            db_op = "select"
        )
    )]
    async fn find_by_provider_subject(
        &self,
        realm_id: &Uuid,
        provider_id: &Uuid,
        subject: &str,
    ) -> Result<Option<FederatedIdentity>> {
        let row: Option<FederatedIdentityRow> = sqlx::query_as(
            "SELECT * FROM federated_identities WHERE realm_id = ? AND provider_id = ? AND subject = ?",
        )
        .bind(realm_id.to_string())
        .bind(provider_id.to_string())
        .bind(subject)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        row.map(TryInto::try_into).transpose()
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            db_table = "federated_identities",
            db_op = "select"
        )
    )]
    async fn list_by_user(
        &self,
        realm_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Vec<FederatedIdentity>> {
        let rows: Vec<FederatedIdentityRow> = sqlx::query_as(
            "SELECT * FROM federated_identities WHERE realm_id = ? AND user_id = ? ORDER BY created_at DESC",
        )
        .bind(realm_id.to_string())
        .bind(user_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            db_table = "federated_identities",
            db_op = "select"
        )
    )]
    async fn list_by_provider(
        &self,
        realm_id: &Uuid,
        provider_id: &Uuid,
    ) -> Result<Vec<FederatedIdentity>> {
        let rows: Vec<FederatedIdentityRow> = sqlx::query_as(
            "SELECT * FROM federated_identities WHERE realm_id = ? AND provider_id = ? ORDER BY created_at DESC",
        )
        .bind(realm_id.to_string())
        .bind(provider_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "federated_identities", db_op = "count")
    )]
    async fn count_by_provider(&self, realm_id: &Uuid, provider_id: &Uuid) -> Result<u64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM federated_identities WHERE realm_id = ? AND provider_id = ?",
        )
        .bind(realm_id.to_string())
        .bind(provider_id.to_string())
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(count.max(0) as u64)
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            db_table = "federated_identities",
            db_op = "delete"
        )
    )]
    async fn delete_by_provider(&self, realm_id: &Uuid, provider_id: &Uuid) -> Result<u64> {
        let result =
            sqlx::query("DELETE FROM federated_identities WHERE realm_id = ? AND provider_id = ?")
                .bind(realm_id.to_string())
                .bind(provider_id.to_string())
                .execute(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(result.rows_affected())
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            db_table = "federated_identities",
            db_op = "delete"
        )
    )]
    async fn delete_by_id_for_user(
        &self,
        realm_id: &Uuid,
        user_id: &Uuid,
        federated_identity_id: &Uuid,
    ) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM federated_identities WHERE realm_id = ? AND user_id = ? AND id = ?",
        )
        .bind(realm_id.to_string())
        .bind(user_id.to_string())
        .bind(federated_identity_id.to_string())
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(result.rows_affected() > 0)
    }
}
