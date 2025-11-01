use reauth_core::{
    adapters::persistence::connection::Database,

};
use std::sync::Arc;
use sqlx::{Executor, SqlitePool};
use reauth_core::adapters::persistence::sqlite_rbac_repository::SqliteRbacRepository;
use reauth_core::application::rbac_service::{CreateGroupPayload, CreateRolePayload, RbacService};
use reauth_core::error::Error;

#[tokio::test]
async fn test_create_role_and_group_flow() -> anyhow::Result<()> {
    // Setup in-memory SQLite DB
    let pool = SqlitePool::connect(":memory:").await?;
    pool.execute(
        r#"
        CREATE TABLE roles (
            id TEXT PRIMARY KEY,
            name TEXT UNIQUE NOT NULL,
            description TEXT
        );
        CREATE TABLE groups (
            id TEXT PRIMARY KEY,
            name TEXT UNIQUE NOT NULL,
            description TEXT
        );
        "#
    ).await?;

    let db = Arc::new(Database::from(pool.clone()));
    let repo = Arc::new(SqliteRbacRepository::new((*db).clone()));
    let service = RbacService::new(repo.clone());

    // Create a new role
    let role_payload = CreateRolePayload {
        name: "admin".to_string(),
        description: Some("Administrator role".to_string()),
    };

    let role = service.create_role(role_payload).await?;
    assert_eq!(role.name, "admin");
    assert!(role.description.as_ref().unwrap().contains("Administrator"));

    // Trying to create a duplicate role should fail
    let duplicate = service.create_role(CreateRolePayload {
        name: "admin".to_string(),
        description: None,
    }).await;
    assert!(matches!(duplicate, Err(Error::RoleAlreadyExists)));

    // Create a group
    let group_payload = CreateGroupPayload {
        name: "dev-team".to_string(),
        description: Some("Developers group".to_string()),
    };

    let group = service.create_group(group_payload).await?;
    assert_eq!(group.name, "dev-team");

    // Duplicate group should fail
    let duplicate_group = service.create_group(CreateGroupPayload {
        name: "dev-team".to_string(),
        description: None,
    }).await;
    assert!(matches!(duplicate_group, Err(Error::GroupAlreadyExists)));

    Ok(())
}
