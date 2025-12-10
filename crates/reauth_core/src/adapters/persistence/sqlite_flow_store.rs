use crate::adapters::persistence::connection::Database;
use crate::{
    domain::{
        flow::{FlowDeployment, FlowDraft, FlowVersion},
        pagination::{PageRequest, PageResponse, SortDirection},
    },
    error::{Error, Result},
    ports::flow_store::FlowStore,
};
use async_trait::async_trait;
use sqlx::{QueryBuilder, Sqlite};
use uuid::Uuid;

pub struct SqliteFlowStore {
    pool: Database,
}

impl SqliteFlowStore {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }

    fn apply_draft_filters<'a>(
        builder: &mut QueryBuilder<'a, Sqlite>,
        realm_id: &Uuid,
        q: &Option<String>,
    ) {
        builder.push(" WHERE realm_id = ");
        builder.push_bind(realm_id.to_string());
        if let Some(query) = q {
            if !query.is_empty() {
                builder.push(" AND name LIKE ");
                builder.push_bind(format!("%{}%", query));
            }
        }
    }
}

#[async_trait]
impl FlowStore for SqliteFlowStore {
    // --- DRAFTS ---

    async fn create_draft(&self, draft: &FlowDraft) -> Result<()> {
        sqlx::query(
            "INSERT INTO flow_drafts (id, realm_id, name, description, graph_json, flow_type, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
            .bind(draft.id.to_string())
            .bind(draft.realm_id.to_string())
            .bind(&draft.name)
            .bind(&draft.description)
            .bind(&draft.graph_json)
            .bind(&draft.flow_type)
            .bind(draft.created_at)
            .bind(draft.updated_at)
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn update_draft(&self, draft: &FlowDraft) -> Result<()> {
        sqlx::query(
            "UPDATE flow_drafts SET name = ?, description = ?, graph_json = ?, updated_at = ? WHERE id = ?"
        )
            .bind(&draft.name)
            .bind(&draft.description)
            .bind(&draft.graph_json)
            .bind(draft.updated_at)
            .bind(draft.id.to_string())
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn get_draft_by_id(&self, id: &Uuid) -> Result<Option<FlowDraft>> {
        let draft = sqlx::query_as("SELECT * FROM flow_drafts WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(draft)
    }

    async fn list_drafts(
        &self,
        realm_id: &Uuid,
        req: &PageRequest,
    ) -> Result<PageResponse<FlowDraft>> {
        let limit = req.per_page.clamp(1, 100);
        let offset = (req.page - 1) * limit;

        // Count
        let mut count_builder = QueryBuilder::new("SELECT COUNT(*) FROM flow_drafts");
        Self::apply_draft_filters(&mut count_builder, realm_id, &req.q);
        let total: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        // Select
        let mut query_builder = QueryBuilder::new("SELECT * FROM flow_drafts");
        Self::apply_draft_filters(&mut query_builder, realm_id, &req.q);

        // Sort
        let sort_col = match req.sort_by.as_deref() {
            Some("updated_at") => "updated_at",
            Some("name") => "name",
            _ => "updated_at",
        };
        let sort_dir = match req.sort_dir.unwrap_or(SortDirection::Desc) {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };
        query_builder.push(format!(" ORDER BY {} {}", sort_col, sort_dir));

        // Paginate
        query_builder
            .push(" LIMIT ")
            .push_bind(limit)
            .push(" OFFSET ")
            .push_bind(offset);

        let drafts: Vec<FlowDraft> = query_builder
            .build_query_as()
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(PageResponse::new(drafts, total, req.page, limit))
    }

    async fn list_all_drafts(&self, realm_id: &Uuid) -> Result<Vec<FlowDraft>> {
        let drafts =
            sqlx::query_as("SELECT * FROM flow_drafts WHERE realm_id = ? ORDER BY created_at DESC")
                .bind(realm_id.to_string())
                .fetch_all(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(drafts)
    }

    async fn delete_draft(&self, id: &Uuid) -> Result<()> {
        sqlx::query("DELETE FROM flow_drafts WHERE id = ?")
            .bind(id.to_string())
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    // --- VERSIONS ---

    async fn create_version(&self, version: &FlowVersion) -> Result<()> {
        sqlx::query(
            "INSERT INTO flow_versions (id, draft_id, version_number, execution_artifact, checksum, created_at)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
            .bind(version.id.to_string())
            .bind(version.draft_id.to_string())
            .bind(version.version_number)
            .bind(&version.execution_artifact)
            .bind(&version.checksum)
            .bind(version.created_at)
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn get_version(&self, id: &Uuid) -> Result<Option<FlowVersion>> {
        let version = sqlx::query_as("SELECT * FROM flow_versions WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(version)
    }

    async fn list_versions(&self, draft_id: &Uuid) -> Result<Vec<FlowVersion>> {
        let versions = sqlx::query_as(
            "SELECT * FROM flow_versions WHERE draft_id = ? ORDER BY version_number DESC",
        )
        .bind(draft_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(versions)
    }

    // --- DEPLOYMENTS ---

    async fn set_deployment(&self, deployment: &FlowDeployment) -> Result<()> {
        // Upsert logic: If a deployment for this realm+type exists, update it. If not, insert it.
        sqlx::query(
            "INSERT INTO flow_deployments (id, realm_id, flow_type, active_version_id, updated_at)
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(realm_id, flow_type) DO UPDATE SET
             active_version_id = excluded.active_version_id,
             updated_at = excluded.updated_at",
        )
        .bind(deployment.id.to_string())
        .bind(deployment.realm_id.to_string())
        .bind(&deployment.flow_type)
        .bind(deployment.active_version_id.to_string())
        .bind(deployment.updated_at)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn get_deployment(
        &self,
        realm_id: &Uuid,
        flow_type: &str,
    ) -> Result<Option<FlowDeployment>> {
        let dep =
            sqlx::query_as("SELECT * FROM flow_deployments WHERE realm_id = ? AND flow_type = ?")
                .bind(realm_id.to_string())
                .bind(flow_type)
                .fetch_optional(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(dep)
    }
}
