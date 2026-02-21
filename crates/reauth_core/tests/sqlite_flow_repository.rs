mod support;

use anyhow::Result;
use reauth_core::adapters::persistence::connection::Database;
use reauth_core::adapters::persistence::sqlite_flow_repository::SqliteFlowRepository;
use reauth_core::adapters::persistence::transaction::SqliteTransactionManager;
use reauth_core::domain::auth_flow::AuthFlow;
use reauth_core::domain::realm::Realm;
use reauth_core::ports::flow_repository::FlowRepository;
use reauth_core::ports::transaction_manager::TransactionManager;
use support::TestDb;
use uuid::Uuid;

fn realm(id: Uuid, name: &str) -> Realm {
    Realm {
        id,
        name: name.to_string(),
        access_token_ttl_secs: 900,
        refresh_token_ttl_secs: 604800,
        browser_flow_id: None,
        registration_flow_id: None,
        direct_grant_flow_id: None,
        reset_credentials_flow_id: None,
    }
}

fn flow(id: Uuid, realm_id: Uuid, name: &str, alias: &str) -> AuthFlow {
    AuthFlow {
        id,
        realm_id,
        name: name.to_string(),
        description: Some(format!("{} description", name)),
        alias: alias.to_string(),
        r#type: "browser".to_string(),
        built_in: false,
    }
}

async fn insert_realm(pool: &Database, realm: &Realm) -> Result<()> {
    sqlx::query(
        "INSERT INTO realms (id, name, access_token_ttl_secs, refresh_token_ttl_secs) VALUES (?, ?, ?, ?)",
    )
    .bind(realm.id.to_string())
    .bind(&realm.name)
    .bind(realm.access_token_ttl_secs)
    .bind(realm.refresh_token_ttl_secs)
    .execute(&**pool)
    .await?;
    Ok(())
}

#[tokio::test]
async fn create_flow_and_find_by_id_and_name() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteFlowRepository::new(db.pool.clone());

    let realm_id = Uuid::new_v4();
    let realm = realm(realm_id, "realm-flow");
    insert_realm(&db.pool, &realm).await?;

    let flow = flow(Uuid::new_v4(), realm_id, "Browser Flow", "browser");
    repo.create_flow(&flow, None).await?;

    let by_id = repo.find_flow_by_id(&flow.id).await?.unwrap();
    assert_eq!(by_id.name, "Browser Flow");

    let by_name = repo.find_flow_by_name(&realm_id, "Browser Flow").await?;
    assert_eq!(by_name.unwrap().id, flow.id);
    Ok(())
}

#[tokio::test]
async fn create_flow_with_transaction() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteFlowRepository::new(db.pool.clone());
    let tx_manager = SqliteTransactionManager::new(db.pool.clone());

    let realm_id = Uuid::new_v4();
    let realm = realm(realm_id, "realm-flow-tx");
    insert_realm(&db.pool, &realm).await?;

    let flow = flow(Uuid::new_v4(), realm_id, "Tx Flow", "tx");
    let mut tx = tx_manager.begin().await?;
    repo.create_flow(&flow, Some(tx.as_mut())).await?;
    tx_manager.commit(tx).await?;

    let by_id = repo.find_flow_by_id(&flow.id).await?.unwrap();
    assert_eq!(by_id.alias, "tx");
    Ok(())
}

#[tokio::test]
async fn list_flows_by_realm_sorted_by_alias() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteFlowRepository::new(db.pool.clone());

    let realm_id = Uuid::new_v4();
    let realm = realm(realm_id, "realm-flow-list");
    insert_realm(&db.pool, &realm).await?;

    let flow_b = flow(Uuid::new_v4(), realm_id, "Flow B", "b-alias");
    let flow_a = flow(Uuid::new_v4(), realm_id, "Flow A", "a-alias");
    repo.create_flow(&flow_b, None).await?;
    repo.create_flow(&flow_a, None).await?;

    let flows = repo.list_flows_by_realm(&realm_id).await?;
    assert_eq!(flows.len(), 2);
    assert_eq!(flows[0].alias, "a-alias");
    assert_eq!(flows[1].alias, "b-alias");
    Ok(())
}
