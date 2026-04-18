use crate::application::runtime_registry::RuntimeRegistry;
use crate::domain::flow::models::NodeContract;
use crate::domain::flow::nodes::condition_node::ConditionNodeProvider;
use crate::domain::flow::nodes::cookie_node::CookieNodeProvider;
use crate::domain::flow::nodes::email_otp_issue_node::EmailOtpIssueNodeProvider;
use crate::domain::flow::nodes::forgot_credentials_node::ForgotCredentialsNodeProvider;
use crate::domain::flow::nodes::oidc_consent_node::OidcConsentNodeProvider;
use crate::domain::flow::nodes::password_node::PasswordNodeProvider;
use crate::domain::flow::nodes::recovery_issue_node::RecoveryIssueNodeProvider;
use crate::domain::flow::nodes::registration_node::RegistrationNodeProvider;
use crate::domain::flow::nodes::reset_password_node::ResetPasswordNodeProvider;
use crate::domain::flow::nodes::scripted_logic_node::ScriptedLogicNodeProvider;
use crate::domain::flow::nodes::scripted_ui_node::ScriptedUiNodeProvider;
use crate::domain::flow::nodes::start_node::StartNode;
use crate::domain::flow::nodes::terminal_node::{AllowNode, DenyNode};
use crate::domain::flow::nodes::verify_email_otp_node::VerifyEmailOtpNodeProvider;
use crate::domain::flow::provider::NodeProvider;
use std::sync::Arc;

pub struct NodeRegistryService {
    providers: Vec<Box<dyn NodeProvider>>,
    runtime_registry: Arc<RuntimeRegistry>,
}

impl NodeRegistryService {
    pub fn new(runtime_registry: Arc<RuntimeRegistry>) -> Self {
        Self::with_providers(
            vec![
                Box::new(StartNode),
                Box::new(ConditionNodeProvider),
                Box::new(RecoveryIssueNodeProvider),
                Box::new(EmailOtpIssueNodeProvider),
                Box::new(CookieNodeProvider),
                Box::new(PasswordNodeProvider),
                Box::new(ForgotCredentialsNodeProvider),
                Box::new(OidcConsentNodeProvider),
                Box::new(RegistrationNodeProvider),
                Box::new(ResetPasswordNodeProvider),
                Box::new(VerifyEmailOtpNodeProvider),
                Box::new(ScriptedLogicNodeProvider),
                Box::new(ScriptedUiNodeProvider),
                Box::new(AllowNode),
                Box::new(DenyNode),
            ],
            runtime_registry,
        )
    }

    pub fn with_providers(
        providers: Vec<Box<dyn NodeProvider>>,
        runtime_registry: Arc<RuntimeRegistry>,
    ) -> Self {
        Self {
            providers,
            runtime_registry,
        }
    }

    pub fn get_available_nodes(&self) -> Vec<NodeContract> {
        self.providers
            .iter()
            .filter(|p| self.runtime_registry.get_definition(p.id()).is_some())
            .map(|p| NodeContract {
                id: p.id().to_string(),
                category: p.category().to_string(),
                display_name: p.display_name().to_string(),
                description: p.description().to_string(),
                icon: p.icon().to_string(),
                inputs: p.inputs().iter().map(|s| s.to_string()).collect(),
                outputs: p.outputs().iter().map(|s| s.to_string()).collect(),
                config_schema: p.config_schema(),
                default_template_key: p.default_template_key().map(|value| value.to_string()),
                contract_version: p.contract_version().to_string(),
                capabilities: p.capabilities(),
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
        runtime.register_definition("core.oidc.consent", StepType::Authenticator);
        runtime.register_definition("core.terminal.allow", StepType::Terminal);
        runtime.register_definition("core.terminal.deny", StepType::Terminal);
        runtime.register_definition("core.logic.condition", StepType::Logic);
        runtime.register_definition("core.logic.recovery_issue", StepType::Logic);

        let registry = NodeRegistryService::new(Arc::new(runtime));
        let nodes = registry.get_available_nodes();
        let ids: Vec<String> = nodes.into_iter().map(|node| node.id).collect();

        assert!(ids.iter().any(|id| id == "core.auth.password"));
        assert!(ids.iter().any(|id| id == "core.auth.cookie"));
        assert!(ids.iter().any(|id| id == "core.oidc.consent"));
        assert!(ids.iter().any(|id| id == "core.logic.condition"));
        assert!(ids.iter().any(|id| id == "core.logic.recovery_issue"));
        assert!(!ids.iter().any(|id| id == "core.auth.otp"));
        assert!(!ids.iter().any(|id| id == "core.logic.scripted"));
    }
}
