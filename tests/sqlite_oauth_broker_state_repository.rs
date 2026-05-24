mod support;

use anyhow::Result;
use chrono::{Duration, Utc};
use reauth::adapters::persistence::connection::Database;
use reauth::adapters::persistence::sqlite_oauth_broker_state_repository::SqliteOAuthBrokerStateRepository;
use reauth::domain::identity_provider::OAuthBrokerState;
use reauth::ports::oauth_broker_state_repository::OAuthBrokerStateRepository;
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

async fn insert_auth_session(pool: &Database, realm_id: Uuid, session_id: Uuid) -> Result<()> {
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO auth_sessions (id, realm_id, flow_version_id, current_node_id, context, status, user_id, created_at, updated_at, expires_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(session_id.to_string())
    .bind(realm_id.to_string())
    .bind(Uuid::new_v4().to_string())
    .bind("start")
    .bind("{}")
    .bind("active")
    .bind(Option::<String>::None)
    .bind(now)
    .bind(now)
    .bind(now + Duration::minutes(10))
    .execute(&**pool)
    .await?;
    Ok(())
}

async fn insert_identity_provider(
    pool: &Database,
    realm_id: Uuid,
    provider_id: Uuid,
) -> Result<()> {
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO identity_providers (
            id, realm_id, alias, display_name, protocol, enabled, client_id, scopes_json,
            claim_mapping_json, pkce_required, allow_login, allow_link, allow_jit_provisioning,
            allow_email_auto_link, require_verified_email, sort_order, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(provider_id.to_string())
    .bind(realm_id.to_string())
    .bind("github")
    .bind("GitHub")
    .bind("oauth2")
    .bind(true)
    .bind("client-github")
    .bind("[\"read:user\",\"user:email\"]")
    .bind("{}")
    .bind(true)
    .bind(true)
    .bind(true)
    .bind(false)
    .bind(true)
    .bind(true)
    .bind(0_i64)
    .bind(now)
    .bind(now)
    .execute(&**pool)
    .await?;
    Ok(())
}

#[tokio::test]
async fn delete_expired_before_is_bounded_by_batch_size() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteOAuthBrokerStateRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();
    let provider_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    insert_realm(&db.pool, realm_id, "realm-oauth-broker").await?;
    insert_auth_session(&db.pool, realm_id, session_id).await?;
    insert_identity_provider(&db.pool, realm_id, provider_id).await?;

    let now = Utc::now();
    for idx in 0..3 {
        let created_at = now - Duration::minutes(15 + idx);
        repo.create(&OAuthBrokerState {
            id: Uuid::new_v4(),
            realm_id,
            provider_id,
            auth_session_id: session_id,
            pkce_verifier_hash: format!("hash-{}", idx),
            redirect_uri: "http://localhost/callback".to_string(),
            nonce: Some(format!("nonce-{}", idx)),
            expires_at: now - Duration::minutes(5),
            consumed_at: None,
            created_at,
            updated_at: created_at,
        })
        .await?;
    }

    let removed = repo.delete_expired_before(now, 2).await?;
    assert_eq!(removed, 2);

    let remaining: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM oauth_broker_states")
        .fetch_one(&*db.pool)
        .await?;
    assert_eq!(remaining.0, 1);

    let removed_rest = repo.delete_expired_before(now, 2).await?;
    assert_eq!(removed_rest, 1);

    Ok(())
}
