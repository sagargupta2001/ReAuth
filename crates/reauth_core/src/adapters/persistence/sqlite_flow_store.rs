use crate::adapters::persistence::connection::Database;
use crate::adapters::persistence::transaction::SqliteTransaction;
use crate::domain::flow::models::{FlowDeployment, FlowDraft, FlowVersion};
use crate::{
    domain::pagination::{PageRequest, PageResponse, SortDirection},
    error::{Error, Result},
    ports::flow_store::FlowStore,
    ports::transaction_manager::Transaction,
};
use async_trait::async_trait;
use chrono::Utc;
use sqlx::{QueryBuilder, Sqlite};
use tracing::instrument;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
struct FlowVersionRow {
    id: String,
    flow_id: String,
    version_number: i32,
    execution_artifact: String,
    graph_json: String,
    checksum: String,
    created_at: chrono::DateTime<Utc>,
}

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

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "flow_drafts", db_op = "insert")
    )]
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

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "flow_drafts", db_op = "insert")
    )]
    async fn create_draft_with_tx(
        &self,
        draft: &FlowDraft,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = sqlx::query(
            "INSERT INTO flow_drafts (id, realm_id, name, description, graph_json, flow_type, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(draft.id.to_string())
        .bind(draft.realm_id.to_string())
        .bind(&draft.name)
        .bind(&draft.description)
        .bind(&draft.graph_json)
        .bind(&draft.flow_type)
        .bind(draft.created_at)
        .bind(draft.updated_at);

        if let Some(tx) = tx {
            let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX");
            query.execute(&mut **sql_tx).await
        } else {
            query.execute(&*self.pool).await
        }
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "flow_drafts", db_op = "update")
    )]
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

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "flow_drafts", db_op = "update")
    )]
    async fn update_draft_with_tx(
        &self,
        draft: &FlowDraft,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = sqlx::query(
            "UPDATE flow_drafts SET name = ?, description = ?, graph_json = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&draft.name)
        .bind(&draft.description)
        .bind(&draft.graph_json)
        .bind(draft.updated_at)
        .bind(draft.id.to_string());

        if let Some(tx) = tx {
            let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX");
            query.execute(&mut **sql_tx).await
        } else {
            query.execute(&*self.pool).await
        }
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "flow_drafts", db_op = "select")
    )]
    async fn get_draft_by_id(&self, id: &Uuid) -> Result<Option<FlowDraft>> {
        let draft = sqlx::query_as("SELECT * FROM flow_drafts WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(draft)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "flow_drafts", db_op = "select")
    )]
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

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "flow_drafts", db_op = "select")
    )]
    async fn list_all_drafts(&self, realm_id: &Uuid) -> Result<Vec<FlowDraft>> {
        let drafts =
            sqlx::query_as("SELECT * FROM flow_drafts WHERE realm_id = ? ORDER BY created_at DESC")
                .bind(realm_id.to_string())
                .fetch_all(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(drafts)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "flow_drafts", db_op = "delete")
    )]
    async fn delete_draft(&self, id: &Uuid) -> Result<()> {
        sqlx::query("DELETE FROM flow_drafts WHERE id = ?")
            .bind(id.to_string())
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "flow_drafts", db_op = "delete")
    )]
    async fn delete_draft_with_tx(
        &self,
        id: &Uuid,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = sqlx::query("DELETE FROM flow_drafts WHERE id = ?").bind(id.to_string());

        if let Some(tx) = tx {
            let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX");
            query.execute(&mut **sql_tx).await
        } else {
            query.execute(&*self.pool).await
        }
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    // --- VERSIONS ---

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "flow_versions", db_op = "insert")
    )]
    async fn create_version(&self, version: &FlowVersion) -> Result<()> {
        sqlx::query(
            "INSERT INTO flow_versions (id, flow_id, version_number, execution_artifact, graph_json, checksum, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
            .bind(&version.id)
            .bind(&version.flow_id)
            .bind(version.version_number)
            .bind(&version.execution_artifact)
            .bind(&version.graph_json)
            .bind(&version.checksum)
            .bind(version.created_at)
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "flow_versions", db_op = "insert")
    )]
    async fn create_version_with_tx(
        &self,
        version: &FlowVersion,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = sqlx::query(
            "INSERT INTO flow_versions (id, flow_id, version_number, execution_artifact, graph_json, checksum, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&version.id)
        .bind(&version.flow_id)
        .bind(version.version_number)
        .bind(&version.execution_artifact)
        .bind(&version.graph_json)
        .bind(&version.checksum)
        .bind(version.created_at);

        if let Some(tx) = tx {
            let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX");
            query.execute(&mut **sql_tx).await
        } else {
            query.execute(&*self.pool).await
        }
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "flow_versions", db_op = "select")
    )]
    async fn get_version(&self, id: &Uuid) -> Result<Option<FlowVersion>> {
        let version = sqlx::query_as("SELECT * FROM flow_versions WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(version)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "flow_versions", db_op = "select")
    )]
    async fn list_versions(
        &self,
        flow_id: &Uuid,
        req: &PageRequest,
    ) -> Result<PageResponse<FlowVersion>> {
        // Standardize constraints
        let limit = req.per_page.clamp(1, 100);
        let offset = (req.page - 1) * limit;

        // 1. Count Total Versions (for pagination metadata)
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM flow_versions WHERE flow_id = ?")
            .bind(flow_id.to_string())
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        // 2. Build Query using QueryBuilder
        let mut query_builder = QueryBuilder::new("SELECT * FROM flow_versions WHERE flow_id = ");
        query_builder.push_bind(flow_id.to_string());

        // 3. Apply Sorting
        // Default to version_number DESC (newest first)
        let sort_col = match req.sort_by.as_deref() {
            Some("created_at") => "created_at",
            Some("version_number") => "version_number",
            _ => "version_number",
        };

        let sort_dir = match req.sort_dir.unwrap_or(SortDirection::Desc) {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };

        query_builder.push(format!(" ORDER BY {} {}", sort_col, sort_dir));

        // 4. Apply Pagination
        query_builder
            .push(" LIMIT ")
            .push_bind(limit)
            .push(" OFFSET ")
            .push_bind(offset);

        // 5. Execute
        let versions: Vec<FlowVersion> = query_builder
            .build_query_as()
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(PageResponse::new(versions, total, req.page, limit))
    }

    // --- DEPLOYMENTS ---

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "flow_deployments", db_op = "upsert")
    )]
    async fn set_deployment(&self, deployment: &FlowDeployment) -> Result<()> {
        // Upsert logic: If a deployment for this realm+type exists, update it. If not, insert it.
        sqlx::query(
            "INSERT INTO flow_deployments (id, realm_id, flow_type, active_version_id, updated_at)
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(realm_id, flow_type) DO UPDATE SET
             active_version_id = excluded.active_version_id,
             updated_at = excluded.updated_at",
        )
        .bind(&deployment.id)
        .bind(deployment.realm_id.to_string())
        .bind(&deployment.flow_type)
        .bind(&deployment.active_version_id)
        .bind(deployment.updated_at)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "flow_deployments", db_op = "upsert")
    )]
    async fn set_deployment_with_tx(
        &self,
        deployment: &FlowDeployment,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = sqlx::query(
            "INSERT INTO flow_deployments (id, realm_id, flow_type, active_version_id, updated_at)
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(realm_id, flow_type) DO UPDATE SET
             active_version_id = excluded.active_version_id,
             updated_at = excluded.updated_at",
        )
        .bind(&deployment.id)
        .bind(deployment.realm_id.to_string())
        .bind(&deployment.flow_type)
        .bind(&deployment.active_version_id)
        .bind(deployment.updated_at);

        if let Some(tx) = tx {
            let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX");
            query.execute(&mut **sql_tx).await
        } else {
            query.execute(&*self.pool).await
        }
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "flow_deployments", db_op = "select")
    )]
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

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "flow_versions", db_op = "select")
    )]
    async fn get_latest_version_number(&self, flow_id: &Uuid) -> Result<Option<i32>> {
        // We use query_scalar to get a single value
        let result: Option<i32> =
            sqlx::query_scalar("SELECT MAX(version_number) FROM flow_versions WHERE flow_id = ?")
                .bind(flow_id.to_string())
                .fetch_optional(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(result)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "flow_versions", db_op = "select")
    )]
    async fn get_latest_version(&self, flow_id: &Uuid) -> Result<Option<FlowVersion>> {
        sqlx::query_as(
            "SELECT * FROM flow_versions WHERE flow_id = ? ORDER BY version_number DESC LIMIT 1",
        )
        .bind(flow_id.to_string())
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "flow_deployments", db_op = "select")
    )]
    async fn get_deployed_version_number(
        &self,
        realm_id: &Uuid,
        flow_type: &str,
        flow_id: &Uuid,
    ) -> Result<Option<i32>> {
        let version_number: Option<i32> = sqlx::query_scalar(
            r#"
            SELECT v.version_number
            FROM flow_deployments d
            JOIN flow_versions v ON d.active_version_id = v.id
            WHERE d.realm_id = ?
              AND d.flow_type = ?
              AND v.flow_id = ?   -- ✅ CRITICAL CHECK: Ensure version belongs to THIS flow
            "#,
        )
        .bind(realm_id.to_string())
        .bind(flow_type)
        .bind(flow_id.to_string()) // Bind the flow ID
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(version_number)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "flow_versions", db_op = "select")
    )]
    async fn get_version_by_number(
        &self,
        flow_id: &Uuid,
        version_number: i32,
    ) -> Result<Option<FlowVersion>> {
        sqlx::query_as("SELECT * FROM flow_versions WHERE flow_id = ? AND version_number = ?")
            .bind(flow_id.to_string())
            .bind(version_number)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "flow_versions", db_op = "select")
    )]
    async fn get_active_version(&self, flow_id: &Uuid) -> Result<Option<FlowVersion>> {
        // We need to find if ANY deployment points to a version that belongs to this flow_id.
        // Schema: flow_versions (v) <--- flow_deployments (d)

        let query = "
            SELECT v.* FROM flow_versions v
            JOIN flow_deployments d ON d.active_version_id = v.id
            WHERE v.flow_id = ?
            LIMIT 1
        ";

        let row = sqlx::query_as::<_, FlowVersionRow>(query)
            .bind(flow_id.to_string())
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        // Map the Row (String IDs) back to Domain Object (Uuid IDs)
        if let Some(r) = row {
            Ok(Some(FlowVersion {
                id: r.id,
                flow_id: r.flow_id,
                version_number: r.version_number,
                execution_artifact: r.execution_artifact,
                graph_json: r.graph_json,
                checksum: r.checksum,
                created_at: r.created_at,
            }))
        } else {
            Ok(None)
        }
    }
}
