use crate::{
    domain::user::User,
    error::{Error, Result},
    ports::user_repository::UserRepository,
};
use async_trait::async_trait;
use crate::adapters::persistence::connection::Database;

/// The SQLx "Adapter" for the UserRepository port.
pub struct SqliteUserRepository {
    // The struct now holds the shared pointer to the pool.
    pool: Database,
}

impl SqliteUserRepository {
    // The `new` function now correctly accepts the shared pool.
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as("SELECT * FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&*self.pool) // You can still use &self.pool here
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(user)
    }

    async fn save(&self, user: &User) -> Result<()> {
        sqlx::query("INSERT INTO users (id, username, hashed_password) VALUES (?, ?, ?)")
            .bind(user.id.to_string())
            .bind(&user.username)
            .bind(&user.hashed_password)
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }
}