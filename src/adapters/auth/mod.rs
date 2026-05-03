pub mod cookie_authenticator;
pub mod email_otp_issue_node;
pub mod forgot_credentials_authenticator;
pub mod invitation_issue_node;
pub mod invitation_token_node;
pub mod oidc_consent_authenticator;
pub mod passkey_assert_authenticator;
pub mod passkey_enroll_authenticator;
pub mod password_authenticator;
pub mod recovery_issue_node;
pub mod registration_authenticator;
pub mod reset_password_authenticator;
pub mod subflow_node;
pub mod verify_email_otp_authenticator;

use crate::adapters::auth::cookie_authenticator::CookieAuthenticator;
use crate::adapters::auth::email_otp_issue_node::EmailOtpIssueNode;
use crate::adapters::auth::forgot_credentials_authenticator::ForgotCredentialsAuthenticator;
use crate::adapters::auth::invitation_issue_node::InvitationIssueNode;
use crate::adapters::auth::invitation_token_node::InvitationTokenNode;
use crate::adapters::auth::oidc_consent_authenticator::OidcConsentAuthenticator;
use crate::adapters::auth::passkey_assert_authenticator::PasskeyAssertAuthenticator;
use crate::adapters::auth::passkey_enroll_authenticator::PasskeyEnrollAuthenticator;
use crate::adapters::auth::password_authenticator::PasswordAuthenticator;
use crate::adapters::auth::recovery_issue_node::RecoveryIssueNode;
use crate::adapters::auth::registration_authenticator::RegistrationAuthenticator;
use crate::adapters::auth::reset_password_authenticator::ResetPasswordAuthenticator;
use crate::adapters::auth::subflow_node::SubflowNode;
use crate::adapters::auth::verify_email_otp_authenticator::VerifyEmailOtpAuthenticator;
use crate::application::audit_service::AuditService;
use crate::application::rbac_service::RbacService;
use crate::application::runtime_registry::RuntimeRegistry;
use crate::application::user_service::UserService;
use crate::domain::execution::StepType;
use crate::ports::auth_session_action_repository::AuthSessionActionRepository;
use crate::ports::flow_store::FlowStore;
use crate::ports::login_attempt_repository::LoginAttemptRepository;
use crate::ports::realm_passkey_settings_repository::RealmPasskeySettingsRepository;
use crate::ports::realm_recovery_settings_repository::RealmRecoverySettingsRepository;
use crate::ports::realm_repository::RealmRepository;
use crate::ports::recovery_attempt_repository::RecoveryAttemptRepository;
use crate::ports::session_repository::SessionRepository;
use crate::ports::user_repository::UserRepository;
use std::sync::Arc;

pub struct BuiltinAuthContext {
    pub user_service: Arc<UserService>,
    pub user_repo: Arc<dyn UserRepository>,
    pub realm_repo: Arc<dyn RealmRepository>,
    pub rbac_service: Arc<RbacService>,
    pub login_attempt_repo: Arc<dyn LoginAttemptRepository>,
    pub lockout_threshold: i64,
    pub lockout_duration_secs: i64,
    pub session_repo: Arc<dyn SessionRepository>,
    pub flow_store: Arc<dyn FlowStore>,
    pub action_repo: Arc<dyn AuthSessionActionRepository>,
    pub recovery_attempt_repo: Arc<dyn RecoveryAttemptRepository>,
    pub audit_service: Arc<AuditService>,
    pub recovery_settings_repo: Arc<dyn RealmRecoverySettingsRepository>,
    pub passkey_settings_repo: Arc<dyn RealmPasskeySettingsRepository>,
}

