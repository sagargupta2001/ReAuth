use crate::{
    domain::user::User,
    error::{Error, Result},
    ports::user_repository::UserRepository,
};
use async_trait::async_trait;
use sqlx::SqlitePool;

/// The SQLx "Adapter" for the UserRepository port.
pub struct SqliteUserRepository {
    pool: SqlitePool,
}

impl SqliteUserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as("SELECT * FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(user)
    }

    async fn save(&self, user: &User) -> Result<()> {
        sqlx::query("INSERT INTO users (id, username, role) VALUES (?, ?, ?)")
            .bind(&user.id)
            .bind(&user.username)
            .bind(&user.role)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }
}