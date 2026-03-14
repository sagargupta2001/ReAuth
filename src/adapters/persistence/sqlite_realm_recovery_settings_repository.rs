use crate::adapters::persistence::connection::Database;
use crate::domain::realm_recovery_settings::RealmRecoverySettings;
use crate::error::{Error, Result};
use crate::ports::realm_recovery_settings_repository::RealmRecoverySettingsRepository;
use async_trait::async_trait;
use tracing::instrument;
use uuid::Uuid;

pub struct SqliteRealmRecoverySettingsRepository {
    pool: Database,
}

impl SqliteRealmRecoverySettingsRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct RealmRecoverySettingsRecord {
    realm_id: String,
    token_ttl_minutes: i64,
    rate_limit_max: i64,
    rate_limit_window_minutes: i64,
    revoke_sessions_on_reset: bool,
    email_subject: Option<String>,
    email_body: Option<String>,
}

impl RealmRecoverySettingsRecord {
    fn into_settings(self) -> Result<RealmRecoverySettings> {
        let realm_id = Uuid::parse_str(&self.realm_id)
            .map_err(|_| Error::System("Invalid realm id in recovery settings".to_string()))?;
        Ok(RealmRecoverySettings {
            realm_id,
            token_ttl_minutes: self.token_ttl_minutes,
            rate_limit_max: self.rate_limit_max,
            rate_limit_window_minutes: self.rate_limit_window_minutes,
            revoke_sessions_on_reset: self.revoke_sessions_on_reset,
            email_subject: self.email_subject,
            email_body: self.email_body,
        })
    }
}

#[async_trait]
impl RealmRecoverySettingsRepository for SqliteRealmRecoverySettingsRepository {
    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            db_table = "realm_recovery_settings",
            db_op = "select"
        )
    )]
    async fn find_by_realm_id(&self, realm_id: &Uuid) -> Result<Option<RealmRecoverySettings>> {
        let record: Option<RealmRecoverySettingsRecord> =
            sqlx::query_as("SELECT * FROM realm_recovery_settings WHERE realm_id = ?")
                .bind(realm_id.to_string())
                .fetch_optional(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        record
            .map(RealmRecoverySettingsRecord::into_settings)
            .transpose()
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            db_table = "realm_recovery_settings",
            db_op = "upsert"
        )
    )]
    async fn upsert(&self, settings: &RealmRecoverySettings) -> Result<()> {
        sqlx::query(
            "INSERT INTO realm_recovery_settings (
                realm_id, token_ttl_minutes, rate_limit_max, rate_limit_window_minutes,
                revoke_sessions_on_reset, email_subject, email_body
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(realm_id) DO UPDATE SET
                token_ttl_minutes = excluded.token_ttl_minutes,
                rate_limit_max = excluded.rate_limit_max,
                rate_limit_window_minutes = excluded.rate_limit_window_minutes,
                revoke_sessions_on_reset = excluded.revoke_sessions_on_reset,
                email_subject = excluded.email_subject,
                email_body = excluded.email_body",
        )
        .bind(settings.realm_id.to_string())
        .bind(settings.token_ttl_minutes)
        .bind(settings.rate_limit_max)
        .bind(settings.rate_limit_window_minutes)
        .bind(settings.revoke_sessions_on_reset)
        .bind(&settings.email_subject)
        .bind(&settings.email_body)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(())
    }
}
