use super::condition_node::ConditionNodeProvider;
use super::cookie_node::CookieNodeProvider;
use super::email_otp_issue_node::EmailOtpIssueNodeProvider;
use super::forgot_credentials_node::ForgotCredentialsNodeProvider;
use super::oidc_consent_node::OidcConsentNodeProvider;
use super::passkey_assert_node::PasskeyAssertNodeProvider;
use super::passkey_enroll_node::PasskeyEnrollNodeProvider;
use super::password_node::PasswordNodeProvider;
use super::recovery_issue_node::RecoveryIssueNodeProvider;
use super::registration_node::RegistrationNodeProvider;
use super::reset_password_node::ResetPasswordNodeProvider;
use super::start_node::StartNode;
use super::subflow_node::SubflowNodeProvider;
use super::terminal_node::{AllowNode, DenyNode};
use super::verify_email_otp_node::VerifyEmailOtpNodeProvider;
use crate::domain::flow::provider::NodeProvider;

#[test]
fn start_node_metadata_is_consistent() {
    let node = StartNode;

    assert_eq!(node.id(), "core.start");
    assert_eq!(node.display_name(), "Start Flow");
    assert_eq!(
        node.description(),
        "The entry point of the authentication flow."
    );
    assert_eq!(node.icon(), "Play");
    assert_eq!(node.category(), "Start");
    assert!(node.inputs().is_empty());
    assert_eq!(node.outputs(), vec!["next"]);
    assert!(node.config_schema().as_object().unwrap().is_empty());
}

#[test]
fn cookie_node_metadata_is_consistent() {
    let node = CookieNodeProvider;

    assert_eq!(node.id(), "core.auth.cookie");
    assert_eq!(node.display_name(), "Cookie / SSO");
    assert!(node.description().contains("valid SSO session cookie"));
    assert_eq!(node.icon(), "cookie");
    assert_eq!(node.category(), "Authenticator");
    assert_eq!(node.inputs(), vec!["default"]);
    assert_eq!(node.outputs(), vec!["continue"]);
    let schema = node.config_schema();
    assert_eq!(
        schema.get("additionalProperties").and_then(|v| v.as_bool()),
        Some(false)
    );
}

#[test]
fn password_node_metadata_is_consistent() {
    let node = PasswordNodeProvider;

    assert_eq!(node.id(), "core.auth.password");
    assert_eq!(node.display_name(), "Username & Password");
    assert!(node.description().contains("Standard login form"));
    assert_eq!(node.icon(), "Lock");
    assert_eq!(node.category(), "Authenticator");
    assert_eq!(node.outputs(), vec!["success", "force_reset", "failure"]);
    assert!(node.config_schema().get("properties").is_some());
}

#[test]
fn passkey_assert_node_metadata_is_consistent() {
    let node = PasskeyAssertNodeProvider;

    assert_eq!(node.id(), "core.auth.passkey_assert");
    assert_eq!(node.display_name(), "Passkey Assert");
    assert!(node.description().contains("WebAuthn passkey"));
    assert_eq!(node.icon(), "Fingerprint");
    assert_eq!(node.category(), "Authenticator");
    assert_eq!(node.outputs(), vec!["success", "fallback", "failure"]);
    assert_eq!(node.default_template_key(), Some("passkey_assert"));
    assert!(node.supports_ui());
}

#[test]
fn passkey_enroll_node_metadata_is_consistent() {
    let node = PasskeyEnrollNodeProvider;

    assert_eq!(node.id(), "core.auth.passkey_enroll");
    assert_eq!(node.display_name(), "Passkey Enroll");
    assert!(node.description().contains("WebAuthn passkey"));
    assert_eq!(node.icon(), "KeyRound");
    assert_eq!(node.category(), "Authenticator");
    assert_eq!(node.outputs(), vec!["success", "skip", "failure"]);
    assert_eq!(node.default_template_key(), Some("passkey_enroll"));
    assert!(node.supports_ui());
}

#[test]
fn registration_node_metadata_is_consistent() {
    let node = RegistrationNodeProvider;

    assert_eq!(node.id(), "core.auth.register");
    assert_eq!(node.display_name(), "Register Account");
    assert!(node.description().contains("new user account"));
    assert_eq!(node.icon(), "UserPlus");
    assert_eq!(node.category(), "Authenticator");
    assert_eq!(node.outputs(), vec!["success", "failure"]);
    assert_eq!(node.default_template_key(), Some("register"));
    assert!(node.supports_ui());
}

#[test]
fn forgot_credentials_node_metadata_is_consistent() {
    let node = ForgotCredentialsNodeProvider;

    assert_eq!(node.id(), "core.auth.forgot_credentials");
    assert_eq!(node.display_name(), "Forgot Credentials");
    assert!(node.description().contains("credential recovery"));
    assert_eq!(node.icon(), "Mail");
    assert_eq!(node.category(), "Authenticator");
    assert_eq!(node.outputs(), vec!["success", "failure"]);
    assert_eq!(node.default_template_key(), Some("forgot_credentials"));
    assert!(node.supports_ui());
}

