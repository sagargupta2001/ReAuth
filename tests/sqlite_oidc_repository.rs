mod support;

use anyhow::Result;
use chrono::{Duration, Utc};
use reauth::adapters::persistence::connection::Database;
use reauth::adapters::persistence::sqlite_oidc_repository::SqliteOidcRepository;
use reauth::domain::oidc::{AuthCode, OidcClient};
use reauth::domain::pagination::{PageRequest, SortDirection};
use reauth::domain::realm::{Realm, RealmIdpDefaultEmailLinkPolicy, RealmIdpDefaultJitPolicy};
use reauth::error::Error;
use reauth::ports::oidc_repository::OidcRepository;
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
        is_system: false,
        registration_enabled: true,
        default_registration_role_ids: Vec::new(),
        invitation_resend_limit: 3,
        idp_broker_enabled: false,
        idp_default_jit_policy: RealmIdpDefaultJitPolicy::PerProvider,
        idp_default_email_link_policy: RealmIdpDefaultEmailLinkPolicy::ManualOnly,
        idp_minimum_remaining_factor: true,
        browser_flow_id: None,
        registration_flow_id: None,
        direct_grant_flow_id: None,
        reset_credentials_flow_id: None,
        invitation_flow_id: None,
    }
}

fn client(id: Uuid, realm_id: Uuid, client_id: &str, origins: &str) -> OidcClient {
    OidcClient {
        id,
        realm_id,
        client_id: client_id.to_string(),
        client_secret: Some("secret".to_string()),
        redirect_uris: "[]".to_string(),
        scopes: "[\"openid\"]".to_string(),
        web_origins: origins.to_string(),
        managed_by_config: false,
    }
}

fn page_request(
    page: i64,
    per_page: i64,
    sort_dir: Option<SortDirection>,
    q: Option<&str>,
) -> PageRequest {
    PageRequest {
        page,
        per_page,
        sort_by: Some("client_id".to_string()),
        sort_dir,
        q: q.map(|value| value.to_string()),
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
async fn create_find_and_update_client() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteOidcRepository::new(db.pool.clone());

    let realm_id = Uuid::new_v4();
    let realm_entity = realm(realm_id, "realm-oidc");
    insert_realm(&db.pool, &realm_entity).await?;

    let client_id = Uuid::new_v4();
    let mut client = client(
        client_id,
        realm_id,
        "client-a",
        "[\"http://localhost:3000\"]",
    );
    repo.create_client(&client).await?;

    let by_id = repo
        .find_client_by_id(&realm_id, "client-a")
        .await?
        .unwrap();
    assert_eq!(by_id.id, client_id);

    let by_uuid = repo.find_client_by_uuid(&client_id).await?.unwrap();
    assert_eq!(by_uuid.client_id, "client-a");

    client.client_id = "client-a-updated".to_string();
    client.redirect_uris = "[\"https://example.com/callback\"]".to_string();
    client.scopes = "[\"openid\", \"profile\"]".to_string();
    client.web_origins = "[\"https://example.com\"]".to_string();
    client.managed_by_config = true;
    repo.update_client(&client).await?;

    let updated = repo.find_client_by_uuid(&client_id).await?.unwrap();
    assert_eq!(updated.client_id, "client-a-updated");
    assert!(updated.managed_by_config);
    Ok(())
}

#[tokio::test]
async fn list_clients_with_filters_and_pagination() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteOidcRepository::new(db.pool.clone());

    let realm_id = Uuid::new_v4();
    let realm_entity = realm(realm_id, "realm-list");
    insert_realm(&db.pool, &realm_entity).await?;

    let other_realm_id = Uuid::new_v4();
    let other_realm_entity = realm(other_realm_id, "realm-other");
    insert_realm(&db.pool, &other_realm_entity).await?;

    let alpha = client(Uuid::new_v4(), realm_id, "alpha", "[]");
    let beta = client(Uuid::new_v4(), realm_id, "beta", "[]");
    let gamma = client(Uuid::new_v4(), realm_id, "gamma", "[]");
    let other = client(Uuid::new_v4(), other_realm_id, "other", "[]");

    for c in [&alpha, &beta, &gamma, &other] {
        repo.create_client(c).await?;
    }

    let page1 = repo
        .find_clients_by_realm(
            &realm_id,
            &page_request(1, 2, Some(SortDirection::Asc), None),
        )
        .await?;
    assert_eq!(page1.meta.total, 3);
    assert_eq!(page1.data.len(), 2);
    assert_eq!(page1.data[0].client_id, "alpha");

    let page2 = repo
        .find_clients_by_realm(
            &realm_id,
            &page_request(2, 2, Some(SortDirection::Asc), None),
        )
        .await?;
    assert_eq!(page2.meta.total, 3);
    assert_eq!(page2.data.len(), 1);
    assert_eq!(page2.data[0].client_id, "gamma");

    let filtered = repo
        .find_clients_by_realm(
            &realm_id,
            &page_request(1, 10, Some(SortDirection::Asc), Some("be")),
        )
        .await?;
    assert_eq!(filtered.meta.total, 1);
    assert_eq!(filtered.data[0].client_id, "beta");

    let desc = repo
        .find_clients_by_realm(
            &realm_id,
            &page_request(1, 3, Some(SortDirection::Desc), None),
        )
        .await?;
    assert_eq!(desc.data[0].client_id, "gamma");
    Ok(())
}

