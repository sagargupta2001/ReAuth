use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Active,
    Completed,
    Failed,
}

// 1. Display (Required for Encode)
impl fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionStatus::Active => write!(f, "active"),
            SessionStatus::Completed => write!(f, "completed"),
            SessionStatus::Failed => write!(f, "failed"),
        }
    }
}

// 2. From String (Required for Decode)
impl From<String> for SessionStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "completed" => SessionStatus::Completed,
            "failed" => SessionStatus::Failed,
            _ => SessionStatus::Active,
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
            status: SessionStatus::Active,
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

#[cfg(test)]
mod tests {
    use super::*;
    // use super::{AuthenticationSession, SessionStatus};
    use serde_json::json;
    use uuid::Uuid;

    #[test]
    fn authentication_session_new_sets_defaults() {
        let realm_id = Uuid::new_v4();
        let flow_id = Uuid::new_v4();
        let session = AuthenticationSession::new(realm_id, flow_id, "start".to_string());

        assert_eq!(session.realm_id, realm_id);
        assert_eq!(session.flow_version_id, flow_id);
        assert_eq!(session.current_node_id, "start");
        assert_eq!(session.status, SessionStatus::Active);
        assert!(session.user_id.is_none());
        assert!(session.context.as_object().unwrap().is_empty());
    }

    #[test]
    fn authentication_session_update_context_inserts_value() {
        let mut session =
            AuthenticationSession::new(Uuid::new_v4(), Uuid::new_v4(), "start".to_string());

        session.update_context("key", json!("value"));

        assert_eq!(session.context.get("key").unwrap(), "value");
    }

    #[test]
    fn session_status_display_and_from_string() {
        assert_eq!(SessionStatus::Active.to_string(), "active");
        assert_eq!(SessionStatus::Completed.to_string(), "completed");
        assert_eq!(SessionStatus::Failed.to_string(), "failed");

        assert!(matches!(
            SessionStatus::from("completed".to_string()),
            SessionStatus::Completed
        ));
        assert!(matches!(
            SessionStatus::from("failed".to_string()),
            SessionStatus::Failed
        ));
        assert!(matches!(
            SessionStatus::from("other".to_string()),
            SessionStatus::Active
        ));
    }
}
