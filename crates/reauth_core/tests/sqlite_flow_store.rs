mod support;

use anyhow::Result;
use chrono::{Duration, Utc};
use reauth_core::adapters::persistence::connection::Database;
use reauth_core::adapters::persistence::sqlite_flow_store::SqliteFlowStore;
use reauth_core::adapters::persistence::transaction::SqliteTransactionManager;
use reauth_core::domain::flow::models::{FlowDeployment, FlowDraft, FlowVersion};
use reauth_core::domain::pagination::{PageRequest, SortDirection};
use reauth_core::domain::realm::Realm;
use reauth_core::ports::flow_store::FlowStore;
use reauth_core::ports::transaction_manager::TransactionManager;
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

async fn insert_flow(pool: &Database, flow_id: &str, realm_id: Uuid, name: &str) -> Result<()> {
    sqlx::query(
        "INSERT INTO auth_flows (id, realm_id, name, alias, type, built_in, description) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(flow_id)
    .bind(realm_id.to_string())
    .bind(name)
    .bind(name)
    .bind("browser")
    .bind(false)
    .bind(Option::<String>::None)
    .execute(&**pool)
    .await?;
    Ok(())
}

fn draft(id: Uuid, realm_id: Uuid, name: &str, updated_at: chrono::DateTime<Utc>) -> FlowDraft {
    FlowDraft {
        id,
        realm_id,
        name: name.to_string(),
        description: Some(format!("{} description", name)),
        graph_json: "{}".to_string(),
        flow_type: "browser".to_string(),
        created_at: updated_at,
        updated_at,
    }
}

fn version(
    id: &str,
    flow_id: &str,
    version_number: i32,
    created_at: chrono::DateTime<Utc>,
) -> FlowVersion {
    FlowVersion {
        id: id.to_string(),
        flow_id: flow_id.to_string(),
        version_number,
        execution_artifact: format!("artifact-{}", version_number),
        graph_json: "{}".to_string(),
        checksum: format!("chk-{}", version_number),
        created_at,
    }
}

fn deployment(
    id: &str,
    realm_id: Uuid,
    flow_type: &str,
    version_id: &str,
    updated_at: chrono::DateTime<Utc>,
) -> FlowDeployment {
    FlowDeployment {
        id: id.to_string(),
        realm_id,
        flow_type: flow_type.to_string(),
        active_version_id: version_id.to_string(),
        updated_at,
    }
}

fn page_request(
    page: i64,
    per_page: i64,
    sort_by: Option<&str>,
    sort_dir: Option<SortDirection>,
    q: Option<&str>,
) -> PageRequest {
    PageRequest {
        page,
        per_page,
        sort_by: sort_by.map(|value| value.to_string()),
        sort_dir,
        q: q.map(|value| value.to_string()),
    }
}

#[tokio::test]
async fn draft_crud_and_listing() -> Result<()> {
    let db = TestDb::new().await;
    let store = SqliteFlowStore::new(db.pool.clone());

    let realm_id = Uuid::new_v4();
    let realm_entity = realm(realm_id, "realm-drafts");
    insert_realm(&db.pool, &realm_entity).await?;

    let now = Utc::now();
    let draft_a = draft(
        Uuid::new_v4(),
        realm_id,
        "Alpha",
        now - Duration::minutes(10),
    );
    let draft_b = draft(Uuid::new_v4(), realm_id, "Beta", now - Duration::minutes(5));
    let draft_c = draft(Uuid::new_v4(), realm_id, "Gamma", now);

    store.create_draft(&draft_a).await?;
    store.create_draft(&draft_b).await?;
    store.create_draft(&draft_c).await?;

    let fetched = store.get_draft_by_id(&draft_b.id).await?.unwrap();
    assert_eq!(fetched.name, "Beta");

    let mut updated = draft_b.clone();
    updated.name = "Beta Updated".to_string();
    updated.description = None;
    updated.graph_json = "{\"nodes\":[]}".to_string();
    updated.updated_at = now + Duration::minutes(1);
    store.update_draft(&updated).await?;

    let refreshed = store.get_draft_by_id(&draft_b.id).await?.unwrap();
    assert_eq!(refreshed.name, "Beta Updated");
    assert!(refreshed.description.is_none());

    let req = page_request(1, 2, Some("updated_at"), Some(SortDirection::Desc), None);
    let page = store.list_drafts(&realm_id, &req).await?;
    assert_eq!(page.meta.total, 3);
    assert_eq!(page.data.len(), 2);
    assert_eq!(page.data[0].name, "Beta Updated");

    let filtered = store
        .list_drafts(
            &realm_id,
            &page_request(1, 10, Some("name"), Some(SortDirection::Asc), Some("Ga")),
        )
        .await?;
    assert_eq!(filtered.meta.total, 1);
    assert_eq!(filtered.data[0].name, "Gamma");

    let all = store.list_all_drafts(&realm_id).await?;
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].name, "Gamma");

    store.delete_draft(&draft_a.id).await?;
    let missing = store.get_draft_by_id(&draft_a.id).await?;
    assert!(missing.is_none());
    Ok(())
}

