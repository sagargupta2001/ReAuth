pub mod cookie_authenticator;
pub mod password_authenticator;

use crate::adapters::auth::cookie_authenticator::CookieAuthenticator;
use crate::adapters::auth::password_authenticator::PasswordAuthenticator;
use crate::application::runtime_registry::RuntimeRegistry;
use crate::domain::execution::StepType;
use crate::ports::login_attempt_repository::LoginAttemptRepository;
use crate::ports::realm_repository::RealmRepository;
use crate::ports::session_repository::SessionRepository;
use crate::ports::user_repository::UserRepository;
use std::sync::Arc;

pub fn register_builtins(
    registry: &mut RuntimeRegistry,
    user_repo: Arc<dyn UserRepository>,
    realm_repo: Arc<dyn RealmRepository>,
    login_attempt_repo: Arc<dyn LoginAttemptRepository>,
    lockout_threshold: i64,
    lockout_duration_secs: i64,
    session_repo: Arc<dyn SessionRepository>,
) {
    // 1. Password Node (Worker)
    // Connects "core.auth.password" string -> PasswordAuthenticator Struct
    let pw_node = Arc::new(PasswordAuthenticator::new(
        user_repo,
        realm_repo,
        login_attempt_repo,
        lockout_threshold,
        lockout_duration_secs,
    ));
    registry.register_node("core.auth.password", pw_node, StepType::Authenticator);

    // 2. Cookie Authenticator (SSO)
    let cookie_node = Arc::new(CookieAuthenticator::new(session_repo));
    registry.register_node("core.auth.cookie", cookie_node, StepType::Authenticator);

    // 3. Terminal Nodes (Definitions only)
    // These nodes use the "Generic Handler" in the Executor loop above.
    registry.register_definition("core.terminal.allow", StepType::Terminal);
    registry.register_definition("core.terminal.deny", StepType::Terminal);

    // 4. Start Node
    registry.register_definition("core.start", StepType::Logic);
}
