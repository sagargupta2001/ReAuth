use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct User {
    #[sqlx(try_from = "String")] // Convert TEXT from DB to Uuid
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub realm_id: Uuid,
    pub username: String,
    pub email: Option<String>,
    #[serde(skip_serializing)] // Don't send hash to UI
    pub hashed_password: String,
    #[serde(default)]
    #[sqlx(default)]
    pub force_password_reset: bool,
    #[serde(default)]
    #[sqlx(default)]
    pub password_login_disabled: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub last_sign_in_at: Option<DateTime<Utc>>,
}

impl User {
    pub fn new(realm_id: Uuid, username: String, hashed_password: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            realm_id,
            username,
            email: None,
            hashed_password,
            force_password_reset: false,
            password_login_disabled: false,
            created_at: Some(Utc::now()),
            last_sign_in_at: None,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.username.trim().is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        if self.username.len() < 3 {
            return Err("Username must be at least 3 characters long".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use sqlx::SqlitePool;
    use uuid::Uuid;

    #[tokio::test]
    async fn user_from_row_works() {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect");
        let id = Uuid::new_v4();
        let realm_id = Uuid::new_v4();

        let user: User = sqlx::query_as(
            "SELECT ? as id, ? as realm_id, ? as username, ? as email, ? as hashed_password, ? as force_password_reset, ? as password_login_disabled, ? as created_at, ? as last_sign_in_at",
        )
        .bind(id.to_string())
        .bind(realm_id.to_string())
        .bind("alice")
        .bind(None::<String>)
        .bind("hash")
        .bind(false)
        .bind(false)
        .bind(None::<DateTime<Utc>>)
        .bind(None::<DateTime<Utc>>)
        .fetch_one(&pool)
        .await
        .expect("fetch user");

        assert_eq!(user.id, id);
        assert_eq!(user.realm_id, realm_id);
        assert_eq!(user.username, "alice");
        assert!(user.email.is_none());
        assert_eq!(user.hashed_password, "hash");
        assert!(!user.force_password_reset);
        assert!(!user.password_login_disabled);
        assert!(user.created_at.is_none());
        assert!(user.last_sign_in_at.is_none());
    }

    #[test]
    fn user_serialization_skips_hashed_password() {
        let user = User {
            id: Uuid::new_v4(),
            realm_id: Uuid::new_v4(),
            username: "alice".to_string(),
            email: None,
            hashed_password: "hash".to_string(),
            force_password_reset: false,
            password_login_disabled: false,
            created_at: None,
            last_sign_in_at: None,
        };

        let value = serde_json::to_value(&user).expect("serialize");
        assert!(value.get("hashed_password").is_none());
    }

    #[test]
    fn user_deserializes_with_hashed_password() {
        let id = Uuid::new_v4();
        let realm_id = Uuid::new_v4();
        let value = json!({
            "id": id,
            "realm_id": realm_id,
            "username": "alice",
            "email": null,
            "hashed_password": "hash",
            "force_password_reset": true,
            "password_login_disabled": true,
            "created_at": null,
            "last_sign_in_at": null
        });

        let user: User = serde_json::from_value(value).expect("deserialize");

        assert_eq!(user.id, id);
        assert_eq!(user.realm_id, realm_id);
        assert_eq!(user.username, "alice");
        assert!(user.email.is_none());
        assert_eq!(user.hashed_password, "hash");
        assert!(user.force_password_reset);
        assert!(user.password_login_disabled);
    }

    #[test]
    fn user_new_generates_id() {
        let realm_id = Uuid::new_v4();
        let user = User::new(realm_id, "bob".to_string(), "hash".to_string());

        assert!(!user.id.is_nil());
        assert_eq!(user.realm_id, realm_id);
        assert_eq!(user.username, "bob");
        assert!(user.email.is_none());
        assert!(!user.force_password_reset);
        assert!(!user.password_login_disabled);
    }

    #[test]
    fn user_validation_works() {
        let realm_id = Uuid::new_v4();

        let valid_user = User::new(realm_id, "alice".to_string(), "hash".to_string());
        assert!(valid_user.validate().is_ok());

        let empty_user = User::new(realm_id, "".to_string(), "hash".to_string());
        assert!(empty_user.validate().is_err());

        let short_user = User::new(realm_id, "al".to_string(), "hash".to_string());
        assert!(short_user.validate().is_err());
    }
}
