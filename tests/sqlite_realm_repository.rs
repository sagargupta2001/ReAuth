mod support;

use anyhow::Result;
use reauth::adapters::persistence::connection::Database;
use reauth::adapters::persistence::sqlite_realm_repository::SqliteRealmRepository;
use reauth::adapters::persistence::transaction::SqliteTransactionManager;
use reauth::domain::auth_flow::AuthFlow;
use reauth::domain::realm::Realm;
use reauth::error::Error;
use reauth::ports::realm_repository::RealmRepository;
use reauth::ports::transaction_manager::TransactionManager;
use support::TestDb;
use uuid::Uuid;

fn realm(id: Uuid, name: &str) -> Realm {
    Realm {
        id,
        name: name.to_string(),
        access_token_ttl_secs: 900,
        refresh_token_ttl_secs: 604800,
        pkce_required_public_clients: true,
        lockout_threshold: 5,
        lockout_duration_secs: 900,
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

async fn insert_flow(pool: &Database, flow: &AuthFlow) -> Result<()> {
    sqlx::query(
        "INSERT INTO auth_flows (id, realm_id, name, alias, type, built_in, description) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(flow.id.to_string())
    .bind(flow.realm_id.to_string())
    .bind(&flow.name)
    .bind(&flow.alias)
    .bind(&flow.r#type)
    .bind(flow.built_in)
    .bind(&flow.description)
    .execute(&**pool)
    .await?;
    Ok(())
}

#[tokio::test]
async fn create_find_list_and_update_realm() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteRealmRepository::new(db.pool.clone());

    let realm_id = Uuid::new_v4();
    let mut realm = realm(realm_id, "realm-one");

    repo.create(&realm, None).await?;

    let by_id = repo.find_by_id(&realm_id).await?.unwrap();
    assert_eq!(by_id.name, "realm-one");

    let by_name = repo.find_by_name("realm-one").await?.unwrap();
    assert_eq!(by_name.id, realm_id);

    let all = repo.list_all().await?;
    assert_eq!(all.len(), 1);

    let browser_flow_id = Uuid::new_v4();
    realm.name = "realm-updated".to_string();
    realm.access_token_ttl_secs = 1200;
    realm.refresh_token_ttl_secs = 700000;
    realm.browser_flow_id = Some(browser_flow_id.to_string());

    repo.update(&realm, None).await?;

    let updated = repo.find_by_id(&realm_id).await?.unwrap();
    assert_eq!(updated.name, "realm-updated");
    assert_eq!(updated.access_token_ttl_secs, 1200);
    assert_eq!(updated.browser_flow_id, Some(browser_flow_id.to_string()));
    Ok(())
}

#[tokio::test]
async fn create_and_update_with_transaction_and_flow_binding() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteRealmRepository::new(db.pool.clone());
    let tx_manager = SqliteTransactionManager::new(db.pool.clone());

    let realm_id = Uuid::new_v4();
    let mut realm = realm(realm_id, "realm-tx");

    let mut tx = tx_manager.begin().await?;
    repo.create(&realm, Some(tx.as_mut())).await?;
    tx_manager.commit(tx).await?;

    let mut tx = tx_manager.begin().await?;
    realm.name = "realm-tx-updated".to_string();
    repo.update(&realm, Some(tx.as_mut())).await?;
    tx_manager.commit(tx).await?;

    let flow_id = Uuid::new_v4();
    let mut tx = tx_manager.begin().await?;
    repo.update_flow_binding(&realm_id, "browser_flow_id", &flow_id, Some(tx.as_mut()))
        .await?;
    tx_manager.commit(tx).await?;

    let updated = repo.find_by_id(&realm_id).await?.unwrap();
    assert_eq!(updated.name, "realm-tx-updated");
    assert_eq!(updated.browser_flow_id, Some(flow_id.to_string()));
    Ok(())
}

#[tokio::test]
async fn update_flow_binding_rejects_invalid_slot() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteRealmRepository::new(db.pool.clone());

    let realm_id = Uuid::new_v4();
    let realm = realm(realm_id, "realm-slot");
    repo.create(&realm, None).await?;

    let flow_id = Uuid::new_v4();
    let err = repo
        .update_flow_binding(&realm_id, "invalid_slot", &flow_id, None)
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Validation(_)));
    Ok(())
}

#[tokio::test]
async fn list_flows_by_realm_returns_sorted_aliases() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteRealmRepository::new(db.pool.clone());

    let realm_id = Uuid::new_v4();
    let realm = realm(realm_id, "realm-flows");
    repo.create(&realm, None).await?;

    let flow_b = flow(Uuid::new_v4(), realm_id, "Flow B", "b-alias");
    let flow_a = flow(Uuid::new_v4(), realm_id, "Flow A", "a-alias");
    insert_flow(&db.pool, &flow_b).await?;
    insert_flow(&db.pool, &flow_a).await?;

    let flows = repo.list_flows_by_realm(&realm_id).await?;
    assert_eq!(flows.len(), 2);
    assert_eq!(flows[0].alias, "a-alias");
    assert_eq!(flows[1].alias, "b-alias");
    Ok(())
}
