use crate::adapters::persistence::connection::Database;
use crate::adapters::persistence::transaction::SqliteTransaction;
use crate::ports::transaction_manager::Transaction;
use crate::{
    domain::auth_flow::AuthFlow,
    error::{Error, Result},
    ports::flow_repository::FlowRepository,
};
use async_trait::async_trait;
use tracing::instrument;
use uuid::Uuid;

pub struct SqliteFlowRepository {
    pool: Database,
}

impl SqliteFlowRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FlowRepository for SqliteFlowRepository {
    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "auth_flows", db_op = "select")
    )]
    async fn find_flow_by_name(&self, realm_id: &Uuid, name: &str) -> Result<Option<AuthFlow>> {
        let flow = sqlx::query_as("SELECT * FROM auth_flows WHERE realm_id = ? AND name = ?")
            .bind(realm_id.to_string())
            .bind(name)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(flow)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "auth_flows", db_op = "select")
    )]
    async fn find_flow_by_id(&self, flow_id: &Uuid) -> Result<Option<AuthFlow>> {
        let flow = sqlx::query_as("SELECT * FROM auth_flows WHERE id = ?")
            .bind(flow_id.to_string())
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(flow)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "auth_flows", db_op = "insert")
    )]
    async fn create_flow<'a>(
        &self,
        flow: &AuthFlow,
        tx: Option<&'a mut dyn Transaction>,
    ) -> Result<()> {
        let query = sqlx::query(
            "INSERT INTO auth_flows (id, realm_id, name, alias, type, built_in, description)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(flow.id.to_string())
        .bind(flow.realm_id.to_string())
        .bind(&flow.name)
        .bind(&flow.alias)
        .bind(&flow.r#type)
        .bind(flow.built_in)
        .bind(&flow.description);

        // Use helper logic or match block
        if let Some(t) = tx {
            let sql_tx = SqliteTransaction::from_trait(t).expect("Invalid TX");
            query.execute(&mut **sql_tx).await
        } else {
            query.execute(&*self.pool).await
        }
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "auth_flows", db_op = "select")
    )]
    async fn list_flows_by_realm(&self, realm_id: &Uuid) -> Result<Vec<AuthFlow>> {
        let flows =
            sqlx::query_as("SELECT * FROM auth_flows WHERE realm_id = ? ORDER BY alias ")
                .bind(realm_id.to_string())
                .fetch_all(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(flows)
    }
}
