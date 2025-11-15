use crate::adapters::persistence::connection::Database;
use crate::adapters::persistence::sqlite_rbac_repository::SqliteRbacRepository;
use crate::adapters::persistence::sqlite_realm_repository::SqliteRealmRepository;
use crate::adapters::persistence::sqlite_session_repository::SqliteSessionRepository;
use crate::adapters::SqliteUserRepository;
use std::sync::Arc;

pub fn initialize_repositories(
    db_pool: &sqlx::SqlitePool,
) -> (
    Arc<SqliteUserRepository>,
    Arc<SqliteRbacRepository>,
    Arc<SqliteRealmRepository>,
    Arc<SqliteSessionRepository>,
) {
    let user_repo = Arc::new(SqliteUserRepository::new(Database::from(db_pool.clone())));
    let rbac_repo = Arc::new(SqliteRbacRepository::new(Database::from(db_pool.clone())));
    let realm_repo = Arc::new(SqliteRealmRepository::new(Database::from(db_pool.clone())));
    let session_repo = Arc::new(SqliteSessionRepository::new(Database::from(
        db_pool.clone(),
    )));

    (user_repo, rbac_repo, realm_repo, session_repo)
}
