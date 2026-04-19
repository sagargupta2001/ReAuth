use crate::domain::auth_session::AuthenticationSession;
use crate::domain::execution::lifecycle::{LifecycleNode, NodeOutcome};
use crate::domain::execution::ExecutionPlan;
use crate::error::{Error, Result};
use crate::ports::flow_store::FlowStore;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

pub struct SubflowNode {
    flow_store: Arc<dyn FlowStore>,
}

impl SubflowNode {
    pub fn new(flow_store: Arc<dyn FlowStore>) -> Self {
        Self { flow_store }
    }
}

#[derive(Debug, Deserialize)]
struct SubflowConfig {
    flow_type: Option<String>,
}

fn load_config(session: &AuthenticationSession) -> Result<SubflowConfig> {
    let config = session
        .context
        .get("node_config")
        .cloned()
        .unwrap_or_else(|| json!({}));
    serde_json::from_value(config)
        .map_err(|err| Error::Validation(format!("Invalid subflow config: {}", err)))
}

#[async_trait]
impl LifecycleNode for SubflowNode {
    #[instrument(
        skip_all,
        fields(telemetry = "span", node = "subflow", phase = "execute")
    )]
    async fn execute(&self, session: &mut AuthenticationSession) -> Result<NodeOutcome> {
        let config = load_config(session)?;
        let flow_type = config
            .flow_type
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| Error::Validation("Subflow node requires flow_type".to_string()))?;

        let deployment = self
            .flow_store
            .get_deployment(&session.realm_id, &flow_type)
            .await?
            .ok_or_else(|| {
                Error::Validation(format!(
                    "Subflow deployment '{}' not found in this realm",
                    flow_type
                ))
            })?;

        let version_id = Uuid::parse_str(&deployment.active_version_id)
            .map_err(|_| Error::System("Subflow deployment version id invalid".to_string()))?;
        if version_id == session.flow_version_id {
            return Err(Error::Validation(
                "Subflow cannot target the same deployed flow version".to_string(),
            ));
        }

        let version = self
            .flow_store
            .get_version(&version_id)
            .await?
            .ok_or_else(|| Error::System("Subflow version missing".to_string()))?;
        let plan: ExecutionPlan = serde_json::from_str(&version.execution_artifact)
            .map_err(|err| Error::System(format!("Subflow artifact corrupt: {}", err)))?;

        Ok(NodeOutcome::CallSubflow {
            flow_version_id: version_id,
            start_node_id: plan.start_node_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::auth_session::AuthenticationSession;
    use crate::domain::execution::{ExecutionNode, StepType};
    use crate::domain::flow::models::{FlowDeployment, FlowDraft, FlowVersion};
    use crate::domain::pagination::{PageRequest, PageResponse};
    use crate::ports::transaction_manager::Transaction;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::collections::HashMap;
    use uuid::Uuid;

    #[derive(Default)]
    struct TestFlowStore {
        deployment: Option<FlowDeployment>,
        version: Option<FlowVersion>,
    }

    #[async_trait]
    impl FlowStore for TestFlowStore {
        async fn create_draft(&self, _draft: &FlowDraft) -> Result<()> {
            unreachable!()
        }
        async fn update_draft(&self, _draft: &FlowDraft) -> Result<()> {
            unreachable!()
        }
        async fn get_draft_by_id(&self, _id: &Uuid) -> Result<Option<FlowDraft>> {
            unreachable!()
        }
        async fn list_drafts(
            &self,
            _realm_id: &Uuid,
            _req: &PageRequest,
        ) -> Result<PageResponse<FlowDraft>> {
            unreachable!()
        }
        async fn list_all_drafts(&self, _realm_id: &Uuid) -> Result<Vec<FlowDraft>> {
            unreachable!()
        }
        async fn delete_draft(&self, _id: &Uuid) -> Result<()> {
            unreachable!()
        }
        async fn create_version(&self, _version: &FlowVersion) -> Result<()> {
            unreachable!()
        }
        async fn get_version(&self, _id: &Uuid) -> Result<Option<FlowVersion>> {
            Ok(self.version.clone())
        }
        async fn list_versions(
            &self,
            _flow_id: &Uuid,
            _req: &PageRequest,
        ) -> Result<PageResponse<FlowVersion>> {
            unreachable!()
        }
        async fn set_deployment(&self, _deployment: &FlowDeployment) -> Result<()> {
            unreachable!()
        }
        async fn get_deployment(
            &self,
            _realm_id: &Uuid,
            _flow_type: &str,
        ) -> Result<Option<FlowDeployment>> {
            Ok(self.deployment.clone())
        }
        async fn get_latest_version_number(&self, _flow_id: &Uuid) -> Result<Option<i32>> {
            unreachable!()
        }
        async fn get_latest_version(&self, _flow_id: &Uuid) -> Result<Option<FlowVersion>> {
            unreachable!()
        }
        async fn get_deployed_version_number(
            &self,
            _realm_id: &Uuid,
            _flow_type: &str,
            _flow_id: &Uuid,
        ) -> Result<Option<i32>> {
            unreachable!()
        }
        async fn get_version_by_number(
            &self,
            _flow_id: &Uuid,
            _version_number: i32,
        ) -> Result<Option<FlowVersion>> {
            unreachable!()
        }
        async fn get_active_version(&self, _flow_id: &Uuid) -> Result<Option<FlowVersion>> {
            unreachable!()
        }
        async fn create_draft_with_tx(
            &self,
            _draft: &FlowDraft,
            _tx: Option<&mut dyn Transaction>,
        ) -> Result<()> {
            unreachable!()
        }
        async fn update_draft_with_tx(
            &self,
            _draft: &FlowDraft,
            _tx: Option<&mut dyn Transaction>,
        ) -> Result<()> {
            unreachable!()
        }
        async fn get_draft_by_id_with_tx(
            &self,
            _id: &Uuid,
            _tx: Option<&mut dyn Transaction>,
        ) -> Result<Option<FlowDraft>> {
            unreachable!()
        }
        async fn delete_draft_with_tx(
            &self,
            _id: &Uuid,
            _tx: Option<&mut dyn Transaction>,
        ) -> Result<()> {
            unreachable!()
        }
        async fn create_version_with_tx(
            &self,
            _version: &FlowVersion,
            _tx: Option<&mut dyn Transaction>,
        ) -> Result<()> {
            unreachable!()
        }
        async fn set_deployment_with_tx(
            &self,
            _deployment: &FlowDeployment,
            _tx: Option<&mut dyn Transaction>,
        ) -> Result<()> {
            unreachable!()
        }
    }

    #[tokio::test]
    async fn execute_returns_call_subflow_outcome() {
        let version_id = Uuid::new_v4();
        let plan = ExecutionPlan {
            start_node_id: "child-start".to_string(),
            nodes: HashMap::from([(
                "child-start".to_string(),
                ExecutionNode {
                    id: "child-start".to_string(),
                    step_type: StepType::Logic,
                    next: HashMap::new(),
                    config: json!({}),
                },
            )]),
        };
        let store = Arc::new(TestFlowStore {
            deployment: Some(FlowDeployment {
                id: Uuid::new_v4().to_string(),
                realm_id: Uuid::new_v4(),
                flow_type: "step_up".to_string(),
                active_version_id: version_id.to_string(),
                updated_at: Utc::now(),
            }),
            version: Some(FlowVersion {
                id: version_id.to_string(),
                flow_id: Uuid::new_v4().to_string(),
                version_number: 1,
                execution_artifact: serde_json::to_string(&plan).unwrap(),
                graph_json: "{}".to_string(),
                checksum: "checksum".to_string(),
                node_contract_versions: "{}".to_string(),
                created_at: Utc::now(),
            }),
        });

        let node = SubflowNode::new(store);
        let mut session =
            AuthenticationSession::new(Uuid::new_v4(), Uuid::new_v4(), "parent".to_string());
        session.update_context("node_config", json!({ "flow_type": "step_up" }));

        let outcome = node.execute(&mut session).await.expect("subflow execute");

        match outcome {
            NodeOutcome::CallSubflow {
                flow_version_id,
                start_node_id,
            } => {
                assert_eq!(flow_version_id, version_id);
                assert_eq!(start_node_id, "child-start");
            }
            other => panic!("unexpected outcome: {other:?}"),
        }
    }
}
