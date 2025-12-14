use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    active,
    completed,
    failed,
}

// 1. Display (Required for Encode)
impl fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionStatus::active => write!(f, "active"),
            SessionStatus::completed => write!(f, "completed"),
            SessionStatus::failed => write!(f, "failed"),
        }
    }
}

// 2. From String (Required for Decode)
impl From<String> for SessionStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "completed" => SessionStatus::completed,
            "failed" => SessionStatus::failed,
            _ => SessionStatus::active,
        }
    }
}

// --- SQLX GLUE CODE ---

// 3. Tell SQLx this enum maps to a SQLite TEXT column
impl sqlx::Type<sqlx::Sqlite> for SessionStatus {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

// 4. Teach SQLx how to write it to DB (Enum -> String -> DB)
impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for SessionStatus {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> sqlx::encode::IsNull {
        let s = self.to_string();
        <String as sqlx::Encode<sqlx::Sqlite>>::encode(s, args)
    }
}

// 5. Teach SQLx how to read it from DB (DB -> String -> Enum)
impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for SessionStatus {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s: String = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        Ok(SessionStatus::from(s))
    }
}

// --- STRUCT DEFINITION ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuthenticationSession {
    #[sqlx(try_from = "String")]
    pub id: Uuid,

    #[sqlx(try_from = "String")]
    pub realm_id: Uuid,

    #[sqlx(try_from = "String")]
    pub flow_version_id: Uuid,

    pub current_node_id: String,

    #[sqlx(json)]
    pub context: Value,

    // No attribute needed since we implemented Type/Encode/Decode manually above
    pub status: SessionStatus,

    pub user_id: Option<Uuid>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub expires_at: DateTime<Utc>,
}

impl AuthenticationSession {
    pub fn new(realm_id: Uuid, flow_version_id: Uuid, start_node_id: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            realm_id,
            flow_version_id,
            current_node_id: start_node_id,
            context: serde_json::json!({}),
            status: SessionStatus::active,
            user_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::minutes(30),
        }
    }

    pub fn update_context(&mut self, key: &str, value: Value) {
        if let Value::Object(ref mut map) = self.context {
            map.insert(key.to_string(), value);
        }
    }
}