#[tokio::test]
async fn draft_operations_with_transactions() -> Result<()> {
    let db = TestDb::new().await;
    let store = SqliteFlowStore::new(db.pool.clone());
    let tx_manager = SqliteTransactionManager::new(db.pool.clone());

    let realm_id = Uuid::new_v4();
    let realm_entity = realm(realm_id, "realm-drafts-tx");
    insert_realm(&db.pool, &realm_entity).await?;

    let now = Utc::now();
    let draft_tx = draft(Uuid::new_v4(), realm_id, "Tx Draft", now);

    let mut tx = tx_manager.begin().await?;
    store
        .create_draft_with_tx(&draft_tx, Some(tx.as_mut()))
        .await?;
    tx_manager.commit(tx).await?;

    let mut updated = draft_tx.clone();
    updated.name = "Tx Draft Updated".to_string();
    updated.updated_at = now + Duration::minutes(2);

    let mut tx = tx_manager.begin().await?;
    store
        .update_draft_with_tx(&updated, Some(tx.as_mut()))
        .await?;
    tx_manager.commit(tx).await?;

    let fetched = store.get_draft_by_id(&draft_tx.id).await?.unwrap();
    assert_eq!(fetched.name, "Tx Draft Updated");

    let mut tx = tx_manager.begin().await?;
    store
        .delete_draft_with_tx(&draft_tx.id, Some(tx.as_mut()))
        .await?;
    tx_manager.commit(tx).await?;

    let missing = store.get_draft_by_id(&draft_tx.id).await?;
    assert!(missing.is_none());
    Ok(())
}

