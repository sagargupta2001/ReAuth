pub mod cookie_authenticator;
pub mod password_authenticator;
pub mod registration_authenticator;

use crate::adapters::auth::cookie_authenticator::CookieAuthenticator;
use crate::adapters::auth::password_authenticator::PasswordAuthenticator;
use crate::adapters::auth::registration_authenticator::RegistrationAuthenticator;
use crate::application::runtime_registry::RuntimeRegistry;
use crate::application::user_service::UserService;
use crate::domain::execution::StepType;
use crate::ports::login_attempt_repository::LoginAttemptRepository;
use crate::ports::realm_repository::RealmRepository;
use crate::ports::session_repository::SessionRepository;
use crate::ports::user_repository::UserRepository;
use std::sync::Arc;

pub struct BuiltinAuthContext {
    pub user_service: Arc<UserService>,
    pub user_repo: Arc<dyn UserRepository>,
    pub realm_repo: Arc<dyn RealmRepository>,
    pub login_attempt_repo: Arc<dyn LoginAttemptRepository>,
    pub lockout_threshold: i64,
    pub lockout_duration_secs: i64,
    pub session_repo: Arc<dyn SessionRepository>,
}

pub fn register_builtins(registry: &mut RuntimeRegistry, ctx: BuiltinAuthContext) {
    // 1. Password Node (Worker)
    // Connects "core.auth.password" string -> PasswordAuthenticator Struct
    let pw_node = Arc::new(PasswordAuthenticator::new(
        ctx.user_repo,
        ctx.realm_repo,
        ctx.login_attempt_repo,
        ctx.lockout_threshold,
        ctx.lockout_duration_secs,
    ));
    registry.register_node("core.auth.password", pw_node, StepType::Authenticator);

    // 2. Registration Node
    let registration_node = Arc::new(RegistrationAuthenticator::new(ctx.user_service));
    registry.register_node(
        "core.auth.register",
        registration_node,
        StepType::Authenticator,
    );

    // 3. Cookie Authenticator (SSO)
    let cookie_node = Arc::new(CookieAuthenticator::new(ctx.session_repo));
    registry.register_node("core.auth.cookie", cookie_node, StepType::Authenticator);

    // 4. Terminal Nodes (Definitions only)
    // These nodes use the "Generic Handler" in the Executor loop above.
    registry.register_definition("core.terminal.allow", StepType::Terminal);
    registry.register_definition("core.terminal.deny", StepType::Terminal);

    // 5. Start Node
    registry.register_definition("core.start", StepType::Logic);
}
