mod support;

use anyhow::Result;
use chrono::Utc;
use reauth_core::adapters::persistence::connection::Database;
use reauth_core::adapters::persistence::sqlite_webhook_repository::SqliteWebhookRepository;
use reauth_core::domain::webhook::WebhookEndpoint;
use reauth_core::ports::webhook_repository::WebhookRepository;
use std::collections::HashMap;
use support::TestDb;
use uuid::Uuid;

async fn insert_realm(pool: &Database, realm_id: Uuid, name: &str) -> Result<()> {
    sqlx::query(
        "INSERT INTO realms (id, name, access_token_ttl_secs, refresh_token_ttl_secs) VALUES (?, ?, ?, ?)",
    )
    .bind(realm_id.to_string())
    .bind(name)
    .bind(900_i64)
    .bind(604800_i64)
    .execute(&**pool)
    .await?;
    Ok(())
}

fn create_endpoint(realm_id: Uuid) -> WebhookEndpoint {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer secret".to_string());

    WebhookEndpoint {
        id: Uuid::new_v4(),
        realm_id,
        name: "test-webhook".to_string(),
        url: "https://example.com/webhook".to_string(),
        http_method: "POST".to_string(),
        status: "active".to_string(),
        signing_secret: "secret-key".to_string(),
        custom_headers: headers,
        description: Some("Test endpoint".to_string()),
        consecutive_failures: 0,
        last_failure_at: None,
        disabled_at: None,
        disabled_reason: None,
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    }
}

#[tokio::test]
async fn create_and_find_webhook_endpoint() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteWebhookRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();

    insert_realm(&db.pool, realm_id, "realm-webhook").await?;

    let endpoint = create_endpoint(realm_id);

    repo.create_endpoint(&endpoint, None).await?;

    let found = repo.find_endpoint(&realm_id, &endpoint.id).await?.unwrap();
    assert_eq!(found.id, endpoint.id);
    assert_eq!(found.name, "test-webhook");
    assert_eq!(
        found.custom_headers.get("Authorization").unwrap(),
        "Bearer secret"
    );

    Ok(())
}

#[tokio::test]
async fn list_and_search_endpoints() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteWebhookRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();

    insert_realm(&db.pool, realm_id, "realm-webhook").await?;

    let endpoint1 = create_endpoint(realm_id);
    repo.create_endpoint(&endpoint1, None).await?;

    let mut endpoint2 = create_endpoint(realm_id);
    endpoint2.id = Uuid::new_v4();
    endpoint2.name = "another-webhook".to_string();
    endpoint2.url = "https://other.com/webhook".to_string();
    repo.create_endpoint(&endpoint2, None).await?;

    let all = repo.list_endpoints(&realm_id).await?;
    assert_eq!(all.len(), 2);

    let search = repo.search_endpoints(&realm_id, "another", 10).await?;
    assert_eq!(search.len(), 1);
    assert_eq!(search[0].id, endpoint2.id);

    Ok(())
}

#[tokio::test]
async fn subscriptions_management() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteWebhookRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();

    insert_realm(&db.pool, realm_id, "realm-webhook").await?;

    let endpoint = create_endpoint(realm_id);
    repo.create_endpoint(&endpoint, None).await?;

    repo.upsert_subscriptions(
        &endpoint.id,
        &["user.created".to_string(), "user.deleted".to_string()],
        None,
    )
    .await?;

    let subs = repo.list_subscriptions(&endpoint.id).await?;
    assert_eq!(subs.len(), 2);

    // Check setting specific subscription
    repo.set_subscription_enabled(&endpoint.id, "user.created", false, None)
        .await?;
    let updated_subs = repo.list_subscriptions(&endpoint.id).await?;

    let user_created_sub = updated_subs
        .iter()
        .find(|s| s.event_type == "user.created")
        .unwrap();
    assert!(!user_created_sub.enabled);

    Ok(())
}
