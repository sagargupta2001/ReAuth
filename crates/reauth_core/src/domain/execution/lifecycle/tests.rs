use super::{LifecycleNode, NodeOutcome};
use crate::domain::auth_session::AuthenticationSession;
use crate::error::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

struct TestNode;

#[async_trait]
impl LifecycleNode for TestNode {
    async fn execute(&self, _session: &mut AuthenticationSession) -> Result<NodeOutcome> {
        Ok(NodeOutcome::SuspendForAsync {
            action_type: "email_verify".to_string(),
            token: "test-token".to_string(),
            expires_at: Utc::now(),
            resume_node_id: None,
            payload: serde_json::json!({}),
            screen: "awaiting-action".to_string(),
            context: serde_json::json!({}),
        })
    }
}

#[tokio::test]
async fn lifecycle_defaults_return_expected_values() {
    let mut session =
        AuthenticationSession::new(Uuid::new_v4(), Uuid::new_v4(), "start".to_string());
    let node = TestNode;

    node.on_enter(&mut session).await.expect("on_enter ok");

    let outcome = node
        .handle_input(&mut session, json!({"value": 1}))
        .await
        .expect("handle_input ok");

    match outcome {
        NodeOutcome::Reject { error } => {
            assert!(error.contains("does not accept input"));
        }
        other => panic!("expected reject outcome, got {other:?}"),
    }

    node.on_exit(&mut session).await.expect("on_exit ok");

    let execute_outcome = node.execute(&mut session).await.expect("execute ok");
    assert!(matches!(
        execute_outcome,
        NodeOutcome::SuspendForAsync { .. }
    ));
}
