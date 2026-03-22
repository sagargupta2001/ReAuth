use crate::adapters::persistence::connection::Database;
use crate::domain::realm_email_settings::RealmEmailSettings;
use crate::error::{Error, Result};
use crate::ports::realm_email_settings_repository::RealmEmailSettingsRepository;
use async_trait::async_trait;
use tracing::instrument;
use uuid::Uuid;

pub struct SqliteRealmEmailSettingsRepository {
    pool: Database,
}

impl SqliteRealmEmailSettingsRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct RealmEmailSettingsRecord {
    realm_id: String,
    enabled: bool,
    from_address: Option<String>,
    from_name: Option<String>,
    reply_to_address: Option<String>,
    smtp_host: Option<String>,
    smtp_port: Option<i64>,
    smtp_username: Option<String>,
    smtp_password: Option<String>,
    smtp_security: String,
}

impl RealmEmailSettingsRecord {
    fn into_settings(self) -> Result<RealmEmailSettings> {
        let realm_id = Uuid::parse_str(&self.realm_id)
            .map_err(|_| Error::System("Invalid realm id in email settings".to_string()))?;
        Ok(RealmEmailSettings {
            realm_id,
            enabled: self.enabled,
            from_address: self.from_address,
            from_name: self.from_name,
            reply_to_address: self.reply_to_address,
            smtp_host: self.smtp_host,
            smtp_port: self.smtp_port,
            smtp_username: self.smtp_username,
            smtp_password: self.smtp_password,
            smtp_security: self.smtp_security,
        })
    }
}

#[async_trait]
impl RealmEmailSettingsRepository for SqliteRealmEmailSettingsRepository {
    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            db_table = "realm_email_settings",
            db_op = "select"
        )
    )]
    async fn find_by_realm_id(&self, realm_id: &Uuid) -> Result<Option<RealmEmailSettings>> {
        let record: Option<RealmEmailSettingsRecord> =
            sqlx::query_as("SELECT * FROM realm_email_settings WHERE realm_id = ?")
                .bind(realm_id.to_string())
                .fetch_optional(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        record
            .map(RealmEmailSettingsRecord::into_settings)
            .transpose()
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            db_table = "realm_email_settings",
            db_op = "upsert"
        )
    )]
    async fn upsert(&self, settings: &RealmEmailSettings) -> Result<()> {
        sqlx::query(
            "INSERT INTO realm_email_settings (
                realm_id, enabled, from_address, from_name, reply_to_address,
                smtp_host, smtp_port, smtp_username, smtp_password, smtp_security
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(realm_id) DO UPDATE SET
                enabled = excluded.enabled,
                from_address = excluded.from_address,
                from_name = excluded.from_name,
                reply_to_address = excluded.reply_to_address,
                smtp_host = excluded.smtp_host,
                smtp_port = excluded.smtp_port,
                smtp_username = excluded.smtp_username,
                smtp_password = excluded.smtp_password,
                smtp_security = excluded.smtp_security",
        )
        .bind(settings.realm_id.to_string())
        .bind(settings.enabled)
        .bind(&settings.from_address)
        .bind(&settings.from_name)
        .bind(&settings.reply_to_address)
        .bind(&settings.smtp_host)
        .bind(settings.smtp_port)
        .bind(&settings.smtp_username)
        .bind(&settings.smtp_password)
        .bind(&settings.smtp_security)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(())
    }
}
