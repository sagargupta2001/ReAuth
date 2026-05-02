use crate::adapters::persistence::connection::Database;
use crate::domain::passkey_credential::PasskeyCredential;
use crate::error::{Error, Result};
use crate::ports::passkey_credential_repository::PasskeyCredentialRepository;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use tracing::instrument;
use uuid::Uuid;

pub struct SqlitePasskeyCredentialRepository {
    pool: Database,
}

impl SqlitePasskeyCredentialRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct PasskeyCredentialRecord {
    id: String,
    realm_id: String,
    user_id: String,
    credential_id_b64url: String,
    public_key_cose_b64url: String,
    sign_count: i64,
    transports_json: String,
    backed_up: bool,
    backup_eligible: bool,
    aaguid: Option<String>,
    friendly_name: Option<String>,
    last_used_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl PasskeyCredentialRecord {
    fn into_domain(self) -> Result<PasskeyCredential> {
        Ok(PasskeyCredential {
            id: Uuid::parse_str(&self.id)
                .map_err(|_| Error::System("Invalid passkey credential id".to_string()))?,
            realm_id: Uuid::parse_str(&self.realm_id)
                .map_err(|_| Error::System("Invalid passkey realm id".to_string()))?,
            user_id: Uuid::parse_str(&self.user_id)
                .map_err(|_| Error::System("Invalid passkey user id".to_string()))?,
            credential_id_b64url: self.credential_id_b64url,
            public_key_cose_b64url: self.public_key_cose_b64url,
            sign_count: self.sign_count,
            transports_json: self.transports_json,
            backed_up: self.backed_up,
            backup_eligible: self.backup_eligible,
            aaguid: self.aaguid,
            friendly_name: self.friendly_name,
            last_used_at: self.last_used_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[async_trait]
impl PasskeyCredentialRepository for SqlitePasskeyCredentialRepository {
    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "passkey_credentials", db_op = "insert")
    )]
    async fn create(&self, credential: &PasskeyCredential) -> Result<()> {
        sqlx::query(
            "INSERT INTO passkey_credentials (
                id, realm_id, user_id, credential_id_b64url, public_key_cose_b64url,
                sign_count, transports_json, backed_up, backup_eligible, aaguid,
                friendly_name, last_used_at, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(credential.id.to_string())
        .bind(credential.realm_id.to_string())
        .bind(credential.user_id.to_string())
        .bind(&credential.credential_id_b64url)
        .bind(&credential.public_key_cose_b64url)
        .bind(credential.sign_count)
        .bind(&credential.transports_json)
        .bind(credential.backed_up)
        .bind(credential.backup_eligible)
        .bind(&credential.aaguid)
        .bind(&credential.friendly_name)
        .bind(credential.last_used_at)
        .bind(credential.created_at)
        .bind(credential.updated_at)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "passkey_credentials", db_op = "select")
    )]
    async fn find_by_realm_and_credential_id(
        &self,
        realm_id: &Uuid,
        credential_id_b64url: &str,
    ) -> Result<Option<PasskeyCredential>> {
        let record: Option<PasskeyCredentialRecord> = sqlx::query_as(
            "SELECT * FROM passkey_credentials WHERE realm_id = ? AND credential_id_b64url = ?",
        )
        .bind(realm_id.to_string())
        .bind(credential_id_b64url)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        record.map(PasskeyCredentialRecord::into_domain).transpose()
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "passkey_credentials", db_op = "select")
    )]
    async fn list_by_user(
        &self,
        realm_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Vec<PasskeyCredential>> {
        let records: Vec<PasskeyCredentialRecord> = sqlx::query_as(
            "SELECT * FROM passkey_credentials WHERE realm_id = ? AND user_id = ? ORDER BY created_at ASC",
        )
        .bind(realm_id.to_string())
        .bind(user_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        records
            .into_iter()
            .map(PasskeyCredentialRecord::into_domain)
            .collect()
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "passkey_credentials", db_op = "update")
    )]
    async fn touch_assertion_state(
        &self,
        credential_id: &Uuid,
        observed_sign_count: i64,
        backed_up: bool,
        last_used_at: DateTime<Utc>,
    ) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE passkey_credentials
             SET sign_count = ?, backed_up = ?, last_used_at = ?, updated_at = ?
             WHERE id = ? AND sign_count <= ?",
        )
        .bind(observed_sign_count)
        .bind(backed_up)
        .bind(last_used_at)
        .bind(Utc::now())
        .bind(credential_id.to_string())
        .bind(observed_sign_count)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(result.rows_affected() == 1)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "passkey_credentials", db_op = "count")
    )]
    async fn count_by_realm(&self, realm_id: &Uuid) -> Result<u64> {
        let row =
            sqlx::query("SELECT COUNT(*) AS count FROM passkey_credentials WHERE realm_id = ?")
                .bind(realm_id.to_string())
                .fetch_one(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(row.get::<i64, _>("count").max(0) as u64)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "passkey_credentials", db_op = "count")
    )]
    async fn count_created_since(&self, realm_id: &Uuid, since: DateTime<Utc>) -> Result<u64> {
        let row = sqlx::query(
            "SELECT COUNT(*) AS count
             FROM passkey_credentials
             WHERE realm_id = ? AND created_at >= ?",
        )
        .bind(realm_id.to_string())
        .bind(since)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(row.get::<i64, _>("count").max(0) as u64)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "passkey_credentials", db_op = "count")
    )]
    async fn count_active_since(&self, realm_id: &Uuid, since: DateTime<Utc>) -> Result<u64> {
        let row = sqlx::query(
            "SELECT COUNT(*) AS count
             FROM passkey_credentials
             WHERE realm_id = ?
               AND (last_used_at >= ? OR created_at >= ?)",
        )
        .bind(realm_id.to_string())
        .bind(since)
        .bind(since)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(row.get::<i64, _>("count").max(0) as u64)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "passkey_credentials", db_op = "delete")
    )]
    async fn delete_by_id_for_user(
        &self,
        realm_id: &Uuid,
        user_id: &Uuid,
        credential_id: &Uuid,
    ) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM passkey_credentials
             WHERE id = ? AND realm_id = ? AND user_id = ?",
        )
        .bind(credential_id.to_string())
        .bind(realm_id.to_string())
        .bind(user_id.to_string())
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(result.rows_affected() == 1)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "passkey_credentials", db_op = "update")
    )]
    async fn update_friendly_name_for_user(
        &self,
        realm_id: &Uuid,
        user_id: &Uuid,
        credential_id: &Uuid,
        friendly_name: Option<String>,
    ) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE passkey_credentials
             SET friendly_name = ?, updated_at = ?
             WHERE id = ? AND realm_id = ? AND user_id = ?",
        )
        .bind(friendly_name)
        .bind(Utc::now())
        .bind(credential_id.to_string())
        .bind(realm_id.to_string())
        .bind(user_id.to_string())
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(result.rows_affected() == 1)
    }
}
