use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Invitation {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
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
