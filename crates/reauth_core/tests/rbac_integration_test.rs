// -----------------------------------------------------------------------------
// 1. COMMON TEST SETUP MODULE
// -----------------------------------------------------------------------------
mod common {
    use reauth_core::{
        adapters::{
            cache::{cache_invalidator::CacheInvalidator, moka_cache::MokaCacheService},
            eventing::in_memory_bus::InMemoryEventBus,
            persistence::{
                connection::{init_db, Database},
                migrate::run_migrations,
                sqlite_rbac_repository::SqliteRbacRepository,
                sqlite_user_repository::SqliteUserRepository,
            },
        },
        application::{rbac_service::RbacService, user_service::UserService},
        config::DatabaseConfig,
        ports::event_bus::EventSubscriber,
    };
    use std::sync::Arc;

    /// A container for all the initialized services needed for a test.
    pub struct TestContext {
        pub rbac_service: Arc<RbacService>,
        pub user_service: Arc<UserService>,
        pub cache_service: Arc<MokaCacheService>,
        pub rbac_repo: Arc<SqliteRbacRepository>,
    }

    /// This is the "Arrange" step for all our tests.
    /// It creates a fresh, isolated, in-memory database and wires up
    /// the entire application stack (repos, services, event bus, cache, invalidator).
    pub async fn setup_test_env() -> TestContext {
        // 1. Create an in-memory SQLite database
        let db_config = DatabaseConfig {
            url: "sqlite::memory:".to_string(), // In-memory DB
            max_connections: 1,
            data_dir: "../../data".to_string(),
        };
        let db_pool = init_db(&db_config)
            .await
            .expect("Failed to init in-memory db");

        // 2. Run all real migrations
        run_migrations(&db_pool)
            .await
            .expect("Failed to run migrations");

        // 3. Initialize Adapters
        let user_repo = Arc::new(SqliteUserRepository::new(db_pool.clone()));
        let rbac_repo = Arc::new(SqliteRbacRepository::new(db_pool.clone()));
        let cache_service = Arc::new(MokaCacheService::new());
        let event_bus = Arc::new(InMemoryEventBus::new());

        // 4. Initialize Application Services
        let user_service = Arc::new(UserService::new(user_repo, event_bus.clone()));
        let rbac_service = Arc::new(RbacService::new(
            rbac_repo.clone(),
            cache_service.clone(),
            event_bus.clone(),
        ));

        // 5. Initialize and Subscribe Listeners
        let cache_invalidator = Arc::new(CacheInvalidator::new(
            cache_service.clone(),
            rbac_repo.clone(),
        ));
        event_bus.subscribe(cache_invalidator).await;

        TestContext {
            rbac_service,
            user_service,
            cache_service,
            rbac_repo,
        }
    }
}

// -----------------------------------------------------------------------------
// 2. REFACTORED CRUD TESTS
// -----------------------------------------------------------------------------
use common::setup_test_env;
use reauth_core::application::rbac_service::{CreateGroupPayload, CreateRolePayload};
use reauth_core::constants::DEFAULT_REALM_NAME;
use reauth_core::error::Error;
use reauth_core::ports::cache_service::CacheService;

// Each test is small, focused, and independent.
#[tokio::test]
async fn test_create_role_success() {
    let ctx = setup_test_env().await;
    let payload = CreateRolePayload {
        name: "admin".to_string(),
        description: None,
    };

    let result = ctx.rbac_service.create_role(payload).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().name, "admin");
}

#[tokio::test]
async fn test_create_role_duplicate_fails() {
    let ctx = setup_test_env().await;
    let payload = CreateRolePayload {
        name: "admin".to_string(),
        description: None,
    };

    ctx.rbac_service.create_role(payload.clone()).await.unwrap();
    let result = ctx.rbac_service.create_role(payload.clone()).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::RoleAlreadyExists));
}

#[tokio::test]
async fn test_create_group_success() {
    let ctx = setup_test_env().await;
    let payload = CreateGroupPayload {
        name: "developers".to_string(),
        description: None,
    };

    let result = ctx.rbac_service.create_group(payload).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().name, "developers");
}

#[tokio::test]
async fn test_create_group_duplicate_fails() {
    let ctx = setup_test_env().await;
    let payload = CreateGroupPayload {
        name: "developers".to_string(),
        description: None,
    };

    ctx.rbac_service
        .create_group(payload.clone())
        .await
        .unwrap();
    let result = ctx.rbac_service.create_group(payload.clone()).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::GroupAlreadyExists));
}

// -----------------------------------------------------------------------------
// 3. NEW EVENT-DRIVEN CACHE INVALIDATION TESTS
// -----------------------------------------------------------------------------

const TEST_PERMISSION: &str = "tickets:read";

