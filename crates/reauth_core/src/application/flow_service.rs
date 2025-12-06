use crate::domain::auth_flow::AuthFlowStep;
use crate::{domain::auth_flow::AuthFlow, error::Result, ports::flow_repository::FlowRepository};
use std::sync::Arc;
use uuid::Uuid;

// A helper struct to return the IDs of the created defaults
pub struct DefaultFlows {
    pub browser_flow_id: Uuid,
    pub registration_flow_id: Uuid,
    pub direct_grant_flow_id: Uuid,
    pub reset_credentials_flow_id: Uuid,
}

pub struct FlowService {
    flow_repo: Arc<dyn FlowRepository>,
}

impl FlowService {
    pub fn new(flow_repo: Arc<dyn FlowRepository>) -> Self {
        Self { flow_repo }
    }

    pub async fn list_flows(&self, realm_id: Uuid) -> Result<Vec<AuthFlow>> {
        self.flow_repo.list_flows_by_realm(&realm_id).await
    }

    /// Creates standard built-in flows (Browser, Direct, etc.) for a specific realm.
    pub async fn setup_default_flows_for_realm(&self, realm_id: Uuid) -> Result<DefaultFlows> {
        let browser_flow_id = self
            .create_builtin_flow(
                realm_id,
                "browser-login",
                "Browser Login",
                "browser",
                vec!["builtin-password-auth"],
            )
            .await?;

        let direct_grant_flow_id = self
            .create_builtin_flow(
                realm_id,
                "direct-grant",
                "Direct Grant",
                "direct",
                vec!["builtin-password-auth"],
            )
            .await?;

        let registration_flow_id = self
            .create_builtin_flow(
                realm_id,
                "registration",
                "Registration",
                "registration",
                vec![],
            )
            .await?;

        let reset_credentials_flow_id = self
            .create_builtin_flow(
                realm_id,
                "reset-credentials",
                "Reset Credentials",
                "reset",
                vec![],
            )
            .await?;

        Ok(DefaultFlows {
            browser_flow_id,
            direct_grant_flow_id,
            registration_flow_id,
            reset_credentials_flow_id,
        })
    }

    // Helper to create flow + steps
    async fn create_builtin_flow(
        &self,
        realm_id: Uuid,
        name: &str,
        alias: &str,
        type_: &str,
        steps: Vec<&str>,
    ) -> Result<Uuid> {
        // Idempotency check: If it exists, return ID
        if let Some(flow) = self.flow_repo.find_flow_by_name(&realm_id, name).await? {
            return Ok(flow.id);
        }

        let id = Uuid::new_v4();
        let flow = AuthFlow {
            id,
            realm_id,
            name: name.to_string(),
            alias: alias.to_string(),
            description: Some(format!("Default {} flow", alias)),
            r#type: type_.to_string(),
            built_in: true,
        };
        self.flow_repo.create_flow(&flow).await?;

        for (i, auth_name) in steps.iter().enumerate() {
            let step = AuthFlowStep {
                id: Uuid::new_v4(),
                flow_id: id,
                authenticator_name: auth_name.to_string(),
                priority: (i as i64) * 10,
                requirement: "REQUIRED".to_string(),
                config: None,
                parent_step_id: None,
            };
            self.flow_repo.add_step_to_flow(&step).await?;
        }

        Ok(id)
    }
}