pub fn register_builtins(registry: &mut RuntimeRegistry, ctx: BuiltinAuthContext) {
    // 1. Password Node (Worker)
    // Connects "core.auth.password" string -> PasswordAuthenticator Struct
    let pw_node = Arc::new(PasswordAuthenticator::new(
        ctx.user_repo,
        ctx.realm_repo.clone(),
        ctx.login_attempt_repo,
        ctx.lockout_threshold,
        ctx.lockout_duration_secs,
    ));
    registry.register_node("core.auth.password", pw_node, StepType::Authenticator);

    // 1.1 Passkey Assert Node
    let passkey_assert_node = Arc::new(PasskeyAssertAuthenticator::new(
        ctx.passkey_settings_repo.clone(),
    ));
    registry.register_node(
        "core.auth.passkey_assert",
        passkey_assert_node,
        StepType::Authenticator,
    );

    let passkey_enroll_node = Arc::new(PasskeyEnrollAuthenticator::new(
        ctx.passkey_settings_repo.clone(),
    ));
    registry.register_node(
        "core.auth.passkey_enroll",
        passkey_enroll_node,
        StepType::Authenticator,
    );

    // 2. Registration Node
    let registration_node = Arc::new(RegistrationAuthenticator::new(
        ctx.user_service.clone(),
        ctx.realm_repo.clone(),
        ctx.rbac_service,
    ));
    registry.register_node(
        "core.auth.register",
        registration_node,
        StepType::Authenticator,
    );

    // 3. Forgot Credentials Node
    let forgot_node = Arc::new(ForgotCredentialsAuthenticator::new(
        ctx.recovery_attempt_repo,
        ctx.recovery_settings_repo.clone(),
    ));
    registry.register_node(
        "core.auth.forgot_credentials",
        forgot_node,
        StepType::Authenticator,
    );

    // 4. Reset Password Node
    let reset_node = Arc::new(ResetPasswordAuthenticator::new(
        ctx.user_service.clone(),
        ctx.session_repo.clone(),
        ctx.audit_service.clone(),
        ctx.recovery_settings_repo.clone(),
        ctx.action_repo.clone(),
    ));
    registry.register_node(
        "core.auth.reset_password",
        reset_node,
        StepType::Authenticator,
    );

    // 5. OIDC Consent Node
    let consent_node = Arc::new(OidcConsentAuthenticator::new());
    registry.register_node("core.oidc.consent", consent_node, StepType::Authenticator);

    // 6. Recovery Issue Logic Node
    let recovery_issue_node = Arc::new(RecoveryIssueNode::new(
        ctx.user_service.clone(),
        ctx.recovery_settings_repo.clone(),
    ));
    registry.register_node(
        "core.logic.recovery_issue",
        recovery_issue_node,
        StepType::Logic,
    );

    // 7. Email OTP Issue Logic Node
    let email_otp_issue_node = Arc::new(EmailOtpIssueNode);
    registry.register_node(
        "core.logic.issue_email_otp",
        email_otp_issue_node,
        StepType::Logic,
    );

    // 8. Verify Email OTP Authenticator
    let verify_email_otp_node = Arc::new(VerifyEmailOtpAuthenticator);
    registry.register_node(
        "core.auth.verify_email_otp",
        verify_email_otp_node,
        StepType::Authenticator,
    );

    // 9. Cookie Authenticator (SSO)
    let cookie_node = Arc::new(CookieAuthenticator::new(ctx.session_repo));
    registry.register_node("core.auth.cookie", cookie_node, StepType::Authenticator);

    // 10. Subflow Node
    let subflow_node = Arc::new(SubflowNode::new(ctx.flow_store.clone()));
    registry.register_node("core.logic.subflow", subflow_node, StepType::Logic);

    // 11. Terminal Nodes (Definitions only)
    // These nodes use the "Generic Handler" in the Executor loop above.
    let invitation_token_node = Arc::new(InvitationTokenNode);
    registry.register_node(
        "core.logic.invitation_token",
        invitation_token_node,
        StepType::Logic,
    );
    let invitation_issue_node = Arc::new(InvitationIssueNode);
    registry.register_node(
        "core.logic.issue_invitation",
        invitation_issue_node,
        StepType::Logic,
    );

    // 12. Terminal Nodes (Definitions only)
    // These nodes use the "Generic Handler" in the Executor loop above.
    registry.register_definition("core.terminal.allow", StepType::Terminal);
    registry.register_definition("core.terminal.deny", StepType::Terminal);

    // 12. Start Node
    registry.register_definition("core.start", StepType::Logic);
    // 13. Condition Logic Node
    registry.register_definition("core.logic.condition", StepType::Logic);
}
