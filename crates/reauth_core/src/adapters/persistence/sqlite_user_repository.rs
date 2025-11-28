use crate::adapters::persistence::connection::Database;
use crate::domain::pagination::{PageRequest, PageResponse, SortDirection};
use crate::{
    domain::user::User,
    error::{Error, Result},
    ports::user_repository::UserRepository,
};
use async_trait::async_trait;
use sqlx::{QueryBuilder, Sqlite};
use uuid::Uuid;

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

    // Helper to keep count and select logic in sync
    fn apply_filters<'a>(builder: &mut QueryBuilder<'a, Sqlite>, q: &Option<String>) {
        if let Some(query_text) = q {
            if !query_text.is_empty() {
                builder.push(" WHERE username LIKE ");
                builder.push_bind(format!("%{}%", query_text));
            }
        }
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

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as("SELECT * FROM users WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&*self.pool)
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

    async fn update(&self, user: &User) -> Result<()> {
        sqlx::query("UPDATE users SET username = ?, hashed_password = ? WHERE id = ?")
            .bind(&user.username)
            .bind(&user.hashed_password)
            .bind(user.id.to_string())
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn list(&self, req: &PageRequest) -> Result<PageResponse<User>> {
        let limit = req.per_page.clamp(1, 100);
        let offset = (req.page - 1) * limit;

        // 1. Count
        let mut count_builder = QueryBuilder::new("SELECT COUNT(*) FROM users");
        Self::apply_filters(&mut count_builder, &req.q);
        let total: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        // 2. Select
        let mut query_builder = QueryBuilder::new("SELECT * FROM users");
        Self::apply_filters(&mut query_builder, &req.q);

        // Sorting
        let sort_col = match req.sort_by.as_deref() {
            Some("username") => "username",
            _ => "username",
        };
        let sort_dir = match req.sort_dir.unwrap_or(SortDirection::Asc) {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };
        query_builder.push(format!(" ORDER BY {} {}", sort_col, sort_dir));

        // Pagination
        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        let users: Vec<User> = query_builder
            .build_query_as()
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(PageResponse::new(users, total, req.page, limit))
    }
}
