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
use serde_json::Value;
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

#[derive(sqlx::FromRow)]
struct RealmRecord {
    id: String,
    name: String,
    access_token_ttl_secs: i64,
    refresh_token_ttl_secs: i64,
    pkce_required_public_clients: bool,
    lockout_threshold: i64,
    lockout_duration_secs: i64,
    is_system: bool,
    registration_enabled: bool,
    default_registration_role_ids: String,
    browser_flow_id: Option<String>,
    registration_flow_id: Option<String>,
    direct_grant_flow_id: Option<String>,
    reset_credentials_flow_id: Option<String>,
}

impl RealmRecord {
    fn into_realm(self) -> Result<Realm> {
        let id =
            Uuid::parse_str(&self.id).map_err(|_| Error::System("Invalid realm id".to_string()))?;
        let role_ids = parse_role_ids(&self.default_registration_role_ids)?;
        Ok(Realm {
            id,
            name: self.name,
            access_token_ttl_secs: self.access_token_ttl_secs,
            refresh_token_ttl_secs: self.refresh_token_ttl_secs,
            pkce_required_public_clients: self.pkce_required_public_clients,
            lockout_threshold: self.lockout_threshold,
            lockout_duration_secs: self.lockout_duration_secs,
            is_system: self.is_system,
            registration_enabled: self.registration_enabled,
            default_registration_role_ids: role_ids,
            browser_flow_id: self.browser_flow_id,
            registration_flow_id: self.registration_flow_id,
            direct_grant_flow_id: self.direct_grant_flow_id,
            reset_credentials_flow_id: self.reset_credentials_flow_id,
        })
    }
}

fn parse_role_ids(raw: &str) -> Result<Vec<Uuid>> {
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }
    let parsed: Value = serde_json::from_str(raw)
        .map_err(|err| Error::System(format!("Invalid realm role ids: {}", err)))?;
    let array = parsed.as_array().ok_or_else(|| {
        Error::System("default_registration_role_ids must be a JSON array".to_string())
    })?;
    let mut role_ids = Vec::with_capacity(array.len());
    for entry in array {
        let Some(value) = entry.as_str() else {
            return Err(Error::System(
                "default_registration_role_ids must contain strings".to_string(),
            ));
        };
        let role_id = Uuid::parse_str(value)
            .map_err(|_| Error::System("Invalid role id in registration defaults".to_string()))?;
        role_ids.push(role_id);
    }
    Ok(role_ids)
}

fn serialize_role_ids(role_ids: &[Uuid]) -> Result<String> {
    serde_json::to_string(role_ids)
        .map_err(|err| Error::System(format!("Failed to serialize role ids: {}", err)))
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
                pkce_required_public_clients, lockout_threshold, lockout_duration_secs,
                is_system, registration_enabled, default_registration_role_ids,
                browser_flow_id, registration_flow_id, direct_grant_flow_id, reset_credentials_flow_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
            .bind(realm.id.to_string())
            .bind(&realm.name)
            .bind(realm.access_token_ttl_secs)
            .bind(realm.refresh_token_ttl_secs)
            .bind(realm.pkce_required_public_clients)
            .bind(realm.lockout_threshold)
            .bind(realm.lockout_duration_secs)
            .bind(realm.is_system)
            .bind(realm.registration_enabled)
            .bind(serialize_role_ids(&realm.default_registration_role_ids)?)
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
        let record: Option<RealmRecord> = sqlx::query_as("SELECT * FROM realms WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        record.map(RealmRecord::into_realm).transpose()
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "realms", db_op = "select")
    )]
    async fn find_by_name(&self, name: &str) -> Result<Option<Realm>> {
        let record: Option<RealmRecord> = sqlx::query_as("SELECT * FROM realms WHERE name = ?")
            .bind(name)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        record.map(RealmRecord::into_realm).transpose()
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "realms", db_op = "select")
    )]
    async fn list_all(&self) -> Result<Vec<Realm>> {
        let records: Vec<RealmRecord> = sqlx::query_as("SELECT * FROM realms")
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        records.into_iter().map(RealmRecord::into_realm).collect()
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
                pkce_required_public_clients = ?,
                lockout_threshold = ?,
                lockout_duration_secs = ?,
                is_system = ?,
                registration_enabled = ?,
                default_registration_role_ids = ?,
                browser_flow_id = ?,
                registration_flow_id = ?,
                direct_grant_flow_id = ?,
                reset_credentials_flow_id = ?
             WHERE id = ?",
        )
        .bind(&realm.name)
        .bind(realm.access_token_ttl_secs)
        .bind(realm.refresh_token_ttl_secs)
        .bind(realm.pkce_required_public_clients)
        .bind(realm.lockout_threshold)
        .bind(realm.lockout_duration_secs)
        .bind(realm.is_system)
        .bind(realm.registration_enabled)
        .bind(serialize_role_ids(&realm.default_registration_role_ids)?)
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