#[tokio::test]
async fn count_client_stats_splits_confidential_and_public() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteOidcRepository::new(db.pool.clone());

    let realm_id = Uuid::new_v4();
    let realm_entity = realm(realm_id, "realm-stats");
    insert_realm(&db.pool, &realm_entity).await?;

    let other_realm_id = Uuid::new_v4();
    let other_realm_entity = realm(other_realm_id, "realm-stats-other");
    insert_realm(&db.pool, &other_realm_entity).await?;

    // Two confidential clients (helper sets a secret) ...
    repo.create_client(&client(Uuid::new_v4(), realm_id, "conf-a", "[]"))
        .await?;
    repo.create_client(&client(Uuid::new_v4(), realm_id, "conf-b", "[]"))
        .await?;

    // ... and one public client (no secret).
    let mut public_client = client(Uuid::new_v4(), realm_id, "public-a", "[]");
    public_client.client_secret = None;
    repo.create_client(&public_client).await?;

    // A client in another realm must not leak into the counts.
    repo.create_client(&client(Uuid::new_v4(), other_realm_id, "other", "[]"))
        .await?;

    let stats = repo.count_client_stats(&realm_id).await?;
    assert_eq!(stats.total, 3);
    assert_eq!(stats.confidential, 2);
    assert_eq!(stats.public, 1);
    Ok(())
}

#[tokio::test]
async fn update_client_persists_scopes() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteOidcRepository::new(db.pool.clone());

    let realm_id = Uuid::new_v4();
    insert_realm(&db.pool, &realm(realm_id, "realm-scopes")).await?;

    let client_id = Uuid::new_v4();
    let mut entity = client(client_id, realm_id, "scoped", "[]");
    repo.create_client(&entity).await?;

    entity.scopes = "[\"openid\",\"profile\",\"email\",\"offline_access\"]".to_string();
    repo.update_client(&entity).await?;

    let stored = repo.find_client_by_uuid(&client_id).await?.unwrap();
    assert_eq!(
        stored.scopes,
        "[\"openid\",\"profile\",\"email\",\"offline_access\"]"
    );
    Ok(())
}

#[tokio::test]
async fn delete_client_reports_impact_and_removes_record() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteOidcRepository::new(db.pool.clone());

    let realm_id = Uuid::new_v4();
    insert_realm(&db.pool, &realm(realm_id, "realm-del")).await?;

    let client_id = Uuid::new_v4();
    repo.create_client(&client(client_id, realm_id, "doomed", "[]"))
        .await?;

    // Two client-scoped roles and one client-scoped custom permission.
    for name in ["role-a", "role-b"] {
        sqlx::query("INSERT INTO roles (id, realm_id, client_id, name) VALUES (?, ?, ?, ?)")
            .bind(Uuid::new_v4().to_string())
            .bind(realm_id.to_string())
            .bind(client_id.to_string())
            .bind(name)
            .execute(&*db.pool)
            .await?;
    }
    sqlx::query(
        "INSERT INTO custom_permissions (id, realm_id, client_id, permission, name) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(realm_id.to_string())
    .bind(client_id.to_string())
    .bind("custom:do")
    .bind("Custom Do")
    .execute(&*db.pool)
    .await?;

    let summary = repo.count_client_delete_summary(&client_id).await?;
    assert_eq!(summary.name, "doomed");
    assert_eq!(summary.role_count, 2);
    assert_eq!(summary.permission_count, 1);

    repo.delete_client(&client_id).await?;
    assert!(repo.find_client_by_uuid(&client_id).await?.is_none());

    Ok(())
}

#[tokio::test]
async fn auth_code_lifecycle_and_expiration() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteOidcRepository::new(db.pool.clone());

    let code = AuthCode {
        code: "code-123".to_string(),
        user_id: Uuid::new_v4(),
        client_id: "client-a".to_string(),
        redirect_uri: "https://example.com/callback".to_string(),
        nonce: Some("nonce".to_string()),
        code_challenge: Some("challenge".to_string()),
        code_challenge_method: "S256".to_string(),
        expires_at: Utc::now() + Duration::minutes(10),
    };

    repo.save_auth_code(&code).await?;
    let fetched = repo.find_auth_code_by_code("code-123").await?.unwrap();
    assert_eq!(fetched.client_id, "client-a");

    repo.delete_auth_code("code-123").await?;
    let missing = repo.find_auth_code_by_code("code-123").await?;
    assert!(missing.is_none());

    let expired = AuthCode {
        code: "code-expired".to_string(),
        user_id: Uuid::new_v4(),
        client_id: "client-a".to_string(),
        redirect_uri: "https://example.com/callback".to_string(),
        nonce: None,
        code_challenge: None,
        code_challenge_method: "plain".to_string(),
        expires_at: Utc::now() - Duration::minutes(5),
    };
    repo.save_auth_code(&expired).await?;
    let expired_fetch = repo.find_auth_code_by_code("code-expired").await?;
    assert!(expired_fetch.is_none());

    let err = repo.delete_auth_code("missing-code").await.unwrap_err();
    assert!(matches!(err, Error::Unexpected(_)));
    Ok(())
}

#[tokio::test]
async fn is_origin_allowed_matches_clients() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteOidcRepository::new(db.pool.clone());

    let realm_id = Uuid::new_v4();
    let realm_entity = realm(realm_id, "realm-origin");
    insert_realm(&db.pool, &realm_entity).await?;

    let client = client(
        Uuid::new_v4(),
        realm_id,
        "origin-client",
        "[\"http://localhost:3000\", \"https://example.com\"]",
    );
    repo.create_client(&client).await?;

    let allowed = repo.is_origin_allowed("http://localhost:3000").await?;
    assert!(allowed);

    let denied = repo.is_origin_allowed("https://not-allowed.com").await?;
    assert!(!denied);
    Ok(())
}
