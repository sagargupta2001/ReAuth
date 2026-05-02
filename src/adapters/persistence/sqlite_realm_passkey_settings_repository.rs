use crate::adapters::persistence::connection::Database;
use crate::domain::realm_passkey_settings::RealmPasskeySettings;
use crate::error::{Error, Result};
use crate::ports::realm_passkey_settings_repository::RealmPasskeySettingsRepository;
use async_trait::async_trait;
use tracing::instrument;
use uuid::Uuid;

pub struct SqliteRealmPasskeySettingsRepository {
    pool: Database,
}

impl SqliteRealmPasskeySettingsRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct RealmPasskeySettingsRecord {
    realm_id: String,
    enabled: bool,
    allow_password_fallback: bool,
    discoverable_preferred: bool,
    challenge_ttl_secs: i64,
    reauth_max_age_secs: i64,
}

impl RealmPasskeySettingsRecord {
    fn into_settings(self) -> Result<RealmPasskeySettings> {
        let realm_id = Uuid::parse_str(&self.realm_id)
            .map_err(|_| Error::System("Invalid realm id in passkey settings".to_string()))?;
        Ok(RealmPasskeySettings {
            realm_id,
            enabled: self.enabled,
            allow_password_fallback: self.allow_password_fallback,
            discoverable_preferred: self.discoverable_preferred,
            challenge_ttl_secs: self.challenge_ttl_secs,
            reauth_max_age_secs: self.reauth_max_age_secs,
        })
    }
}

#[async_trait]
impl RealmPasskeySettingsRepository for SqliteRealmPasskeySettingsRepository {
    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            db_table = "realm_passkey_settings",
            db_op = "select"
        )
    )]
    async fn find_by_realm_id(&self, realm_id: &Uuid) -> Result<Option<RealmPasskeySettings>> {
        let record: Option<RealmPasskeySettingsRecord> =
            sqlx::query_as("SELECT * FROM realm_passkey_settings WHERE realm_id = ?")
                .bind(realm_id.to_string())
                .fetch_optional(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;

        record
            .map(RealmPasskeySettingsRecord::into_settings)
            .transpose()
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            db_table = "realm_passkey_settings",
            db_op = "upsert"
        )
    )]
    async fn upsert(&self, settings: &RealmPasskeySettings) -> Result<()> {
        sqlx::query(
            "INSERT INTO realm_passkey_settings (
                realm_id, enabled, allow_password_fallback, discoverable_preferred,
                challenge_ttl_secs, reauth_max_age_secs
            ) VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(realm_id) DO UPDATE SET
                enabled = excluded.enabled,
                allow_password_fallback = excluded.allow_password_fallback,
                discoverable_preferred = excluded.discoverable_preferred,
                challenge_ttl_secs = excluded.challenge_ttl_secs,
                reauth_max_age_secs = excluded.reauth_max_age_secs,
                updated_at = CURRENT_TIMESTAMP",
        )
        .bind(settings.realm_id.to_string())
        .bind(settings.enabled)
        .bind(settings.allow_password_fallback)
        .bind(settings.discoverable_preferred)
        .bind(settings.challenge_ttl_secs)
        .bind(settings.reauth_max_age_secs)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(())
    }
}
