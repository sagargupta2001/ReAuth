use crate::domain::flow::models::NodeMetadata;
use crate::domain::flow::nodes::condition_node::ConditionNode;
use crate::domain::flow::nodes::cookie_node::CookieNodeProvider;
use crate::domain::flow::nodes::otp_node::OtpNode;
use crate::domain::flow::nodes::password_node::PasswordNodeProvider;
use crate::domain::flow::nodes::script_node::ScriptNode;
use crate::domain::flow::nodes::start_node::StartNode;
use crate::domain::flow::nodes::terminal_node::{AllowNode, DenyNode};
use crate::domain::flow::provider::NodeProvider;

pub struct NodeRegistryService {
    providers: Vec<Box<dyn NodeProvider>>,
}

impl NodeRegistryService {
    pub fn new() -> Self {
        Self {
            providers: vec![
                Box::new(StartNode),
                Box::new(CookieNodeProvider),
                Box::new(PasswordNodeProvider),
                Box::new(OtpNode),
                Box::new(ConditionNode),
                Box::new(ScriptNode),
                Box::new(AllowNode),
                Box::new(DenyNode),
            ],
        }
    }

    pub fn get_available_nodes(&self) -> Vec<NodeMetadata> {
        self.providers
            .iter()
            .map(|p| NodeMetadata {
                id: p.id().to_string(),
                category: p.category().to_string(),
                display_name: p.display_name().to_string(),
                description: p.description().to_string(),
                icon: p.icon().to_string(),
                inputs: p.inputs().iter().map(|s| s.to_string()).collect(),
                outputs: p.outputs().iter().map(|s| s.to_string()).collect(),
                config_schema: p.config_schema(),
            })
            .collect()
    }
}

impl Default for NodeRegistryService {
    fn default() -> Self {
        Self::new()
    }
}
