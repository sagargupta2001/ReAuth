use crate::database::Database;
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub role: String,
}

pub struct UserRepository;

impl UserRepository {
    pub async fn get_user(db: &Database, id: &str) -> anyhow::Result<Option<User>> {
        let row = sqlx::query("SELECT id, username, role FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(&**db)
            .await?;

        if let Some(row) = row {
            Ok(Some(User {
                id: row.get("id"),
                username: row.get("username"),
                role: row.get("role"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn create_user(db: &Database, user: &User) -> anyhow::Result<User> {
        sqlx::query("INSERT INTO users (id, username, role) VALUES (?, ?, ?)")
            .bind(&user.id)
            .bind(&user.username)
            .bind(&user.role)
            .execute(&**db)
            .await?;

        Ok(user.clone())
    }
}
