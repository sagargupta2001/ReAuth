use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshToken {
    pub id: Uuid,
    pub family_id: Uuid,
    pub user_id: Uuid,
    pub realm_id: Uuid,
    pub client_id: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub replaced_by: Option<Uuid>,
}

impl RefreshToken {
    pub fn new(
        user_id: Uuid,
        realm_id: Uuid,
        client_id: Option<String>,
        expires_in: chrono::Duration,
    ) -> Self {
        let now = Utc::now();
        let family_id = Uuid::new_v4();
        Self {
            id: Uuid::new_v4(),
            family_id,
            user_id,
            realm_id,
            client_id,
            expires_at: now + expires_in,
            ip_address: None,
            user_agent: None,
            created_at: now,
            last_used_at: now,
            revoked_at: None,
            replaced_by: None,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }
}

// Manual implementation to safely map SQLite Strings -> Rust Uuid
impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for RefreshToken {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        let parse_uuid = |val: String, col_name: &str| -> Result<Uuid, sqlx::Error> {
            Uuid::parse_str(&val).map_err(|e| sqlx::Error::ColumnDecode {
                index: col_name.into(),
                source: Box::new(e),
            })
        };

        let id_str: String = row.try_get("id")?;
        let family_id_str: String = row.try_get("family_id")?;
        let user_id_str: String = row.try_get("user_id")?;
        let realm_id_str: String = row.try_get("realm_id")?;
        let replaced_by_str: Option<String> = row.try_get("replaced_by")?;
        let replaced_by = match replaced_by_str {
            Some(value) => Some(parse_uuid(value, "replaced_by")?),
            None => None,
        };

        Ok(RefreshToken {
            id: parse_uuid(id_str, "id")?,
            family_id: parse_uuid(family_id_str, "family_id")?,
            user_id: parse_uuid(user_id_str, "user_id")?,
            realm_id: parse_uuid(realm_id_str, "realm_id")?,
            client_id: row.try_get("client_id")?,
            expires_at: row.try_get("expires_at")?,
            ip_address: row.try_get("ip_address")?,
            user_agent: row.try_get("user_agent")?,
            created_at: row.try_get("created_at")?,
            last_used_at: row.try_get("last_used_at")?,
            revoked_at: row.try_get("revoked_at")?,
            replaced_by,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use super::RefreshToken;
    use chrono::{TimeZone, Utc};
    use sqlx::SqlitePool;
    use uuid::Uuid;

    #[tokio::test]
    async fn refresh_token_from_row_works() {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect");
        let id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let realm_id = Uuid::new_v4();
        let now = Utc::now();

        let token: RefreshToken = sqlx::query_as(
        "SELECT ? as id, ? as family_id, ? as user_id, ? as realm_id, ? as client_id, ? as expires_at, ? as ip_address, ? as user_agent, ? as created_at, ? as last_used_at, ? as revoked_at, ? as replaced_by",
    )
    .bind(id.to_string())
    .bind(id.to_string())
    .bind(user_id.to_string())
    .bind(realm_id.to_string())
    .bind("client")
    .bind(now)
    .bind("127.0.0.1")
    .bind("agent")
    .bind(now)
    .bind(now)
    .bind::<Option<chrono::DateTime<Utc>>>(None)
    .bind::<Option<String>>(None)
    .fetch_one(&pool)
    .await
    .expect("fetch token");

        assert_eq!(token.id, id);
        assert_eq!(token.family_id, id);
        assert_eq!(token.user_id, user_id);
        assert_eq!(token.realm_id, realm_id);
        assert_eq!(token.client_id, Some("client".to_string()));
        // SQLite might lose some precision in datetime, but should be close enough
        assert_eq!(token.expires_at.timestamp(), now.timestamp());
        assert_eq!(token.ip_address, Some("127.0.0.1".to_string()));
        assert_eq!(token.user_agent, Some("agent".to_string()));
        assert!(token.revoked_at.is_none());
        assert!(token.replaced_by.is_none());
    }

    #[test]
    fn refresh_token_round_trip() {
        let now = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
        let token = RefreshToken {
            id: Uuid::new_v4(),
            family_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            realm_id: Uuid::new_v4(),
            client_id: Some("client".to_string()),
            expires_at: now,
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("test-agent".to_string()),
            created_at: now,
            last_used_at: now,
            revoked_at: None,
            replaced_by: None,
        };

        let json = serde_json::to_string(&token).expect("serialize");
        let decoded: RefreshToken = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(decoded.id, token.id);
        assert_eq!(decoded.user_id, token.user_id);
        assert_eq!(decoded.realm_id, token.realm_id);
        assert_eq!(decoded.client_id, token.client_id);
        assert_eq!(decoded.expires_at, token.expires_at);
        assert_eq!(decoded.ip_address, token.ip_address);
        assert_eq!(decoded.user_agent, token.user_agent);
        assert_eq!(decoded.created_at, token.created_at);
        assert_eq!(decoded.last_used_at, token.last_used_at);
    }

    #[test]
    fn refresh_token_new_logic() {
        let user_id = Uuid::new_v4();
        let realm_id = Uuid::new_v4();
        let duration = chrono::Duration::hours(1);

        let token = RefreshToken::new(user_id, realm_id, Some("client".to_string()), duration);

        assert!(!token.id.is_nil());
        assert_eq!(token.user_id, user_id);
        assert_eq!(token.realm_id, realm_id);
        assert_eq!(token.client_id, Some("client".to_string()));
        assert!(token.expires_at > token.created_at);
        assert!(!token.is_expired());
    }

    #[test]
    fn refresh_token_expiration() {
        let user_id = Uuid::new_v4();
        let realm_id = Uuid::new_v4();

        // Expired token
        let expired_token =
            RefreshToken::new(user_id, realm_id, None, chrono::Duration::seconds(-1));
        assert!(expired_token.is_expired());

        // Valid token
        let valid_token = RefreshToken::new(user_id, realm_id, None, chrono::Duration::minutes(1));
        assert!(!valid_token.is_expired());
    }
}
