use crate::application::runtime_registry::RuntimeRegistry;
use crate::domain::flow::models::NodeMetadata;
use crate::domain::flow::nodes::condition_node::ConditionNode;
use crate::domain::flow::nodes::cookie_node::CookieNodeProvider;
use crate::domain::flow::nodes::otp_node::OtpNode;
use crate::domain::flow::nodes::password_node::PasswordNodeProvider;
use crate::domain::flow::nodes::registration_node::RegistrationNodeProvider;
use crate::domain::flow::nodes::script_node::ScriptNode;
use crate::domain::flow::nodes::start_node::StartNode;
use crate::domain::flow::nodes::terminal_node::{AllowNode, DenyNode};
use crate::domain::flow::provider::NodeProvider;
use std::sync::Arc;

pub struct NodeRegistryService {
    providers: Vec<Box<dyn NodeProvider>>,
    runtime_registry: Arc<RuntimeRegistry>,
}

impl NodeRegistryService {
    pub fn new(runtime_registry: Arc<RuntimeRegistry>) -> Self {
        Self {
            providers: vec![
                Box::new(StartNode),
                Box::new(CookieNodeProvider),
                Box::new(PasswordNodeProvider),
                Box::new(RegistrationNodeProvider),
                Box::new(OtpNode),
                Box::new(ConditionNode),
                Box::new(ScriptNode),
                Box::new(AllowNode),
                Box::new(DenyNode),
            ],
            runtime_registry,
        }
    }

    pub fn get_available_nodes(&self) -> Vec<NodeMetadata> {
        self.providers
            .iter()
            .filter(|p| self.runtime_registry.get_definition(p.id()).is_some())
            .map(|p| NodeMetadata {
                id: p.id().to_string(),
                category: p.category().to_string(),
                display_name: p.display_name().to_string(),
                description: p.description().to_string(),
                icon: p.icon().to_string(),
                inputs: p.inputs().iter().map(|s| s.to_string()).collect(),
                outputs: p.outputs().iter().map(|s| s.to_string()).collect(),
                config_schema: p.config_schema(),
                supports_ui: p.supports_ui(),
                default_template_key: p.default_template_key().map(|value| value.to_string()),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::NodeRegistryService;
    use crate::application::runtime_registry::RuntimeRegistry;
    use crate::domain::auth_session::AuthenticationSession;
    use crate::domain::execution::lifecycle::{LifecycleNode, NodeOutcome};
    use crate::domain::execution::StepType;
    use crate::error::Result;
    use async_trait::async_trait;
    use serde_json::Value;
    use std::sync::Arc;

    #[derive(Default)]
    struct StubNode;

    #[async_trait]
    impl LifecycleNode for StubNode {
        async fn execute(&self, _session: &mut AuthenticationSession) -> Result<NodeOutcome> {
            Ok(NodeOutcome::Continue {
                output: "success".to_string(),
            })
        }

        async fn handle_input(
            &self,
            _session: &mut AuthenticationSession,
            _input: Value,
        ) -> Result<NodeOutcome> {
            Ok(NodeOutcome::Continue {
                output: "success".to_string(),
            })
        }
    }

    #[test]
    fn node_registry_only_returns_runtime_registered_nodes() {
        let mut runtime = RuntimeRegistry::new();
        runtime.register_definition("core.start", StepType::Logic);
        runtime.register_node(
            "core.auth.password",
            Arc::new(StubNode),
            StepType::Authenticator,
        );
        runtime.register_node(
            "core.auth.cookie",
            Arc::new(StubNode),
            StepType::Authenticator,
        );
        runtime.register_definition("core.terminal.allow", StepType::Terminal);
        runtime.register_definition("core.terminal.deny", StepType::Terminal);

        let registry = NodeRegistryService::new(Arc::new(runtime));
        let nodes = registry.get_available_nodes();
        let ids: Vec<String> = nodes.into_iter().map(|node| node.id).collect();

        assert!(ids.iter().any(|id| id == "core.auth.password"));
        assert!(ids.iter().any(|id| id == "core.auth.cookie"));
        assert!(!ids.iter().any(|id| id == "core.auth.otp"));
        assert!(!ids.iter().any(|id| id == "core.logic.condition"));
        assert!(!ids.iter().any(|id| id == "core.logic.script"));
    }
}
