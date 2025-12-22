use crate::adapters::persistence::connection::Database;
use crate::domain::role::Permission;
use crate::{
    domain::{group::Group, role::Role},
    error::{Error, Result},
    ports::rbac_repository::RbacRepository,
};
use async_trait::async_trait;
use std::collections::HashSet;
use sqlx::{QueryBuilder, Sqlite};
use uuid::Uuid;
use crate::domain::pagination::{PageRequest, PageResponse, SortDirection};

pub struct SqliteRbacRepository {
    pool: Database,
}

impl SqliteRbacRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }

    fn apply_filters<'a>(
        builder: &mut QueryBuilder<'a, Sqlite>,
        realm_id: &Uuid,
        q: &Option<String>,
    ) {
        builder.push(" WHERE realm_id = ");
        builder.push_bind(realm_id.to_string());

        if let Some(query) = q {
            if !query.trim().is_empty() {
                builder.push(" AND (name LIKE ");
                builder.push_bind(format!("%{}%", query));
                builder.push(" OR description LIKE ");
                builder.push_bind(format!("%{}%", query));
                builder.push(")");
            }
        }
    }
}

#[async_trait]
impl RbacRepository for SqliteRbacRepository {
    async fn create_role(&self, role: &Role) -> Result<()> {
        sqlx::query("INSERT INTO roles (id, realm_id, name, description) VALUES (?, ?, ?, ?)")
            .bind(role.id.to_string())
            .bind(role.realm_id.to_string()) // [NEW] Realm Scope
            .bind(&role.name)
            .bind(&role.description)
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn create_group(&self, group: &Group) -> Result<()> {
        sqlx::query("INSERT INTO groups (id, realm_id, name, description) VALUES (?, ?, ?, ?)")
            .bind(group.id.to_string())
            .bind(group.realm_id.to_string()) // [NEW] Realm Scope
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
        Ok(())
    }

    async fn assign_user_to_group(&self, user_id: &Uuid, group_id: &Uuid) -> Result<()> {
        sqlx::query("INSERT INTO user_groups (user_id, group_id) VALUES (?, ?)")
            .bind(user_id.to_string())
            .bind(group_id.to_string())
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn assign_permission_to_role(
        &self,
        permission: &Permission,
        role_id: &Uuid,
    ) -> Result<()> {
        sqlx::query("INSERT INTO role_permissions (role_id, permission_name) VALUES (?, ?)")
            .bind(role_id.to_string())
            .bind(permission)
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn assign_role_to_user(&self, user_id: &Uuid, role_id: &Uuid) -> Result<()> {
        // We use the `user_roles` table created in Phase 1 migration
        sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES (?, ?)")
            .bind(user_id.to_string())
            .bind(role_id.to_string())
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    // [Helper] Check if a role exists in a specific realm
    async fn find_role_by_name(&self, realm_id: &Uuid, name: &str) -> Result<Option<Role>> {
        let role = sqlx::query_as("SELECT * FROM roles WHERE realm_id = ? AND name = ?")
            .bind(realm_id.to_string())
            .bind(name)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(role)
    }

    async fn find_group_by_name(&self, realm_id: &Uuid, name: &str) -> Result<Option<Group>> {
        let group = sqlx::query_as("SELECT * FROM groups WHERE realm_id = ? AND name = ?")
            .bind(realm_id.to_string())
            .bind(name)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(group)
    }

    async fn list_roles(
        &self,
        realm_id: &Uuid,
        req: &PageRequest,
    ) -> Result<PageResponse<Role>> {
        let limit = req.per_page.clamp(1, 100);
        let offset = (req.page - 1) * limit;

        // 1. Count Query
        let mut count_builder = QueryBuilder::new("SELECT COUNT(*) FROM roles");
        Self::apply_filters(&mut count_builder, realm_id, &req.q);

        let total: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        // 2. Select Query
        let mut query_builder = QueryBuilder::new("SELECT * FROM roles");
        Self::apply_filters(&mut query_builder, realm_id, &req.q);

        // Sorting
        // Map API sort keys to Safe Database Columns
        let sort_col = match req.sort_by.as_deref() {
            Some("name") => "name",
            Some("description") => "description",
            Some("created_at") => "created_at",
            _ => "name", // Default sort
        };

        let sort_dir = match req.sort_dir.unwrap_or(SortDirection::Asc) {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };

        // Push ORDER BY directly (safe because we matched against string literals above)
        query_builder.push(format!(" ORDER BY {} {}", sort_col, sort_dir));

        // Pagination
        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        // Execute
        let roles: Vec<Role> = query_builder
            .build_query_as()
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(PageResponse::new(roles, total, req.page, limit))
    }

    async fn find_role_by_id(&self, role_id: &Uuid) -> Result<Option<Role>> {
        let role = sqlx::query_as("SELECT * FROM roles WHERE id = ?")
            .bind(role_id.to_string())
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(role)
    }
    async fn find_groups_by_realm(&self, realm_id: &Uuid) -> Result<Vec<Group>> {
        let groups = sqlx::query_as("SELECT * FROM groups WHERE realm_id = ? ORDER BY name ASC")
            .bind(realm_id.to_string())
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(groups)
    }

    async fn find_user_ids_in_group(&self, group_id: &Uuid) -> Result<Vec<Uuid>> {
        // We query for a Vec of tuples, where each tuple contains one string (the user_id).
        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT user_id FROM user_groups WHERE group_id = ?")
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
        Ok(uuids)
    }

    async fn find_permissions_for_roles(&self, role_ids: &[Uuid]) -> Result<HashSet<Permission>> {
        // Convert Vec<Uuid> to Vec<String> for the JSON array
        let role_id_strings: Vec<String> = role_ids.iter().map(|id| id.to_string()).collect();
        // `sqlx` can't bind a `Vec` directly to a `json_each` function, so we pass a JSON array string.
        let json_array =
            serde_json::to_string(&role_id_strings).map_err(|e| Error::Unexpected(e.into()))?;

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
            "#,
        )
        .bind(json_array)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        // Collect permissions into a HashSet to remove duplicates
        Ok(rows.into_iter().map(|(perm,)| perm).collect())
    }

    async fn find_user_ids_for_role(&self, role_id: &Uuid) -> Result<Vec<Uuid>> {
        // This query finds all users who have a specific role,
        // including through composite roles.
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
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
        "#,
        )
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

    async fn get_effective_permissions_for_user(&self, user_id: &Uuid) -> Result<HashSet<String>> {
        let user_id_str = user_id.to_string();

        // RECURSIVE CTE EXPLANATION:
        // 1. user_direct_roles: Finds roles assigned directly to user.
        // 2. user_group_roles: Finds roles assigned to user's groups.
        // 3. all_effective_roles: Recursively walks up the tree (Child -> Parent).
        // 4. Final Select: Grabs permissions for ALL those roles.

        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            WITH RECURSIVE
            -- 1. Roots: Get all roles the user explicitly has (Direct or via Group)
            base_roles(role_id) AS (
                SELECT role_id FROM user_roles WHERE user_id = ?
                UNION
                SELECT gr.role_id
                FROM group_roles gr
                JOIN user_groups ug ON gr.group_id = ug.group_id
                WHERE ug.user_id = ?
            ),
            -- 2. Recursion: Find all parent roles (Inheritance)
            -- If Role A inherits Role B, and user has Role A, they also get Role B.
            all_effective_roles(role_id) AS (
                SELECT role_id FROM base_roles
                UNION
                SELECT rcr.child_role_id
                FROM role_composite_roles rcr
                JOIN all_effective_roles ar ON rcr.parent_role_id = ar.role_id
            )
            -- 3. Resolve: Get permissions for everything found
            SELECT DISTINCT p.permission_name
            FROM role_permissions p
            JOIN all_effective_roles ar ON p.role_id = ar.role_id
        "#,
        )
        .bind(&user_id_str)
        .bind(&user_id_str)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows.into_iter().map(|(p,)| p).collect())
    }

    async fn find_role_names_for_user(&self, user_id: &Uuid) -> Result<Vec<String>> {
        // We reuse the recursive logic or just join tables if composite roles aren't needed in token names
        // Usually, Access Tokens contain *Effective* Roles (flattened).
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            WITH RECURSIVE effective_roles(id) AS (
                SELECT role_id FROM user_roles WHERE user_id = ?
                UNION
                SELECT gr.role_id
                FROM group_roles gr JOIN user_groups ug ON gr.group_id = ug.group_id
                WHERE ug.user_id = ?
                UNION
                SELECT rcr.child_role_id FROM role_composite_roles rcr
                JOIN effective_roles er ON rcr.parent_role_id = er.id
            )
            SELECT DISTINCT r.name
            FROM roles r
            JOIN effective_roles er ON r.id = er.id
        "#,
        )
        .bind(user_id.to_string())
        .bind(user_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows.into_iter().map(|(n,)| n).collect())
    }

    async fn find_group_names_for_user(&self, user_id: &Uuid) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT g.name
            FROM groups g
            JOIN user_groups ug ON g.id = ug.group_id
            WHERE ug.user_id = ?
        "#,
        )
        .bind(user_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows.into_iter().map(|(n,)| n).collect())
    }

    async fn delete_role(&self, role_id: &Uuid) -> Result<()> {
        // Simple delete. The database FK constraints (ON DELETE CASCADE)
        // will automatically clean up role_permissions, user_roles, etc.
        let result = sqlx::query("DELETE FROM roles WHERE id = ?")
            .bind(role_id.to_string())
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound("Role not found for deletion".into()));
        }

        Ok(())
    }
}
