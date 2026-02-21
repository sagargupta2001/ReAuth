use crate::adapters::persistence::connection::Database;
use crate::domain::pagination::{PageRequest, PageResponse, SortDirection};
use crate::domain::rbac::{
    CustomPermission, GroupMemberFilter, GroupMemberRow, GroupRoleFilter, GroupRoleRow,
    GroupTreeRow, RoleCompositeFilter, RoleCompositeRow, RoleMemberFilter, RoleMemberRow,
    UserRoleFilter, UserRoleRow,
};
use crate::domain::role::Permission;
use crate::{
    domain::{group::Group, role::Role},
    error::{Error, Result},
    ports::rbac_repository::RbacRepository,
};
use async_trait::async_trait;
use sqlx::{QueryBuilder, Sqlite};
use std::collections::HashSet;
use uuid::Uuid;

pub struct SqliteRbacRepository {
    pool: Database,
}

impl SqliteRbacRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }

    fn apply_group_tree_filters<'a>(
        builder: &mut QueryBuilder<'a, Sqlite>,
        realm_id: &Uuid,
        parent_id: Option<&Uuid>,
        q: &Option<String>,
    ) {
        builder.push(" WHERE g.realm_id = ");
        builder.push_bind(realm_id.to_string());

        match parent_id {
            Some(pid) => {
                builder.push(" AND g.parent_id = ");
                builder.push_bind(pid.to_string());
            }
            None => {
                builder.push(" AND g.parent_id IS NULL");
            }
        }

        if let Some(query) = q {
            if !query.trim().is_empty() {
                builder.push(" AND (g.name LIKE ");
                builder.push_bind(format!("%{}%", query));
                builder.push(" OR g.description LIKE ");
                builder.push_bind(format!("%{}%", query));
                builder.push(")");
            }
        }
    }

    fn apply_filters<'a>(
        builder: &mut QueryBuilder<'a, Sqlite>,
        realm_id: &Uuid,
        client_filter: Option<&Uuid>, // None = Global, Some = Client Specific
        q: &Option<String>,
    ) {
        builder.push(" WHERE realm_id = ");
        builder.push_bind(realm_id.to_string());

        // [LOGIC] Switch based on filter
        match client_filter {
            Some(cid) => {
                builder.push(" AND client_id = ");
                builder.push_bind(cid.to_string());
            }
            None => {
                builder.push(" AND client_id IS NULL ");
            }
        }

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

    fn apply_group_filters<'a>(
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
        // [UPDATED] Added client_id to INSERT
        sqlx::query(
            "INSERT INTO roles (id, realm_id, client_id, name, description) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(role.id.to_string())
        .bind(role.realm_id.to_string())
        .bind(role.client_id.map(|id| id.to_string()))
        .bind(&role.name)
        .bind(&role.description)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn create_group(&self, group: &Group) -> Result<()> {
        sqlx::query(
            "INSERT INTO groups (id, realm_id, parent_id, name, description, sort_order) VALUES (?, ?, ?, ?, ?, ?)"
        )
            .bind(group.id.to_string())
            .bind(group.realm_id.to_string()) // [NEW] Realm Scope
            .bind(group.parent_id.map(|id| id.to_string()))
            .bind(&group.name)
            .bind(&group.description)
            .bind(group.sort_order)
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn assign_role_to_group(&self, role_id: &Uuid, group_id: &Uuid) -> Result<()> {
        sqlx::query("INSERT OR IGNORE INTO group_roles (group_id, role_id) VALUES (?, ?)")
            .bind(group_id.to_string())
            .bind(role_id.to_string())
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn remove_role_from_group(&self, role_id: &Uuid, group_id: &Uuid) -> Result<()> {
        sqlx::query("DELETE FROM group_roles WHERE group_id = ? AND role_id = ?")
            .bind(group_id.to_string())
            .bind(role_id.to_string())
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn assign_user_to_group(&self, user_id: &Uuid, group_id: &Uuid) -> Result<()> {
        sqlx::query("INSERT OR IGNORE INTO user_groups (user_id, group_id) VALUES (?, ?)")
            .bind(user_id.to_string())
            .bind(group_id.to_string())
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn remove_user_from_group(&self, user_id: &Uuid, group_id: &Uuid) -> Result<()> {
        sqlx::query("DELETE FROM user_groups WHERE user_id = ? AND group_id = ?")
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
        sqlx::query("INSERT OR IGNORE INTO user_roles (user_id, role_id) VALUES (?, ?)")
            .bind(user_id.to_string())
            .bind(role_id.to_string())
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn remove_role_from_user(&self, user_id: &Uuid, role_id: &Uuid) -> Result<()> {
        sqlx::query("DELETE FROM user_roles WHERE user_id = ? AND role_id = ?")
            .bind(user_id.to_string())
            .bind(role_id.to_string())
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn create_custom_permission(&self, permission: &CustomPermission) -> Result<()> {
        sqlx::query(
            "INSERT INTO custom_permissions (id, realm_id, client_id, permission, name, description, created_by) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(permission.id.to_string())
        .bind(permission.realm_id.to_string())
        .bind(permission.client_id.map(|id| id.to_string()))
        .bind(&permission.permission)
        .bind(&permission.name)
        .bind(&permission.description)
        .bind(permission.created_by.map(|id| id.to_string()))
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn update_custom_permission(&self, permission: &CustomPermission) -> Result<()> {
        sqlx::query(
            "UPDATE custom_permissions SET name = ?, description = ?, updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) WHERE id = ?",
        )
        .bind(&permission.name)
        .bind(&permission.description)
        .bind(permission.id.to_string())
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn delete_custom_permission(&self, permission_id: &Uuid) -> Result<()> {
        sqlx::query("DELETE FROM custom_permissions WHERE id = ?")
            .bind(permission_id.to_string())
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn find_custom_permission_by_key(
        &self,
        realm_id: &Uuid,
        client_id: Option<&Uuid>,
        permission: &str,
    ) -> Result<Option<CustomPermission>> {
        let mut query_builder =
            QueryBuilder::new("SELECT * FROM custom_permissions WHERE realm_id = ");
        query_builder.push_bind(realm_id.to_string());
        query_builder.push(" AND permission = ");
        query_builder.push_bind(permission);

        match client_id {
            Some(id) => {
                query_builder.push(" AND client_id = ");
                query_builder.push_bind(id.to_string());
            }
            None => {
                query_builder.push(" AND client_id IS NULL");
            }
        }

        let permission = query_builder
            .build_query_as()
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(permission)
    }

    async fn find_custom_permission_by_id(
        &self,
        realm_id: &Uuid,
        permission_id: &Uuid,
    ) -> Result<Option<CustomPermission>> {
        let permission =
            sqlx::query_as("SELECT * FROM custom_permissions WHERE realm_id = ? AND id = ?")
                .bind(realm_id.to_string())
                .bind(permission_id.to_string())
                .fetch_optional(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(permission)
    }

    async fn list_custom_permissions(
        &self,
        realm_id: &Uuid,
        client_id: Option<&Uuid>,
    ) -> Result<Vec<CustomPermission>> {
        let mut query_builder =
            QueryBuilder::new("SELECT * FROM custom_permissions WHERE realm_id = ");
        query_builder.push_bind(realm_id.to_string());

        match client_id {
            Some(id) => {
                query_builder.push(" AND client_id = ");
                query_builder.push_bind(id.to_string());
            }
            None => {
                query_builder.push(" AND client_id IS NULL");
            }
        }

        query_builder.push(" ORDER BY permission ASC");

        let permissions: Vec<CustomPermission> = query_builder
            .build_query_as()
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(permissions)
    }

    async fn remove_role_permissions_by_key(&self, permission: &str) -> Result<()> {
        sqlx::query("DELETE FROM role_permissions WHERE permission_name = ?")
            .bind(permission)
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    // [Helper] Check if a role exists in a specific realm
    async fn find_role_by_name(&self, realm_id: &Uuid, name: &str) -> Result<Option<Role>> {
        let role = sqlx::query_as(
            "SELECT * FROM roles WHERE realm_id = ? AND name = ? AND client_id IS NULL",
        )
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

    async fn find_group_by_id(&self, group_id: &Uuid) -> Result<Option<Group>> {
        let group = sqlx::query_as("SELECT * FROM groups WHERE id = ?")
            .bind(group_id.to_string())
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(group)
    }

    async fn list_roles(&self, realm_id: &Uuid, req: &PageRequest) -> Result<PageResponse<Role>> {
        let limit = req.per_page.clamp(1, 100);
        let offset = (req.page - 1) * limit;

        // 1. Count Query
        let mut count_builder = QueryBuilder::new("SELECT COUNT(*) FROM roles");
        Self::apply_filters(&mut count_builder, realm_id, None, &req.q);

        let total: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        // 2. Select Query
        let mut query_builder = QueryBuilder::new("SELECT * FROM roles");
        Self::apply_filters(&mut query_builder, realm_id, None, &req.q);

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

    async fn list_client_roles(
        &self,
        realm_id: &Uuid,
        client_id: &Uuid,
        req: &PageRequest,
    ) -> Result<PageResponse<Role>> {
        let limit = req.per_page.clamp(1, 100);
        let offset = (req.page - 1) * limit;

        let mut count_builder = QueryBuilder::new("SELECT COUNT(*) FROM roles");
        // Pass `Some(client_id)` to filter by client
        Self::apply_filters(&mut count_builder, realm_id, Some(client_id), &req.q);

        let total: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        let mut query_builder = QueryBuilder::new("SELECT * FROM roles");
        Self::apply_filters(&mut query_builder, realm_id, Some(client_id), &req.q);

        // Sorting (Copy sorting logic from list_roles)
        let sort_col = match req.sort_by.as_deref() {
            Some("name") => "name",
            _ => "name",
        };
        let sort_dir = match req.sort_dir.unwrap_or(SortDirection::Asc) {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };
        query_builder.push(format!(" ORDER BY {} {}", sort_col, sort_dir));

        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

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

    async fn list_groups(&self, realm_id: &Uuid, req: &PageRequest) -> Result<PageResponse<Group>> {
        let limit = req.per_page.clamp(1, 100);
        let offset = (req.page - 1) * limit;

        let mut count_builder = QueryBuilder::new("SELECT COUNT(*) FROM groups");
        Self::apply_group_filters(&mut count_builder, realm_id, &req.q);
        let total: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        let mut query_builder = QueryBuilder::new("SELECT * FROM groups");
        Self::apply_group_filters(&mut query_builder, realm_id, &req.q);

        let sort_col = match req.sort_by.as_deref() {
            Some("name") => "name",
            _ => "name",
        };
        let sort_dir = match req.sort_dir.unwrap_or(SortDirection::Asc) {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };
        query_builder.push(format!(" ORDER BY {} {}", sort_col, sort_dir));

        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        let groups: Vec<Group> = query_builder
            .build_query_as()
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(PageResponse::new(groups, total, req.page, limit))
    }

    async fn list_group_roots(
        &self,
        realm_id: &Uuid,
        req: &PageRequest,
    ) -> Result<PageResponse<GroupTreeRow>> {
        let limit = req.per_page.clamp(1, 100);
        let offset = (req.page - 1) * limit;

        let mut count_builder = QueryBuilder::new("SELECT COUNT(*) FROM groups g");
        Self::apply_group_tree_filters(&mut count_builder, realm_id, None, &req.q);
        let total: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        let mut query_builder = QueryBuilder::new(
            "SELECT g.id, g.parent_id, g.name, g.description, g.sort_order, \
             EXISTS (SELECT 1 FROM groups c WHERE c.parent_id = g.id) AS has_children \
             FROM groups g",
        );
        Self::apply_group_tree_filters(&mut query_builder, realm_id, None, &req.q);

        let sort_col = match req.sort_by.as_deref() {
            Some("name") => "g.name",
            Some("sort_order") => "g.sort_order",
            _ => "g.sort_order",
        };
        let sort_dir = match req.sort_dir.unwrap_or(SortDirection::Asc) {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };
        query_builder.push(format!(" ORDER BY {} {}", sort_col, sort_dir));
        if sort_col != "g.name" {
            query_builder.push(", g.name ASC");
        } else {
            query_builder.push(", g.sort_order ASC");
        }

        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        let groups: Vec<GroupTreeRow> = query_builder
            .build_query_as()
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(PageResponse::new(groups, total, req.page, limit))
    }

    async fn list_group_children(
        &self,
        realm_id: &Uuid,
        parent_id: &Uuid,
        req: &PageRequest,
    ) -> Result<PageResponse<GroupTreeRow>> {
        let limit = req.per_page.clamp(1, 100);
        let offset = (req.page - 1) * limit;

        let mut count_builder = QueryBuilder::new("SELECT COUNT(*) FROM groups g");
        Self::apply_group_tree_filters(&mut count_builder, realm_id, Some(parent_id), &req.q);
        let total: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        let mut query_builder = QueryBuilder::new(
            "SELECT g.id, g.parent_id, g.name, g.description, g.sort_order, \
             EXISTS (SELECT 1 FROM groups c WHERE c.parent_id = g.id) AS has_children \
             FROM groups g",
        );
        Self::apply_group_tree_filters(&mut query_builder, realm_id, Some(parent_id), &req.q);

        let sort_col = match req.sort_by.as_deref() {
            Some("name") => "g.name",
            Some("sort_order") => "g.sort_order",
            _ => "g.sort_order",
        };
        let sort_dir = match req.sort_dir.unwrap_or(SortDirection::Asc) {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };
        query_builder.push(format!(" ORDER BY {} {}", sort_col, sort_dir));
        if sort_col != "g.name" {
            query_builder.push(", g.name ASC");
        } else {
            query_builder.push(", g.sort_order ASC");
        }

        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        let groups: Vec<GroupTreeRow> = query_builder
            .build_query_as()
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(PageResponse::new(groups, total, req.page, limit))
    }

    async fn list_role_members(
        &self,
        realm_id: &Uuid,
        role_id: &Uuid,
        filter: RoleMemberFilter,
        req: &PageRequest,
    ) -> Result<PageResponse<RoleMemberRow>> {
        let limit = req.per_page.clamp(1, 100);
        let offset = (req.page - 1) * limit;

        // Count
        let mut count_builder = QueryBuilder::new("");
        count_builder.push("WITH RECURSIVE role_hierarchy(id) AS ( SELECT ");
        count_builder.push_bind(role_id.to_string());
        count_builder.push(
            " AS id UNION SELECT rcr.child_role_id FROM role_composite_roles rcr JOIN role_hierarchy rh ON rcr.parent_role_id = rh.id ),",
        );
        count_builder.push(" direct_users AS (SELECT user_id FROM user_roles WHERE role_id = ");
        count_builder.push_bind(role_id.to_string());
        count_builder.push("), effective_users AS (");
        count_builder.push(
            "SELECT user_id FROM user_roles WHERE role_id IN (SELECT id FROM role_hierarchy) ",
        );
        count_builder.push("UNION SELECT ug.user_id FROM user_groups ug JOIN group_roles gr ON ug.group_id = gr.group_id ");
        count_builder.push("WHERE gr.role_id IN (SELECT id FROM role_hierarchy)) ");
        count_builder.push("SELECT COUNT(*) FROM users u ");
        count_builder.push("LEFT JOIN direct_users du ON u.id = du.user_id ");
        count_builder.push("LEFT JOIN effective_users eu ON u.id = eu.user_id ");
        count_builder.push("WHERE u.realm_id = ");
        count_builder.push_bind(realm_id.to_string());

        if let Some(q) = &req.q {
            if !q.trim().is_empty() {
                count_builder.push(" AND u.username LIKE ");
                count_builder.push_bind(format!("%{}%", q));
            }
        }

        match filter {
            RoleMemberFilter::All => {}
            RoleMemberFilter::Direct => {
                count_builder.push(" AND du.user_id IS NOT NULL");
            }
            RoleMemberFilter::Effective => {
                count_builder.push(" AND du.user_id IS NULL AND eu.user_id IS NOT NULL");
            }
            RoleMemberFilter::Unassigned => {
                count_builder.push(" AND eu.user_id IS NULL");
            }
        }

        let total: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        // Select
        let mut query_builder = QueryBuilder::new("");
        query_builder.push("WITH RECURSIVE role_hierarchy(id) AS ( SELECT ");
        query_builder.push_bind(role_id.to_string());
        query_builder.push(
            " AS id UNION SELECT rcr.child_role_id FROM role_composite_roles rcr JOIN role_hierarchy rh ON rcr.parent_role_id = rh.id ),",
        );
        query_builder.push(" direct_users AS (SELECT user_id FROM user_roles WHERE role_id = ");
        query_builder.push_bind(role_id.to_string());
        query_builder.push("), effective_users AS (");
        query_builder.push(
            "SELECT user_id FROM user_roles WHERE role_id IN (SELECT id FROM role_hierarchy) ",
        );
        query_builder.push("UNION SELECT ug.user_id FROM user_groups ug JOIN group_roles gr ON ug.group_id = gr.group_id ");
        query_builder.push("WHERE gr.role_id IN (SELECT id FROM role_hierarchy)) ");
        query_builder.push("SELECT u.id, u.username, ");
        query_builder.push("CASE WHEN du.user_id IS NOT NULL THEN 1 ELSE 0 END AS is_direct, ");
        query_builder.push("CASE WHEN eu.user_id IS NOT NULL THEN 1 ELSE 0 END AS is_effective ");
        query_builder.push("FROM users u ");
        query_builder.push("LEFT JOIN direct_users du ON u.id = du.user_id ");
        query_builder.push("LEFT JOIN effective_users eu ON u.id = eu.user_id ");
        query_builder.push("WHERE u.realm_id = ");
        query_builder.push_bind(realm_id.to_string());

        if let Some(q) = &req.q {
            if !q.trim().is_empty() {
                query_builder.push(" AND u.username LIKE ");
                query_builder.push_bind(format!("%{}%", q));
            }
        }

        match filter {
            RoleMemberFilter::All => {}
            RoleMemberFilter::Direct => {
                query_builder.push(" AND du.user_id IS NOT NULL");
            }
            RoleMemberFilter::Effective => {
                query_builder.push(" AND du.user_id IS NULL AND eu.user_id IS NOT NULL");
            }
            RoleMemberFilter::Unassigned => {
                query_builder.push(" AND eu.user_id IS NULL");
            }
        }

        let sort_col = match req.sort_by.as_deref() {
            Some("username") => "u.username",
            Some("id") => "u.id",
            _ => "u.username",
        };
        let sort_dir = match req.sort_dir.unwrap_or(SortDirection::Asc) {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };
        query_builder.push(format!(" ORDER BY {} {}", sort_col, sort_dir));

        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        let users: Vec<RoleMemberRow> = query_builder
            .build_query_as()
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(PageResponse::new(users, total, req.page, limit))
    }

    async fn list_group_members(
        &self,
        realm_id: &Uuid,
        group_id: &Uuid,
        filter: GroupMemberFilter,
        req: &PageRequest,
    ) -> Result<PageResponse<GroupMemberRow>> {
        let limit = req.per_page.clamp(1, 100);
        let offset = (req.page - 1) * limit;

        let mut count_builder = QueryBuilder::new("");
        count_builder.push("WITH members AS (SELECT user_id FROM user_groups WHERE group_id = ");
        count_builder.push_bind(group_id.to_string());
        count_builder.push(") ");
        count_builder.push("SELECT COUNT(*) FROM users u ");
        count_builder.push("LEFT JOIN members m ON u.id = m.user_id ");
        count_builder.push("WHERE u.realm_id = ");
        count_builder.push_bind(realm_id.to_string());

        if let Some(q) = &req.q {
            if !q.trim().is_empty() {
                count_builder.push(" AND u.username LIKE ");
                count_builder.push_bind(format!("%{}%", q));
            }
        }

        match filter {
            GroupMemberFilter::All => {}
            GroupMemberFilter::Members => {
                count_builder.push(" AND m.user_id IS NOT NULL");
            }
            GroupMemberFilter::NonMembers => {
                count_builder.push(" AND m.user_id IS NULL");
            }
        }

        let total: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        let mut query_builder = QueryBuilder::new("");
        query_builder.push("WITH members AS (SELECT user_id FROM user_groups WHERE group_id = ");
        query_builder.push_bind(group_id.to_string());
        query_builder.push(") ");
        query_builder.push("SELECT u.id, u.username, ");
        query_builder.push("CASE WHEN m.user_id IS NOT NULL THEN 1 ELSE 0 END AS is_member ");
        query_builder.push("FROM users u ");
        query_builder.push("LEFT JOIN members m ON u.id = m.user_id ");
        query_builder.push("WHERE u.realm_id = ");
        query_builder.push_bind(realm_id.to_string());

        if let Some(q) = &req.q {
            if !q.trim().is_empty() {
                query_builder.push(" AND u.username LIKE ");
                query_builder.push_bind(format!("%{}%", q));
            }
        }

        match filter {
            GroupMemberFilter::All => {}
            GroupMemberFilter::Members => {
                query_builder.push(" AND m.user_id IS NOT NULL");
            }
            GroupMemberFilter::NonMembers => {
                query_builder.push(" AND m.user_id IS NULL");
            }
        }

        let sort_col = match req.sort_by.as_deref() {
            Some("username") => "u.username",
            Some("id") => "u.id",
            _ => "u.username",
        };
        let sort_dir = match req.sort_dir.unwrap_or(SortDirection::Asc) {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };
        query_builder.push(format!(" ORDER BY {} {}", sort_col, sort_dir));

        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        let users: Vec<GroupMemberRow> = query_builder
            .build_query_as()
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(PageResponse::new(users, total, req.page, limit))
    }

    async fn list_group_roles(
        &self,
        realm_id: &Uuid,
        group_id: &Uuid,
        filter: GroupRoleFilter,
        req: &PageRequest,
    ) -> Result<PageResponse<GroupRoleRow>> {
        let limit = req.per_page.clamp(1, 100);
        let offset = (req.page - 1) * limit;

        let mut count_builder = QueryBuilder::new("");
        count_builder
            .push("WITH direct_roles AS (SELECT role_id FROM group_roles WHERE group_id = ");
        count_builder.push_bind(group_id.to_string());
        count_builder.push("), role_hierarchy(id) AS (");
        count_builder.push("SELECT role_id FROM direct_roles ");
        count_builder.push("UNION SELECT rcr.child_role_id FROM role_composite_roles rcr ");
        count_builder.push("JOIN role_hierarchy rh ON rcr.parent_role_id = rh.id");
        count_builder.push(") ");
        count_builder.push("SELECT COUNT(*) FROM roles r ");
        count_builder.push("LEFT JOIN direct_roles dr ON r.id = dr.role_id ");
        count_builder.push("LEFT JOIN role_hierarchy rh ON r.id = rh.id ");
        count_builder.push("WHERE r.realm_id = ");
        count_builder.push_bind(realm_id.to_string());
        count_builder.push(" AND r.client_id IS NULL ");

        if let Some(q) = &req.q {
            if !q.trim().is_empty() {
                count_builder.push(" AND (r.name LIKE ");
                count_builder.push_bind(format!("%{}%", q));
                count_builder.push(" OR r.description LIKE ");
                count_builder.push_bind(format!("%{}%", q));
                count_builder.push(")");
            }
        }

        match filter {
            GroupRoleFilter::All => {}
            GroupRoleFilter::Direct => {
                count_builder.push(" AND dr.role_id IS NOT NULL");
            }
            GroupRoleFilter::Effective => {
                count_builder.push(" AND dr.role_id IS NULL AND rh.id IS NOT NULL");
            }
            GroupRoleFilter::Unassigned => {
                count_builder.push(" AND rh.id IS NULL");
            }
        }

        let total: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        let mut query_builder = QueryBuilder::new("");
        query_builder
            .push("WITH direct_roles AS (SELECT role_id FROM group_roles WHERE group_id = ");
        query_builder.push_bind(group_id.to_string());
        query_builder.push("), role_hierarchy(id) AS (");
        query_builder.push("SELECT role_id FROM direct_roles ");
        query_builder.push("UNION SELECT rcr.child_role_id FROM role_composite_roles rcr ");
        query_builder.push("JOIN role_hierarchy rh ON rcr.parent_role_id = rh.id");
        query_builder.push(") ");
        query_builder.push("SELECT r.id, r.name, r.description, ");
        query_builder.push("CASE WHEN dr.role_id IS NOT NULL THEN 1 ELSE 0 END AS is_direct, ");
        query_builder.push("CASE WHEN rh.id IS NOT NULL THEN 1 ELSE 0 END AS is_effective ");
        query_builder.push("FROM roles r ");
        query_builder.push("LEFT JOIN direct_roles dr ON r.id = dr.role_id ");
        query_builder.push("LEFT JOIN role_hierarchy rh ON r.id = rh.id ");
        query_builder.push("WHERE r.realm_id = ");
        query_builder.push_bind(realm_id.to_string());
        query_builder.push(" AND r.client_id IS NULL ");

        if let Some(q) = &req.q {
            if !q.trim().is_empty() {
                query_builder.push(" AND (r.name LIKE ");
                query_builder.push_bind(format!("%{}%", q));
                query_builder.push(" OR r.description LIKE ");
                query_builder.push_bind(format!("%{}%", q));
                query_builder.push(")");
            }
        }

        match filter {
            GroupRoleFilter::All => {}
            GroupRoleFilter::Direct => {
                query_builder.push(" AND dr.role_id IS NOT NULL");
            }
            GroupRoleFilter::Effective => {
                query_builder.push(" AND dr.role_id IS NULL AND rh.id IS NOT NULL");
            }
            GroupRoleFilter::Unassigned => {
                query_builder.push(" AND rh.id IS NULL");
            }
        }

        let sort_col = match req.sort_by.as_deref() {
            Some("name") => "r.name",
            _ => "r.name",
        };
        let sort_dir = match req.sort_dir.unwrap_or(SortDirection::Asc) {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };
        query_builder.push(format!(" ORDER BY {} {}", sort_col, sort_dir));

        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        let roles: Vec<GroupRoleRow> = query_builder
            .build_query_as()
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(PageResponse::new(roles, total, req.page, limit))
    }

    async fn list_user_roles(
        &self,
        realm_id: &Uuid,
        user_id: &Uuid,
        filter: UserRoleFilter,
        req: &PageRequest,
    ) -> Result<PageResponse<UserRoleRow>> {
        let limit = req.per_page.clamp(1, 100);
        let offset = (req.page - 1) * limit;

        let mut count_builder = QueryBuilder::new("");
        count_builder.push("WITH direct_roles AS (SELECT role_id FROM user_roles WHERE user_id = ");
        count_builder.push_bind(user_id.to_string());
        count_builder.push("), group_roles_cte AS (");
        count_builder.push(
            "SELECT gr.role_id FROM group_roles gr JOIN user_groups ug ON gr.group_id = ug.group_id WHERE ug.user_id = ",
        );
        count_builder.push_bind(user_id.to_string());
        count_builder.push("), base_roles AS (");
        count_builder
            .push("SELECT role_id FROM direct_roles UNION SELECT role_id FROM group_roles_cte");
        count_builder.push("), role_hierarchy(id) AS (");
        count_builder.push("SELECT role_id FROM base_roles ");
        count_builder.push("UNION SELECT rcr.child_role_id FROM role_composite_roles rcr ");
        count_builder.push("JOIN role_hierarchy rh ON rcr.parent_role_id = rh.id");
        count_builder.push(") ");
        count_builder.push("SELECT COUNT(*) FROM roles r ");
        count_builder.push("LEFT JOIN direct_roles dr ON r.id = dr.role_id ");
        count_builder.push("LEFT JOIN role_hierarchy rh ON r.id = rh.id ");
        count_builder.push("WHERE r.realm_id = ");
        count_builder.push_bind(realm_id.to_string());
        count_builder.push(" AND r.client_id IS NULL ");

        if let Some(q) = &req.q {
            if !q.trim().is_empty() {
                count_builder.push(" AND (r.name LIKE ");
                count_builder.push_bind(format!("%{}%", q));
                count_builder.push(" OR r.description LIKE ");
                count_builder.push_bind(format!("%{}%", q));
                count_builder.push(")");
            }
        }

        match filter {
            UserRoleFilter::All => {}
            UserRoleFilter::Direct => {
                count_builder.push(" AND dr.role_id IS NOT NULL");
            }
            UserRoleFilter::Effective => {
                count_builder.push(" AND dr.role_id IS NULL AND rh.id IS NOT NULL");
            }
            UserRoleFilter::Unassigned => {
                count_builder.push(" AND rh.id IS NULL");
            }
        }

        let total: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        let mut query_builder = QueryBuilder::new("");
        query_builder.push("WITH direct_roles AS (SELECT role_id FROM user_roles WHERE user_id = ");
        query_builder.push_bind(user_id.to_string());
        query_builder.push("), group_roles_cte AS (");
        query_builder.push(
            "SELECT gr.role_id FROM group_roles gr JOIN user_groups ug ON gr.group_id = ug.group_id WHERE ug.user_id = ",
        );
        query_builder.push_bind(user_id.to_string());
        query_builder.push("), base_roles AS (");
        query_builder
            .push("SELECT role_id FROM direct_roles UNION SELECT role_id FROM group_roles_cte");
        query_builder.push("), role_hierarchy(id) AS (");
        query_builder.push("SELECT role_id FROM base_roles ");
        query_builder.push("UNION SELECT rcr.child_role_id FROM role_composite_roles rcr ");
        query_builder.push("JOIN role_hierarchy rh ON rcr.parent_role_id = rh.id");
        query_builder.push(") ");
        query_builder.push("SELECT r.id, r.name, r.description, ");
        query_builder.push("CASE WHEN dr.role_id IS NOT NULL THEN 1 ELSE 0 END AS is_direct, ");
        query_builder.push("CASE WHEN rh.id IS NOT NULL THEN 1 ELSE 0 END AS is_effective ");
        query_builder.push("FROM roles r ");
        query_builder.push("LEFT JOIN direct_roles dr ON r.id = dr.role_id ");
        query_builder.push("LEFT JOIN role_hierarchy rh ON r.id = rh.id ");
        query_builder.push("WHERE r.realm_id = ");
        query_builder.push_bind(realm_id.to_string());
        query_builder.push(" AND r.client_id IS NULL ");

        if let Some(q) = &req.q {
            if !q.trim().is_empty() {
                query_builder.push(" AND (r.name LIKE ");
                query_builder.push_bind(format!("%{}%", q));
                query_builder.push(" OR r.description LIKE ");
                query_builder.push_bind(format!("%{}%", q));
                query_builder.push(")");
            }
        }

        match filter {
            UserRoleFilter::All => {}
            UserRoleFilter::Direct => {
                query_builder.push(" AND dr.role_id IS NOT NULL");
            }
            UserRoleFilter::Effective => {
                query_builder.push(" AND dr.role_id IS NULL AND rh.id IS NOT NULL");
            }
            UserRoleFilter::Unassigned => {
                query_builder.push(" AND rh.id IS NULL");
            }
        }

        let sort_col = match req.sort_by.as_deref() {
            Some("name") => "r.name",
            _ => "r.name",
        };
        let sort_dir = match req.sort_dir.unwrap_or(SortDirection::Asc) {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };
        query_builder.push(format!(" ORDER BY {} {}", sort_col, sort_dir));

        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        let roles: Vec<UserRoleRow> = query_builder
            .build_query_as()
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(PageResponse::new(roles, total, req.page, limit))
    }

    async fn list_role_composites(
        &self,
        realm_id: &Uuid,
        role_id: &Uuid,
        client_id: &Option<Uuid>,
        filter: RoleCompositeFilter,
        req: &PageRequest,
    ) -> Result<PageResponse<RoleCompositeRow>> {
        let limit = req.per_page.clamp(1, 100);
        let offset = (req.page - 1) * limit;

        let mut count_builder = QueryBuilder::new("");
        count_builder.push("WITH direct_roles AS (");
        count_builder.push(
            "SELECT child_role_id AS role_id FROM role_composite_roles WHERE parent_role_id = ",
        );
        count_builder.push_bind(role_id.to_string());
        count_builder.push("), role_hierarchy(id) AS (");
        count_builder.push("SELECT role_id FROM direct_roles ");
        count_builder.push("UNION SELECT rcr.child_role_id FROM role_composite_roles rcr ");
        count_builder.push("JOIN role_hierarchy rh ON rcr.parent_role_id = rh.id");
        count_builder.push(") ");
        count_builder.push("SELECT COUNT(*) FROM roles r ");
        count_builder.push("LEFT JOIN direct_roles dr ON r.id = dr.role_id ");
        count_builder.push("LEFT JOIN role_hierarchy rh ON r.id = rh.id ");
        count_builder.push("WHERE r.realm_id = ");
        count_builder.push_bind(realm_id.to_string());
        count_builder.push(" AND r.id != ");
        count_builder.push_bind(role_id.to_string());

        match client_id {
            Some(id) => {
                count_builder.push(" AND r.client_id = ");
                count_builder.push_bind(id.to_string());
            }
            None => {
                count_builder.push(" AND r.client_id IS NULL");
            }
        }

        if let Some(q) = &req.q {
            if !q.trim().is_empty() {
                count_builder.push(" AND (r.name LIKE ");
                count_builder.push_bind(format!("%{}%", q));
                count_builder.push(" OR r.description LIKE ");
                count_builder.push_bind(format!("%{}%", q));
                count_builder.push(")");
            }
        }

        match filter {
            RoleCompositeFilter::All => {}
            RoleCompositeFilter::Direct => {
                count_builder.push(" AND dr.role_id IS NOT NULL");
            }
            RoleCompositeFilter::Effective => {
                count_builder.push(" AND dr.role_id IS NULL AND rh.id IS NOT NULL");
            }
            RoleCompositeFilter::Unassigned => {
                count_builder.push(" AND rh.id IS NULL");
            }
        }

        let total: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        let mut query_builder = QueryBuilder::new("");
        query_builder.push("WITH direct_roles AS (");
        query_builder.push(
            "SELECT child_role_id AS role_id FROM role_composite_roles WHERE parent_role_id = ",
        );
        query_builder.push_bind(role_id.to_string());
        query_builder.push("), role_hierarchy(id) AS (");
        query_builder.push("SELECT role_id FROM direct_roles ");
        query_builder.push("UNION SELECT rcr.child_role_id FROM role_composite_roles rcr ");
        query_builder.push("JOIN role_hierarchy rh ON rcr.parent_role_id = rh.id");
        query_builder.push(") ");
        query_builder.push("SELECT r.id, r.name, r.description, ");
        query_builder.push("CASE WHEN dr.role_id IS NOT NULL THEN 1 ELSE 0 END AS is_direct, ");
        query_builder.push("CASE WHEN rh.id IS NOT NULL THEN 1 ELSE 0 END AS is_effective ");
        query_builder.push("FROM roles r ");
        query_builder.push("LEFT JOIN direct_roles dr ON r.id = dr.role_id ");
        query_builder.push("LEFT JOIN role_hierarchy rh ON r.id = rh.id ");
        query_builder.push("WHERE r.realm_id = ");
        query_builder.push_bind(realm_id.to_string());
        query_builder.push(" AND r.id != ");
        query_builder.push_bind(role_id.to_string());

        match client_id {
            Some(id) => {
                query_builder.push(" AND r.client_id = ");
                query_builder.push_bind(id.to_string());
            }
            None => {
                query_builder.push(" AND r.client_id IS NULL");
            }
        }

        if let Some(q) = &req.q {
            if !q.trim().is_empty() {
                query_builder.push(" AND (r.name LIKE ");
                query_builder.push_bind(format!("%{}%", q));
                query_builder.push(" OR r.description LIKE ");
                query_builder.push_bind(format!("%{}%", q));
                query_builder.push(")");
            }
        }

        match filter {
            RoleCompositeFilter::All => {}
            RoleCompositeFilter::Direct => {
                query_builder.push(" AND dr.role_id IS NOT NULL");
            }
            RoleCompositeFilter::Effective => {
                query_builder.push(" AND dr.role_id IS NULL AND rh.id IS NOT NULL");
            }
            RoleCompositeFilter::Unassigned => {
                query_builder.push(" AND rh.id IS NULL");
            }
        }

        let sort_col = match req.sort_by.as_deref() {
            Some("name") => "r.name",
            _ => "r.name",
        };
        let sort_dir = match req.sort_dir.unwrap_or(SortDirection::Asc) {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };
        query_builder.push(format!(" ORDER BY {} {}", sort_col, sort_dir));

        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        let roles: Vec<RoleCompositeRow> = query_builder
            .build_query_as()
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(PageResponse::new(roles, total, req.page, limit))
    }

    async fn list_group_ids_by_parent(
        &self,
        realm_id: &Uuid,
        parent_id: Option<&Uuid>,
    ) -> Result<Vec<Uuid>> {
        let mut query_builder = QueryBuilder::new("SELECT id FROM groups g");
        Self::apply_group_tree_filters(&mut query_builder, realm_id, parent_id, &None);
        query_builder.push(" ORDER BY g.sort_order ASC, g.name ASC");

        let rows: Vec<(String,)> = query_builder
            .build_query_as()
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows
            .into_iter()
            .filter_map(|(id,)| Uuid::parse_str(&id).ok())
            .collect())
    }

    async fn list_group_subtree_ids(&self, realm_id: &Uuid, root_id: &Uuid) -> Result<Vec<Uuid>> {
        let rows: Vec<String> = sqlx::query_scalar(
            r#"
            WITH RECURSIVE subtree(id) AS (
                SELECT id FROM groups WHERE id = ? AND realm_id = ?
                UNION ALL
                SELECT g.id FROM groups g
                JOIN subtree s ON g.parent_id = s.id
                WHERE g.realm_id = ?
            )
            SELECT id FROM subtree
            "#,
        )
        .bind(root_id.to_string())
        .bind(realm_id.to_string())
        .bind(realm_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows
            .into_iter()
            .filter_map(|id| Uuid::parse_str(&id).ok())
            .collect())
    }

    async fn set_group_orders(
        &self,
        realm_id: &Uuid,
        parent_id: Option<&Uuid>,
        ordered_ids: &[Uuid],
    ) -> Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        let parent = parent_id.map(|id| id.to_string());
        let realm = realm_id.to_string();

        for (index, group_id) in ordered_ids.iter().enumerate() {
            sqlx::query(
                "UPDATE groups SET parent_id = ?, sort_order = ? WHERE id = ? AND realm_id = ?",
            )
            .bind(parent.clone())
            .bind(index as i64)
            .bind(group_id.to_string())
            .bind(&realm)
            .execute(&mut *tx)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        }

        tx.commit().await.map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn is_group_descendant(
        &self,
        realm_id: &Uuid,
        ancestor_id: &Uuid,
        candidate_id: &Uuid,
    ) -> Result<bool> {
        let row: (i64,) = sqlx::query_as(
            r#"
            WITH RECURSIVE descendants(id) AS (
                SELECT id FROM groups WHERE parent_id = ? AND realm_id = ?
                UNION ALL
                SELECT g.id FROM groups g
                JOIN descendants d ON g.parent_id = d.id
                WHERE g.realm_id = ?
            )
            SELECT COUNT(1) FROM descendants WHERE id = ?
            "#,
        )
        .bind(ancestor_id.to_string())
        .bind(realm_id.to_string())
        .bind(realm_id.to_string())
        .bind(candidate_id.to_string())
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(row.0 > 0)
    }

    async fn get_next_group_sort_order(
        &self,
        realm_id: &Uuid,
        parent_id: Option<&Uuid>,
    ) -> Result<i64> {
        let mut query_builder =
            QueryBuilder::new("SELECT COALESCE(MAX(sort_order), -1) + 1 FROM groups g");
        Self::apply_group_tree_filters(&mut query_builder, realm_id, parent_id, &None);

        let next: i64 = query_builder
            .build_query_scalar()
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(next)
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

    async fn find_user_ids_in_groups(&self, group_ids: &[Uuid]) -> Result<Vec<Uuid>> {
        if group_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut query_builder =
            QueryBuilder::new("SELECT DISTINCT user_id FROM user_groups WHERE group_id IN (");
        let mut separated = query_builder.separated(", ");
        for id in group_ids {
            separated.push_bind(id.to_string());
        }
        query_builder.push(")");

        let rows: Vec<String> = query_builder
            .build_query_scalar()
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows
            .into_iter()
            .filter_map(|id| Uuid::parse_str(&id).ok())
            .collect())
    }

    async fn find_role_ids_for_group(&self, group_id: &Uuid) -> Result<Vec<Uuid>> {
        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT role_id FROM group_roles WHERE group_id = ?")
                .bind(group_id.to_string())
                .fetch_all(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;

        let uuids = rows
            .into_iter()
            .filter_map(|(id,)| Uuid::parse_str(&id).ok())
            .collect();
        Ok(uuids)
    }
    async fn find_effective_role_ids_for_group(&self, group_id: &Uuid) -> Result<Vec<Uuid>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            WITH RECURSIVE role_hierarchy(id) AS (
                SELECT role_id FROM group_roles WHERE group_id = ?
                UNION
                SELECT rcr.child_role_id
                FROM role_composite_roles rcr
                JOIN role_hierarchy rh ON rcr.parent_role_id = rh.id
            )
            SELECT DISTINCT id FROM role_hierarchy
            "#,
        )
        .bind(group_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows
            .into_iter()
            .filter_map(|(id,)| Uuid::parse_str(&id).ok())
            .collect())
    }

    async fn count_user_ids_in_groups(&self, group_ids: &[Uuid]) -> Result<i64> {
        if group_ids.is_empty() {
            return Ok(0);
        }

        let mut query_builder = QueryBuilder::new(
            "SELECT COUNT(DISTINCT user_id) FROM user_groups WHERE group_id IN (",
        );
        let mut separated = query_builder.separated(", ");
        for id in group_ids {
            separated.push_bind(id.to_string());
        }
        query_builder.push(")");

        let count: i64 = query_builder
            .build_query_scalar()
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(count)
    }

    async fn count_role_ids_in_groups(&self, group_ids: &[Uuid]) -> Result<i64> {
        if group_ids.is_empty() {
            return Ok(0);
        }

        let mut query_builder = QueryBuilder::new(
            "SELECT COUNT(DISTINCT role_id) FROM group_roles WHERE group_id IN (",
        );
        let mut separated = query_builder.separated(", ");
        for id in group_ids {
            separated.push_bind(id.to_string());
        }
        query_builder.push(")");

        let count: i64 = query_builder
            .build_query_scalar()
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(count)
    }

    async fn find_direct_role_ids_for_user(&self, user_id: &Uuid) -> Result<Vec<Uuid>> {
        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT role_id FROM user_roles WHERE user_id = ?")
                .bind(user_id.to_string())
                .fetch_all(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows
            .into_iter()
            .filter_map(|(id,)| Uuid::parse_str(&id).ok())
            .collect())
    }

    async fn find_effective_role_ids_for_user(&self, user_id: &Uuid) -> Result<Vec<Uuid>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            WITH RECURSIVE role_hierarchy(id) AS (
                SELECT role_id FROM user_roles WHERE user_id = ?
                UNION
                SELECT gr.role_id
                FROM group_roles gr
                JOIN user_groups ug ON gr.group_id = ug.group_id
                WHERE ug.user_id = ?
                UNION
                SELECT rcr.child_role_id
                FROM role_composite_roles rcr
                JOIN role_hierarchy rh ON rcr.parent_role_id = rh.id
            )
            SELECT DISTINCT id FROM role_hierarchy
            "#,
        )
        .bind(user_id.to_string())
        .bind(user_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows
            .into_iter()
            .filter_map(|(id,)| Uuid::parse_str(&id).ok())
            .collect())
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
            SELECT DISTINCT user_id FROM (
                -- 3a. Direct user-role assignments
                SELECT user_id FROM user_roles
                WHERE role_id IN (SELECT id FROM role_hierarchy)
                UNION
                -- 3b. Users in groups that have any of these roles
                SELECT ug.user_id FROM user_groups ug
                JOIN group_roles gr ON ug.group_id = gr.group_id
                WHERE gr.role_id IN (SELECT id FROM role_hierarchy)
            )
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

    async fn find_direct_user_ids_for_role(&self, role_id: &Uuid) -> Result<Vec<Uuid>> {
        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT user_id FROM user_roles WHERE role_id = ?")
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

    async fn list_role_composite_ids(&self, role_id: &Uuid) -> Result<Vec<Uuid>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT child_role_id FROM role_composite_roles WHERE parent_role_id = ?",
        )
        .bind(role_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows
            .into_iter()
            .filter_map(|(id,)| Uuid::parse_str(&id).ok())
            .collect())
    }

    async fn list_effective_role_composite_ids(&self, role_id: &Uuid) -> Result<Vec<Uuid>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            WITH RECURSIVE role_hierarchy(id) AS (
                SELECT child_role_id FROM role_composite_roles WHERE parent_role_id = ?
                UNION
                SELECT rcr.child_role_id
                FROM role_composite_roles rcr
                JOIN role_hierarchy rh ON rcr.parent_role_id = rh.id
            )
            SELECT DISTINCT id FROM role_hierarchy
            "#,
        )
        .bind(role_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows
            .into_iter()
            .filter_map(|(id,)| Uuid::parse_str(&id).ok())
            .collect())
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

    async fn delete_groups(&self, group_ids: &[Uuid]) -> Result<()> {
        if group_ids.is_empty() {
            return Ok(());
        }

        let mut query_builder = QueryBuilder::new("DELETE FROM groups WHERE id IN (");
        let mut separated = query_builder.separated(", ");
        for id in group_ids {
            separated.push_bind(id.to_string());
        }
        query_builder.push(")");

        query_builder
            .build()
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(())
    }

    async fn update_role(&self, role: &Role) -> Result<()> {
        sqlx::query("UPDATE roles SET name = ?, description = ? WHERE id = ?")
            .bind(&role.name)
            .bind(&role.description)
            .bind(role.id.to_string())
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(())
    }

    async fn update_group(&self, group: &Group) -> Result<()> {
        sqlx::query("UPDATE groups SET name = ?, description = ? WHERE id = ?")
            .bind(&group.name)
            .bind(&group.description)
            .bind(group.id.to_string())
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(())
    }

    async fn get_permissions_for_role(&self, role_id: &Uuid) -> Result<Vec<String>> {
        let perms = sqlx::query_scalar::<_, String>(
            "SELECT permission_name FROM role_permissions WHERE role_id = ?",
        )
        .bind(role_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(perms)
    }

    async fn remove_permission(&self, role_id: &Uuid, permission: &str) -> Result<()> {
        sqlx::query("DELETE FROM role_permissions WHERE role_id = ? AND permission_name = ?")
            .bind(role_id.to_string())
            .bind(permission)
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    // Efficient Bulk Operation
    async fn bulk_update_permissions(
        &self,
        role_id: &Uuid,
        permissions: Vec<String>,
        action: &str,
    ) -> Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        let rid = role_id.to_string();

        for perm in permissions {
            if action == "add" {
                // Ignore if exists
                sqlx::query("INSERT OR IGNORE INTO role_permissions (role_id, permission_name) VALUES (?, ?)")
                    .bind(&rid)
                    .bind(perm)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| Error::Unexpected(e.into()))?;
            } else if action == "remove" {
                sqlx::query(
                    "DELETE FROM role_permissions WHERE role_id = ? AND permission_name = ?",
                )
                .bind(&rid)
                .bind(perm)
                .execute(&mut *tx)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
            }
        }

        tx.commit().await.map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn assign_composite_role(
        &self,
        parent_role_id: &Uuid,
        child_role_id: &Uuid,
    ) -> Result<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO role_composite_roles (parent_role_id, child_role_id) VALUES (?, ?)",
        )
        .bind(parent_role_id.to_string())
        .bind(child_role_id.to_string())
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn remove_composite_role(
        &self,
        parent_role_id: &Uuid,
        child_role_id: &Uuid,
    ) -> Result<()> {
        sqlx::query(
            "DELETE FROM role_composite_roles WHERE parent_role_id = ? AND child_role_id = ?",
        )
        .bind(parent_role_id.to_string())
        .bind(child_role_id.to_string())
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn is_role_descendant(&self, ancestor_id: &Uuid, candidate_id: &Uuid) -> Result<bool> {
        let row: (i64,) = sqlx::query_as(
            r#"
            WITH RECURSIVE descendants(id) AS (
                SELECT child_role_id FROM role_composite_roles WHERE parent_role_id = ?
                UNION ALL
                SELECT rcr.child_role_id
                FROM role_composite_roles rcr
                JOIN descendants d ON rcr.parent_role_id = d.id
            )
            SELECT COUNT(1) FROM descendants WHERE id = ?
            "#,
        )
        .bind(ancestor_id.to_string())
        .bind(candidate_id.to_string())
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(row.0 > 0)
    }
}
