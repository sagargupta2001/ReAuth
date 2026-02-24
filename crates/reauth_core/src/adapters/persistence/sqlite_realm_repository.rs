use crate::adapters::persistence::connection::Database;
use crate::adapters::persistence::transaction::SqliteTransaction;
use crate::domain::auth_flow::AuthFlow;
use crate::ports::transaction_manager::Transaction;
use crate::{
    domain::realm::Realm,
    error::{Error, Result},
    ports::realm_repository::RealmRepository,
};
use async_trait::async_trait;
use tracing::instrument;
use uuid::Uuid;

pub struct SqliteRealmRepository {
    pool: Database,
}
impl SqliteRealmRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RealmRepository for SqliteRealmRepository {
    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "realms", db_op = "insert")
    )]
    async fn create<'a>(&self, realm: &Realm, tx: Option<&'a mut dyn Transaction>) -> Result<()> {
        // Build the query object, but DO NOT execute it yet.
        let query = sqlx::query(
            "INSERT INTO realms (
                id, name, access_token_ttl_secs, refresh_token_ttl_secs,
                browser_flow_id, registration_flow_id, direct_grant_flow_id, reset_credentials_flow_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
            .bind(realm.id.to_string())
            .bind(&realm.name)
            .bind(realm.access_token_ttl_secs)
            .bind(realm.refresh_token_ttl_secs)
            .bind(&realm.browser_flow_id)
            .bind(&realm.registration_flow_id)
            .bind(&realm.direct_grant_flow_id)
            .bind(&realm.reset_credentials_flow_id);

        // Choose the executor and run it
        if let Some(t) = tx {
            // Case A: Use the Transaction
            let sql_tx = SqliteTransaction::from_trait(t).expect("Invalid TX type");
            query.execute(&mut **sql_tx).await
        } else {
            // Case B: Use the Pool
            query.execute(&*self.pool).await
        }
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "realms", db_op = "select")
    )]
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Realm>> {
        Ok(sqlx::query_as("SELECT * FROM realms WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "realms", db_op = "select")
    )]
    async fn find_by_name(&self, name: &str) -> Result<Option<Realm>> {
        Ok(sqlx::query_as("SELECT * FROM realms WHERE name = ?")
            .bind(name)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "realms", db_op = "select")
    )]
    async fn list_all(&self) -> Result<Vec<Realm>> {
        Ok(sqlx::query_as("SELECT * FROM realms")
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "realms", db_op = "update")
    )]
    async fn update<'a>(&self, realm: &Realm, tx: Option<&'a mut dyn Transaction>) -> Result<()> {
        let query = sqlx::query(
            "UPDATE realms SET
                name = ?,
                access_token_ttl_secs = ?,
                refresh_token_ttl_secs = ?,
                browser_flow_id = ?,
                registration_flow_id = ?,
                direct_grant_flow_id = ?,
                reset_credentials_flow_id = ?
             WHERE id = ?",
        )
        .bind(&realm.name)
        .bind(realm.access_token_ttl_secs)
        .bind(realm.refresh_token_ttl_secs)
        .bind(&realm.browser_flow_id)
        .bind(&realm.registration_flow_id)
        .bind(&realm.direct_grant_flow_id)
        .bind(&realm.reset_credentials_flow_id)
        .bind(realm.id.to_string());

        // Execute on correct target
        if let Some(t) = tx {
            let sql_tx = SqliteTransaction::from_trait(t).expect("Invalid TX type");
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
        let flows = sqlx::query_as("SELECT * FROM auth_flows WHERE realm_id = ? ORDER BY alias ")
            .bind(realm_id.to_string())
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(flows)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "realms", db_op = "update")
    )]
    async fn update_flow_binding<'a>(
        &self,
        realm_id: &Uuid,
        slot: &str,
        flow_id: &Uuid,
        tx: Option<&'a mut dyn Transaction>,
    ) -> Result<()> {
        // 1. Whitelist valid columns to prevent SQL injection
        let valid_slots = [
            "browser_flow_id",
            "registration_flow_id",
            "direct_grant_flow_id",
            "reset_credentials_flow_id",
            "client_authentication_flow_id",
            "docker_authentication_flow_id",
        ];

        if !valid_slots.contains(&slot) {
            return Err(Error::Validation(format!("Invalid binding slot: {}", slot)));
        }

        // 2. Construct Query
        // note: We use ? for sqlite binding placeholders
        let sql = format!("UPDATE realms SET {} = ? WHERE id = ?", slot);

        // 3. Prepare the query object
        // We bind the values as Strings, matching your 'create' method pattern
        let query = sqlx::query(&sql)
            .bind(flow_id.to_string())
            .bind(realm_id.to_string());

        // 4. Execute on correct target (Transaction or Pool)
        if let Some(t) = tx {
            let sql_tx = SqliteTransaction::from_trait(t).expect("Invalid TX type");
            query.execute(&mut **sql_tx).await
        } else {
            query.execute(&*self.pool).await
        }
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(())
    }
}
