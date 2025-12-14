pub mod password_authenticator;

use crate::adapters::auth::password_authenticator::PasswordAuthenticator;
use crate::application::runtime_registry::RuntimeRegistry;
use crate::domain::execution::StepType;
use crate::ports::user_repository::UserRepository;
use std::sync::Arc;

pub fn register_builtins(registry: &mut RuntimeRegistry, user_repo: Arc<dyn UserRepository>) {
    // 1. Password Node (Worker)
    // Connects "core.auth.password" string -> PasswordAuthenticator Struct
    let pw_node = Arc::new(PasswordAuthenticator::new(user_repo));
    registry.register_node("core.auth.password", pw_node, StepType::Authenticator);

    // 2. Terminal Nodes (Definitions only)
    // These nodes use the "Generic Handler" in the Executor loop above.
    registry.register_definition("core.terminal.allow", StepType::Terminal);
    registry.register_definition("core.terminal.deny", StepType::Terminal);

    // 3. Start Node
    registry.register_definition("core.start", StepType::Logic);
}
