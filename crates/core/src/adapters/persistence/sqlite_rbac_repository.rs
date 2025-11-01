use crate::{
    domain::{group::Group, role::Role},
    error::{Error, Result},
    ports::rbac_repository::RbacRepository,
};
use async_trait::async_trait;
use crate::adapters::persistence::connection::Database;

pub struct SqliteRbacRepository {
    pool: Database,
}

impl SqliteRbacRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RbacRepository for SqliteRbacRepository {
    async fn create_role(&self, role: &Role) -> Result<()> {
        sqlx::query("INSERT INTO roles (id, name, description) VALUES (?, ?, ?)")
            .bind(role.id.to_string())
            .bind(&role.name)
            .bind(&role.description)
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn find_role_by_name(&self, name: &str) -> Result<Option<Role>> {
        let role = sqlx::query_as("SELECT * FROM roles WHERE name = ?")
            .bind(name)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(role)
    }

    async fn create_group(&self, group: &Group) -> Result<()> {
        sqlx::query("INSERT INTO groups (id, name, description) VALUES (?, ?, ?)")
            .bind(group.id.to_string())
            .bind(&group.name)
            .bind(&group.description)
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn find_group_by_name(&self, name: &str) -> Result<Option<Group>> {
        let group = sqlx::query_as("SELECT * FROM groups WHERE name = ?")
            .bind(name)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(group)
    }
}