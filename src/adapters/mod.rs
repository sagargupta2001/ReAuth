pub(crate) mod auth;
pub mod cache;
pub mod crypto;
pub mod eventing;
pub mod logging;
pub mod observability;
pub mod persistence;
pub mod web;

pub use persistence::{
    connection::init_db, migrate::run_migrations, sqlite_user_repository::SqliteUserRepository,
};

pub use web::server::start_server;
