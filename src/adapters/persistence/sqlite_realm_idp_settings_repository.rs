use crate::adapters::persistence::connection::Database;
use crate::domain::realm_idp_settings::RealmIdpSettings;
use crate::error::{Error, Result};
use crate::ports::realm_idp_settings_repository::RealmIdpSettingsRepository;
use async_trait::async_trait;
use tracing::instrument;
use uuid::Uuid;

pub struct SqliteRealmIdpSettingsRepository {
    pool: Database,
}

impl SqliteRealmIdpSettingsRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct RealmIdpSettingsRecord {
    realm_id: String,
    oauth_start_rate_limit_max: i64,
    oauth_start_rate_limit_window_minutes: i64,
}

impl RealmIdpSettingsRecord {
    fn into_settings(self) -> Result<RealmIdpSettings> {
        let realm_id = Uuid::parse_str(&self.realm_id).map_err(|_| {
            Error::System("Invalid realm id in identity provider settings".to_string())
        })?;
        Ok(RealmIdpSettings {
            realm_id,
            oauth_start_rate_limit_max: self.oauth_start_rate_limit_max,
            oauth_start_rate_limit_window_minutes: self.oauth_start_rate_limit_window_minutes,
        })
    }
}

#[async_trait]
impl RealmIdpSettingsRepository for SqliteRealmIdpSettingsRepository {
    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "realm_idp_settings", db_op = "select")
    )]
    async fn find_by_realm_id(&self, realm_id: &Uuid) -> Result<Option<RealmIdpSettings>> {
        let record: Option<RealmIdpSettingsRecord> =
            sqlx::query_as("SELECT * FROM realm_idp_settings WHERE realm_id = ?")
                .bind(realm_id.to_string())
                .fetch_optional(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        record
            .map(RealmIdpSettingsRecord::into_settings)
            .transpose()
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "realm_idp_settings", db_op = "upsert")
    )]
    async fn upsert(&self, settings: &RealmIdpSettings) -> Result<()> {
        sqlx::query(
            "INSERT INTO realm_idp_settings (
                realm_id, oauth_start_rate_limit_max, oauth_start_rate_limit_window_minutes
            ) VALUES (?, ?, ?)
            ON CONFLICT(realm_id) DO UPDATE SET
                oauth_start_rate_limit_max = excluded.oauth_start_rate_limit_max,
                oauth_start_rate_limit_window_minutes = excluded.oauth_start_rate_limit_window_minutes",
        )
        .bind(settings.realm_id.to_string())
        .bind(settings.oauth_start_rate_limit_max)
        .bind(settings.oauth_start_rate_limit_window_minutes)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(())
    }
}
