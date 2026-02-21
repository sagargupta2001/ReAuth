use super::condition_node::ConditionNode;
use super::cookie_node::CookieNodeProvider;
use super::otp_node::OtpNode;
use super::password_node::PasswordNodeProvider;
use super::script_node::ScriptNode;
use super::start_node::StartNode;
use super::terminal_node::{AllowNode, DenyNode};
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
fn condition_node_metadata_is_consistent() {
    let node = ConditionNode;

    assert_eq!(node.id(), "core.logic.condition");
    assert_eq!(node.display_name(), "Condition Check");
    assert_eq!(
        node.description(),
        "Branch flow based on user or session data."
    );
    assert_eq!(node.icon(), "Split");
    assert_eq!(node.category(), "Logic");
    assert_eq!(node.outputs(), vec!["true", "false"]);

    let schema = node.config_schema();
    let required = schema.get("required").and_then(|v| v.as_array());
    assert!(required.is_some());
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
    assert_eq!(node.outputs(), vec!["success", "failure"]);
    assert!(node.config_schema().get("properties").is_some());
}

#[test]
fn otp_node_metadata_is_consistent() {
    let node = OtpNode;

    assert_eq!(node.id(), "core.auth.otp");
    assert_eq!(node.display_name(), "One-Time Password");
    assert!(node.description().contains("verification code"));
    assert_eq!(node.icon(), "Smartphone");
    assert_eq!(node.category(), "Authenticator");
    assert_eq!(node.outputs(), vec!["success", "failure", "resend"]);
    assert!(node.config_schema().get("properties").is_some());
}

#[test]
fn script_node_metadata_is_consistent() {
    let node = ScriptNode;

    assert_eq!(node.id(), "core.logic.script");
    assert_eq!(node.display_name(), "Execution Script");
    assert!(node.description().contains("custom internal logic"));
    assert_eq!(node.icon(), "Code");
    assert_eq!(node.category(), "Logic");
    assert_eq!(node.outputs(), vec!["next", "error"]);
    assert!(node.config_schema().get("properties").is_some());
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
