use crate::adapters::auth::password_authenticator::PasswordAuthenticator;
use crate::application::runtime_registry::RuntimeRegistry;
use crate::ports::user_repository::UserRepository;
use std::sync::Arc;

pub mod password_authenticator;

// [FIX] Change argument to accept the Repository, not the Service
pub fn register_builtins(registry: &mut RuntimeRegistry, user_repo: Arc<dyn UserRepository>) {
    let pw = Arc::new(PasswordAuthenticator::new(user_repo));
    registry.register_authenticator("core.auth.password", pw);
}
