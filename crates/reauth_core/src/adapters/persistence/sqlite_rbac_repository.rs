use std::collections::HashSet;
use crate::{
    domain::{group::Group, role::Role},
    error::{Error, Result},
    ports::rbac_repository::RbacRepository,
};
use async_trait::async_trait;
use uuid::Uuid;
use crate::adapters::persistence::connection::Database;
use crate::domain::role::Permission;

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

    async fn assign_role_to_group(&self, role_id: &Uuid, group_id: &Uuid) -> Result<()> {
        sqlx::query("INSERT INTO group_roles (group_id, role_id) VALUES (?, ?)")
            .bind(group_id.to_string())
            .bind(role_id.to_string())
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())    }

    async fn assign_user_to_group(&self, user_id: &Uuid, group_id: &Uuid) -> Result<()> {
        sqlx::query("INSERT INTO user_groups (user_id, group_id) VALUES (?, ?)")
            .bind(user_id.to_string())
            .bind(group_id.to_string())
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())    }

    async fn assign_permission_to_role(&self, permission: &Permission, role_id: &Uuid) -> Result<()> {
        sqlx::query("INSERT INTO role_permissions (role_id, permission_name) VALUES (?, ?)")
            .bind(role_id.to_string())
            .bind(permission)
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

    async fn find_group_by_name(&self, name: &str) -> Result<Option<Group>> {
        let group = sqlx::query_as("SELECT * FROM groups WHERE name = ?")
            .bind(name)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(group)
    }

    async fn find_user_ids_in_group(&self, group_id: &Uuid) -> Result<Vec<Uuid>> {
        // We query for a Vec of tuples, where each tuple contains one string (the user_id).
        let rows: Vec<(String,)> = sqlx::query_as("SELECT user_id FROM user_groups WHERE group_id = ?")
            .bind(group_id.to_string())
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        // Convert the Vec<String> into a Vec<Uuid>.
        let uuids = rows
            .into_iter()
            .filter_map(|(id,)| Uuid::parse_str(&id).ok()) // Safely parse, skipping any invalid IDs
            .collect();

        Ok(uuids)
    }

    async fn find_role_ids_for_user(&self, user_id: &Uuid) -> Result<Vec<Uuid>> {
        // Find all roles directly assigned to the user via groups
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT role_id FROM group_roles WHERE group_id IN (SELECT group_id FROM user_groups WHERE user_id = ?)"
        )
            .bind(user_id.to_string())
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        let uuids = rows
            .into_iter()
            .filter_map(|(id,)| Uuid::parse_str(&id).ok())
            .collect();
        Ok(uuids)    }

    async fn find_permissions_for_roles(&self, role_ids: &[Uuid]) -> Result<HashSet<Permission>> {
        // Convert Vec<Uuid> to Vec<String> for the JSON array
        let role_id_strings: Vec<String> = role_ids.iter().map(|id| id.to_string()).collect();
        // `sqlx` can't bind a `Vec` directly to a `json_each` function, so we pass a JSON array string.
        let json_array = serde_json::to_string(&role_id_strings)
            .map_err(|e| Error::Unexpected(e.into()))?;

        // This query recursively finds all child roles and then collects their permissions
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            WITH RECURSIVE effective_roles(id) AS (
                -- 1. Base case: The roles we are looking for
                SELECT value as id FROM json_each(?)
                UNION
                -- 2. Recursive step: Find all child roles
                SELECT rcr.child_role_id FROM role_composite_roles rcr
                JOIN effective_roles er ON rcr.parent_role_id = er.id
            )
            -- 3. Select all permissions for all roles found
            SELECT permission_name FROM role_permissions
            WHERE role_id IN (SELECT id FROM effective_roles)
            "#
        )
            .bind(json_array)
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        // Collect permissions into a HashSet to remove duplicates
        Ok(rows.into_iter().map(|(perm,)| perm).collect())    }

    async fn find_user_ids_for_role(&self, role_id: &Uuid) -> Result<Vec<Uuid>> {
        // This query finds all users who have a specific role,
        // including through composite roles.
        let rows: Vec<(String,)> = sqlx::query_as(r#"
            WITH RECURSIVE role_hierarchy(id) AS (
                -- 1. Base case: The role we are looking for (and its children)
                SELECT ? AS id
                UNION
                -- 2. Recursive step: Find all child roles
                SELECT rcr.child_role_id FROM role_composite_roles rcr
                JOIN role_hierarchy rh ON rcr.parent_role_id = rh.id
            )
            -- 3. Find all users in groups that have any of these roles
            SELECT DISTINCT ug.user_id FROM user_groups ug
            JOIN group_roles gr ON ug.group_id = gr.group_id
            WHERE gr.role_id IN (SELECT id FROM role_hierarchy)
        "#)
            .bind(role_id.to_string())
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        let uuids = rows
            .into_iter()
            .filter_map(|(id,)| Uuid::parse_str(&id).ok())
            .collect();
        Ok(uuids)
    }
}