#[test]
fn reset_password_node_metadata_is_consistent() {
    let node = ResetPasswordNodeProvider;

    assert_eq!(node.id(), "core.auth.reset_password");
    assert_eq!(node.display_name(), "Reset Password");
    assert!(node.description().contains("new password"));
    assert_eq!(node.icon(), "Key");
    assert_eq!(node.category(), "Authenticator");
    assert_eq!(node.outputs(), vec!["success", "failure"]);
    assert_eq!(node.default_template_key(), Some("reset_password"));
    assert!(node.supports_ui());
}

#[test]
fn terminal_nodes_have_no_outputs() {
    let allow = AllowNode;
    let deny = DenyNode;

    assert_eq!(allow.id(), "core.terminal.allow");
    assert_eq!(allow.display_name(), "Allow Access");
    assert!(allow.description().contains("issue tokens"));
    assert_eq!(allow.icon(), "CheckCircle");
    assert_eq!(allow.category(), "Terminal");
    assert!(allow.outputs().is_empty());
    assert!(allow.config_schema().get("properties").is_some());
    assert_eq!(deny.id(), "core.terminal.deny");
    assert_eq!(deny.display_name(), "Deny Access");
    assert!(deny.description().contains("Reject"));
    assert_eq!(deny.icon(), "XCircle");
    assert_eq!(deny.category(), "Terminal");
    assert!(deny.outputs().is_empty());
    assert!(deny.config_schema().get("properties").is_some());
}

#[test]
fn oidc_consent_node_metadata_is_consistent() {
    let node = OidcConsentNodeProvider;

    assert_eq!(node.id(), "core.oidc.consent");
    assert_eq!(node.display_name(), "OIDC Consent");
    assert!(node.description().contains("OIDC scopes"));
    assert_eq!(node.icon(), "ShieldAlert");
    assert_eq!(node.category(), "Authenticator");
    assert_eq!(node.outputs(), vec!["allow", "deny"]);
    assert_eq!(node.default_template_key(), Some("consent"));
    assert!(node.supports_ui());
}

#[test]
fn recovery_issue_node_metadata_is_consistent() {
    let node = RecoveryIssueNodeProvider;

    assert_eq!(node.id(), "core.logic.recovery_issue");
    assert_eq!(node.display_name(), "Issue Recovery Token");
    assert!(node.description().contains("recovery token"));
    assert_eq!(node.icon(), "ShieldAlert");
    assert_eq!(node.category(), "Logic");
    assert_eq!(node.inputs(), vec!["default"]);
    assert_eq!(node.outputs(), vec!["issued"]);
    assert!(node.config_schema().get("properties").is_some());
    assert_eq!(node.default_template_key(), Some("awaiting_action"));
    assert!(node.supports_ui());
}

#[test]
fn email_otp_issue_node_metadata_is_consistent() {
    let node = EmailOtpIssueNodeProvider;

    assert_eq!(node.id(), "core.logic.issue_email_otp");
    assert_eq!(node.display_name(), "Issue Email OTP");
    assert!(node.description().contains("verification"));
    assert_eq!(node.icon(), "Mail");
    assert_eq!(node.category(), "Logic");
    assert_eq!(node.inputs(), vec!["default"]);
    assert_eq!(node.outputs(), vec!["issued"]);
    assert!(node.config_schema().get("properties").is_some());
}

#[test]
fn verify_email_otp_node_metadata_is_consistent() {
    let node = VerifyEmailOtpNodeProvider;

    assert_eq!(node.id(), "core.auth.verify_email_otp");
    assert_eq!(node.display_name(), "Verify Email OTP");
    assert!(node.description().contains("verification"));
    assert_eq!(node.icon(), "CheckCircle");
    assert_eq!(node.category(), "Authenticator");
    assert_eq!(node.outputs(), vec!["success", "failure"]);
    assert_eq!(node.default_template_key(), Some("verify_email"));
    assert!(node.supports_ui());
}

#[test]
fn condition_node_metadata_is_consistent() {
    let node = ConditionNodeProvider;

    assert_eq!(node.id(), "core.logic.condition");
    assert_eq!(node.display_name(), "Condition");
    assert!(node.description().contains("session context"));
    assert_eq!(node.icon(), "Split");
    assert_eq!(node.category(), "Logic");
    assert_eq!(node.inputs(), vec!["default"]);
    assert_eq!(node.outputs(), vec!["true", "false"]);
    assert!(node.config_schema().get("properties").is_some());
}

#[test]
fn subflow_node_metadata_is_consistent() {
    let node = SubflowNodeProvider;

    assert_eq!(node.id(), "core.logic.subflow");
    assert_eq!(node.display_name(), "Call Subflow");
    assert!(node.description().contains("deployed child flow"));
    assert_eq!(node.icon(), "Workflow");
    assert_eq!(node.category(), "Logic");
    assert_eq!(node.inputs(), vec!["default"]);
    assert_eq!(node.outputs(), vec!["success", "failure"]);
    assert!(node.config_schema().get("properties").is_some());
}
