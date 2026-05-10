use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Expired,
    Revoked,
}

impl fmt::Display for InvitationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvitationStatus::Pending => write!(f, "pending"),
            InvitationStatus::Accepted => write!(f, "accepted"),
            InvitationStatus::Expired => write!(f, "expired"),
            InvitationStatus::Revoked => write!(f, "revoked"),
        }
    }
}

impl From<String> for InvitationStatus {
    fn from(value: String) -> Self {
        match value.as_str() {
            "accepted" => InvitationStatus::Accepted,
            "expired" => InvitationStatus::Expired,
            "revoked" => InvitationStatus::Revoked,
            _ => InvitationStatus::Pending,
        }
    }
}

impl sqlx::Type<sqlx::Sqlite> for InvitationStatus {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for InvitationStatus {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> std::result::Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let value = self.to_string();
        <String as sqlx::Encode<sqlx::Sqlite>>::encode(value, args)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for InvitationStatus {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let value: String = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        Ok(InvitationStatus::from(value))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invitation {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub email: String,
    pub email_normalized: String,
    pub status: InvitationStatus,
    pub token_hash: String,
    pub expiry_days: i64,
    pub expires_at: DateTime<Utc>,
    pub invited_by_user_id: Option<Uuid>,
    pub accepted_user_id: Option<Uuid>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub resend_count: i64,
    pub last_sent_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Invitation {
    pub fn is_expired(&self) -> bool {
        self.expires_at <= Utc::now()
    }
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for Invitation {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        let parse_uuid = |val: String, col_name: &str| -> Result<Uuid, sqlx::Error> {
            Uuid::parse_str(&val).map_err(|e| sqlx::Error::ColumnDecode {
                index: col_name.into(),
                source: Box::new(e),
            })
        };

        let id_str: String = row.try_get("id")?;
        let realm_id_str: String = row.try_get("realm_id")?;
        let invited_by_user_id_str: Option<String> = row.try_get("invited_by_user_id")?;
        let accepted_user_id_str: Option<String> = row.try_get("accepted_user_id")?;

        let invited_by_user_id = match invited_by_user_id_str {
            Some(value) => Some(parse_uuid(value, "invited_by_user_id")?),
            None => None,
        };
        let accepted_user_id = match accepted_user_id_str {
            Some(value) => Some(parse_uuid(value, "accepted_user_id")?),
            None => None,
        };

        Ok(Self {
            id: parse_uuid(id_str, "id")?,
            realm_id: parse_uuid(realm_id_str, "realm_id")?,
            email: row.try_get("email")?,
            email_normalized: row.try_get("email_normalized")?,
            status: row.try_get("status")?,
            token_hash: row.try_get("token_hash")?,
            expiry_days: row.try_get("expiry_days")?,
            expires_at: row.try_get("expires_at")?,
            invited_by_user_id,
            accepted_user_id,
            accepted_at: row.try_get("accepted_at")?,
            revoked_at: row.try_get("revoked_at")?,
            resend_count: row.try_get("resend_count")?,
            last_sent_at: row.try_get("last_sent_at")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
