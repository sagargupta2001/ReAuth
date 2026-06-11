use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct UserPhoneNumber {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub user_id: Uuid,
    #[sqlx(try_from = "String")]
    pub realm_id: Uuid,
    pub phone_number: String,
    pub phone_number_normalized: String,
    pub is_primary: bool,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserPhoneNumber {
    pub fn new(
        user_id: Uuid,
        realm_id: Uuid,
        phone_number: String,
        is_primary: bool,
        is_verified: bool,
    ) -> Self {
        let phone_number = phone_number.trim().to_string();
        Self {
            id: Uuid::new_v4(),
            user_id,
            realm_id,
            phone_number_normalized: normalize_phone_number(&phone_number),
            phone_number,
            is_primary,
            is_verified,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

pub fn normalize_phone_number(phone_number: &str) -> String {
    phone_number
        .trim()
        .chars()
        .filter(|character| character.is_ascii_digit() || *character == '+')
        .collect()
}
