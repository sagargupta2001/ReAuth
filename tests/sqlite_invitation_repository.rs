mod support;

use anyhow::Result;
use chrono::{Duration, Utc};
use reauth::adapters::persistence::connection::Database;
use reauth::adapters::persistence::sqlite_invitation_repository::SqliteInvitationRepository;
use reauth::domain::invitation::{Invitation, InvitationStatus};
use reauth::domain::pagination::{PageRequest, SortDirection};
use reauth::ports::invitation_repository::InvitationRepository;
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

fn invitation(realm_id: Uuid, email: &str, status: InvitationStatus) -> Invitation {
    let now = Utc::now();
    Invitation {
        id: Uuid::new_v4(),
        realm_id,
        email: email.to_string(),
        email_normalized: email.to_lowercase(),
        status,
        token_hash: Uuid::new_v4().to_string(),
        expiry_days: 7,
        expires_at: now + Duration::days(7),
        invited_by_user_id: None,
        accepted_user_id: None,
        accepted_at: None,
        revoked_at: None,
        resend_count: 0,
        last_sent_at: Some(now),
        created_at: now,
        updated_at: now,
    }
}

#[tokio::test]
async fn list_invitations_filters_by_multiple_statuses() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteInvitationRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();

    insert_realm(&db.pool, realm_id, "realm-invitations-list").await?;

    let pending = invitation(realm_id, "pending@example.com", InvitationStatus::Pending);
    let accepted = invitation(realm_id, "accepted@example.com", InvitationStatus::Accepted);
    let revoked = invitation(realm_id, "revoked@example.com", InvitationStatus::Revoked);

    for invite in [&pending, &accepted, &revoked] {
        repo.create(invite, None).await?;
    }

    let page = repo
        .list(
            &realm_id,
            &PageRequest {
                page: 1,
                per_page: 10,
                sort_by: Some("email".to_string()),
                sort_dir: Some(SortDirection::Asc),
                q: None,
            },
            &[InvitationStatus::Pending, InvitationStatus::Revoked],
        )
        .await?;

    assert_eq!(page.meta.total, 2);
    let emails = page
        .data
        .iter()
        .map(|invitation| invitation.email.as_str())
        .collect::<Vec<_>>();
    assert_eq!(emails, vec!["pending@example.com", "revoked@example.com"]);

    Ok(())
}