// #[tokio::test]
// async fn test_cache_is_invalidated_when_user_assigned_to_group() -> anyhow::Result<()> {
//     // 1. ARRANGE
//     let ctx = setup_test_env().await;
//     let realm = ctx.realm
//     let user = ctx
//         .user_service
//         .create_user(DEFAULT_REALM_NAME, "test@user.com", "pass")
//         .await?;
//     let role = ctx
//         .rbac_service
//         .create_role(CreateRolePayload {
//             name: "support".to_string(),
//             ..Default::default()
//         })
//         .await?;
//     let group = ctx
//         .rbac_service
//         .create_group(CreateGroupPayload {
//             name: "support_team".to_string(),
//             ..Default::default()
//         })
//         .await?;
//
//     // Link permission -> role -> group
//     ctx.rbac_service
//         .assign_permission_to_role(TEST_PERMISSION.to_string(), role.id)
//         .await?;
//     ctx.rbac_service
//         .assign_role_to_group(role.id, group.id)
//         .await?;
//
//     // 2. ACT (Prime the cache)
//     // The user has no permissions yet.
//     let perms_before = ctx.rbac_service.get_effective_permissions(&user.id).await?;
//     assert!(
//         perms_before.is_empty(),
//         "Permissions should be empty initially"
//     );
//
//     // Prove it's in the cache by checking the adapter directly
//     let cached = ctx.cache_service.get_user_permissions(&user.id).await;
//     assert!(
//         cached.is_some(),
//         "Cache should be populated (with empty set)"
//     );
//
//     // 3. ACT (Trigger the event)
//     ctx.rbac_service
//         .assign_user_to_group(user.id, group.id)
//         .await?;
//
//     // Give the event bus a moment to fire (important for `tokio::spawn` tasks)
//     tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
//
//     // 4. ASSERT
//     // Prove the cache was invalidated by the event
//     let cached_after = ctx.cache_service.get_user_permissions(&user.id).await;
//     assert!(
//         cached_after.is_none(),
//         "Cache should have been invalidated by the event"
//     );
//
//     // Prove that the next call fetches the *new* permissions
//     let perms_after = ctx.rbac_service.get_effective_permissions(&user.id).await?;
//     assert!(!perms_after.is_empty(), "New permissions should be fetched");
//     assert!(perms_after.contains(&TEST_PERMISSION.to_string()));
//
//     Ok(())
// }
//
// #[tokio::test]
// async fn test_cache_is_invalidated_when_role_assigned_to_group() -> anyhow::Result<()> {
//     // 1. ARRANGE
//     let ctx = setup_test_env().await;
//     let user = ctx
//         .user_service
//         .create_user("test@user.com", "pass")
//         .await?;
//     let role = ctx
//         .rbac_service
//         .create_role(CreateRolePayload {
//             name: "support".to_string(),
//             ..Default::default()
//         })
//         .await?;
//     let group = ctx
//         .rbac_service
//         .create_group(CreateGroupPayload {
//             name: "support_team".to_string(),
//             ..Default::default()
//         })
//         .await?;
//
//     ctx.rbac_service
//         .assign_permission_to_role(TEST_PERMISSION.to_string(), role.id)
//         .await?;
//     // This time, assign the user to the group *before* priming the cache
//     ctx.rbac_service
//         .assign_user_to_group(user.id, group.id)
//         .await?;
//
//     // 2. ACT (Prime the cache)
//     // The user is in the group, but the group has no roles yet.
//     let perms_before = ctx.rbac_service.get_effective_permissions(&user.id).await?;
//     assert!(
//         perms_before.is_empty(),
//         "Permissions should be empty initially"
//     );
//
//     // Prove it's in the cache
//     let cached = ctx.cache_service.get_user_permissions(&user.id).await;
//     assert!(
//         cached.is_some(),
//         "Cache should be populated (with empty set)"
//     );
//
//     // 3. ACT (Trigger the event)
//     // Now, assign the role to the group. This event should invalidate
//     // the cache for all users in that group (i.e., our test user).
//     ctx.rbac_service
//         .assign_role_to_group(role.id, group.id)
//         .await?;
//
//     tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
//
//     // 4. ASSERT
//     // Prove the cache was invalidated by the event
//     let cached_after = ctx.cache_service.get_user_permissions(&user.id).await;
//     assert!(cached_after.is_none(), "Cache should have been invalidated");
//
//     // Prove that the next call fetches the *new* permissions
//     let perms_after = ctx.rbac_service.get_effective_permissions(&user.id).await?;
//     assert!(!perms_after.is_empty(), "New permissions should be fetched");
//     assert!(perms_after.contains(&TEST_PERMISSION.to_string()));
//
//     Ok(())
// }
