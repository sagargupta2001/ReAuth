use crate::adapters::persistence::connection::Database;
use crate::domain::realm_security_headers::RealmSecurityHeaders;
use crate::error::{Error, Result};
use crate::ports::realm_security_headers_repository::RealmSecurityHeadersRepository;
use async_trait::async_trait;
use tracing::instrument;
use uuid::Uuid;

pub struct SqliteRealmSecurityHeadersRepository {
    pool: Database,
}

impl SqliteRealmSecurityHeadersRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct RealmSecurityHeadersRecord {
    realm_id: String,
    x_frame_options: Option<String>,
    content_security_policy: Option<String>,
    x_content_type_options: Option<String>,
    referrer_policy: Option<String>,
    strict_transport_security: Option<String>,
}

impl RealmSecurityHeadersRecord {
    fn into_settings(self) -> Result<RealmSecurityHeaders> {
        let realm_id = Uuid::parse_str(&self.realm_id)
            .map_err(|_| Error::System("Invalid realm id in security headers".to_string()))?;
        Ok(RealmSecurityHeaders {
            realm_id,
            x_frame_options: self.x_frame_options,
            content_security_policy: self.content_security_policy,
            x_content_type_options: self.x_content_type_options,
            referrer_policy: self.referrer_policy,
            strict_transport_security: self.strict_transport_security,
        })
    }
}

#[async_trait]
impl RealmSecurityHeadersRepository for SqliteRealmSecurityHeadersRepository {
    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            db_table = "realm_security_headers",
            db_op = "select"
        )
    )]
    async fn find_by_realm_id(&self, realm_id: &Uuid) -> Result<Option<RealmSecurityHeaders>> {
        let record: Option<RealmSecurityHeadersRecord> =
            sqlx::query_as("SELECT * FROM realm_security_headers WHERE realm_id = ?")
                .bind(realm_id.to_string())
                .fetch_optional(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        record
            .map(RealmSecurityHeadersRecord::into_settings)
            .transpose()
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            db_table = "realm_security_headers",
            db_op = "upsert"
        )
    )]
    async fn upsert(&self, settings: &RealmSecurityHeaders) -> Result<()> {
        sqlx::query(
            "INSERT INTO realm_security_headers (
                realm_id, x_frame_options, content_security_policy,
                x_content_type_options, referrer_policy, strict_transport_security
            ) VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(realm_id) DO UPDATE SET
                x_frame_options = excluded.x_frame_options,
                content_security_policy = excluded.content_security_policy,
                x_content_type_options = excluded.x_content_type_options,
                referrer_policy = excluded.referrer_policy,
                strict_transport_security = excluded.strict_transport_security",
        )
        .bind(settings.realm_id.to_string())
        .bind(&settings.x_frame_options)
        .bind(&settings.content_security_policy)
        .bind(&settings.x_content_type_options)
        .bind(&settings.referrer_policy)
        .bind(&settings.strict_transport_security)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(())
    }
}