#[tokio::test]
async fn version_and_deployment_queries() -> Result<()> {
    let db = TestDb::new().await;
    let store = SqliteFlowStore::new(db.pool.clone());
    let tx_manager = SqliteTransactionManager::new(db.pool.clone());

    let realm_id = Uuid::new_v4();
    let realm_entity = realm(realm_id, "realm-versions");
    insert_realm(&db.pool, &realm_entity).await?;

    let flow_id = Uuid::new_v4().to_string();
    insert_flow(&db.pool, &flow_id, realm_id, "Browser Flow").await?;

    let now = Utc::now();
    let v1 = version(
        &Uuid::new_v4().to_string(),
        &flow_id,
        1,
        now - Duration::minutes(10),
    );
    let v2 = version(
        &Uuid::new_v4().to_string(),
        &flow_id,
        2,
        now - Duration::minutes(5),
    );
    let v3 = version(&Uuid::new_v4().to_string(), &flow_id, 3, now);

    store.create_version(&v1).await?;
    let mut tx = tx_manager.begin().await?;
    store.create_version_with_tx(&v2, Some(tx.as_mut())).await?;
    tx_manager.commit(tx).await?;
    store.create_version(&v3).await?;

    let fetched = store.get_version(&Uuid::parse_str(&v2.id)?).await?.unwrap();
    assert_eq!(fetched.version_number, 2);

    let page = store
        .list_versions(
            &Uuid::parse_str(&flow_id)?,
            &page_request(
                1,
                2,
                Some("version_number"),
                Some(SortDirection::Desc),
                None,
            ),
        )
        .await?;
    assert_eq!(page.meta.total, 3);
    assert_eq!(page.data[0].version_number, 3);

    let latest_number = store
        .get_latest_version_number(&Uuid::parse_str(&flow_id)?)
        .await?;
    assert_eq!(latest_number, Some(3));

    let latest = store
        .get_latest_version(&Uuid::parse_str(&flow_id)?)
        .await?
        .unwrap();
    assert_eq!(latest.version_number, 3);

    let deployment = deployment(
        &Uuid::new_v4().to_string(),
        realm_id,
        "browser",
        &v3.id,
        now,
    );
    store.set_deployment(&deployment).await?;
    let fetched_dep = store.get_deployment(&realm_id, "browser").await?.unwrap();
    assert_eq!(fetched_dep.active_version_id, v3.id);

    let version_number = store
        .get_deployed_version_number(&realm_id, "browser", &Uuid::parse_str(&flow_id)?)
        .await?;
    assert_eq!(version_number, Some(3));

    let flow_id_other = Uuid::new_v4().to_string();
    insert_flow(&db.pool, &flow_id_other, realm_id, "Other Flow").await?;
    let mismatch = store
        .get_deployed_version_number(&realm_id, "browser", &Uuid::parse_str(&flow_id_other)?)
        .await?;
    assert!(mismatch.is_none());

    let by_number = store
        .get_version_by_number(&Uuid::parse_str(&flow_id)?, 2)
        .await?
        .unwrap();
    assert_eq!(by_number.id, v2.id);

    let active = store
        .get_active_version(&Uuid::parse_str(&flow_id)?)
        .await?
        .unwrap();
    assert_eq!(active.id, v3.id);

    Ok(())
}

#[tokio::test]
async fn deployment_with_transaction_updates_active_version() -> Result<()> {
    let db = TestDb::new().await;
    let store = SqliteFlowStore::new(db.pool.clone());
    let tx_manager = SqliteTransactionManager::new(db.pool.clone());

    let realm_id = Uuid::new_v4();
    let realm_entity = realm(realm_id, "realm-deploy");
    insert_realm(&db.pool, &realm_entity).await?;

    let flow_id = Uuid::new_v4().to_string();
    insert_flow(&db.pool, &flow_id, realm_id, "Browser Flow").await?;

    let now = Utc::now();
    let v1 = version(
        &Uuid::new_v4().to_string(),
        &flow_id,
        1,
        now - Duration::minutes(5),
    );
    let v2 = version(&Uuid::new_v4().to_string(), &flow_id, 2, now);
    store.create_version(&v1).await?;
    store.create_version(&v2).await?;

    let first = deployment(
        &Uuid::new_v4().to_string(),
        realm_id,
        "browser",
        &v1.id,
        now,
    );
    let mut tx = tx_manager.begin().await?;
    store
        .set_deployment_with_tx(&first, Some(tx.as_mut()))
        .await?;
    tx_manager.commit(tx).await?;

    let updated = deployment(
        &Uuid::new_v4().to_string(),
        realm_id,
        "browser",
        &v2.id,
        now + Duration::minutes(1),
    );
    let mut tx = tx_manager.begin().await?;
    store
        .set_deployment_with_tx(&updated, Some(tx.as_mut()))
        .await?;
    tx_manager.commit(tx).await?;

    let fetched = store.get_deployment(&realm_id, "browser").await?.unwrap();
    assert_eq!(fetched.active_version_id, v2.id);
    Ok(())
}
