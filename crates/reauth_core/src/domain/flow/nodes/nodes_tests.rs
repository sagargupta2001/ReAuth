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
    assert!(node.inputs().is_empty());
    assert_eq!(node.outputs(), vec!["next"]);
    assert!(node.config_schema().as_object().unwrap().is_empty());
}

#[test]
fn condition_node_metadata_is_consistent() {
    let node = ConditionNode;

    assert_eq!(node.id(), "core.logic.condition");
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
    assert_eq!(node.category(), "Authenticator");
    assert_eq!(node.inputs(), vec!["default"]);
    assert_eq!(node.outputs(), vec!["continue"]);
}

#[test]
fn password_node_metadata_is_consistent() {
    let node = PasswordNodeProvider;

    assert_eq!(node.id(), "core.auth.password");
    assert_eq!(node.outputs(), vec!["success", "failure"]);
    assert!(node.config_schema().get("properties").is_some());
}

#[test]
fn otp_node_metadata_is_consistent() {
    let node = OtpNode;

    assert_eq!(node.id(), "core.auth.otp");
    assert_eq!(node.outputs(), vec!["success", "failure", "resend"]);
    assert!(node.config_schema().get("properties").is_some());
}

#[test]
fn script_node_metadata_is_consistent() {
    let node = ScriptNode;

    assert_eq!(node.id(), "core.logic.script");
    assert_eq!(node.outputs(), vec!["next", "error"]);
    assert!(node.config_schema().get("properties").is_some());
}

#[test]
fn terminal_nodes_have_no_outputs() {
    let allow = AllowNode;
    let deny = DenyNode;

    assert_eq!(allow.id(), "core.terminal.allow");
    assert!(allow.outputs().is_empty());
    assert_eq!(deny.id(), "core.terminal.deny");
    assert!(deny.outputs().is_empty());
}
