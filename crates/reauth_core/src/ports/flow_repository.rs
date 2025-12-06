use crate::error::Error;
use crate::ports::transaction_manager::Transaction;
use crate::{
    domain::auth_flow::{AuthFlow, AuthFlowStep, AuthenticatorConfig, LoginSession},
    error::Result,
};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait FlowRepository: Send + Sync {
    async fn find_flow_by_name(&self, realm_id: &Uuid, name: &str) -> Result<Option<AuthFlow>>;
    async fn find_steps_for_flow(&self, flow_id: &Uuid) -> Result<Vec<AuthFlowStep>>;
    async fn find_config_for_authenticator(
        &self,
        realm_id: &Uuid,
        authenticator_name: &str,
    ) -> Result<Option<AuthenticatorConfig>>;
    async fn create_flow<'a>(
        &self,
        flow: &AuthFlow,
        tx: Option<&'a mut dyn Transaction>,
    ) -> Result<()>;
    async fn add_step_to_flow<'a>(
        &self,
        step: &AuthFlowStep,
        tx: Option<&'a mut dyn Transaction>,
    ) -> Result<()>;
    async fn save_login_session(&self, session: &LoginSession) -> Result<()>;
    async fn find_login_session_by_id(&self, id: &Uuid) -> Result<Option<LoginSession>>;
    async fn update_login_session(&self, session: &LoginSession) -> Result<()>;
    async fn delete_login_session(&self, id: &Uuid) -> Result<()>;
    async fn list_flows_by_realm(&self, realm_id: &Uuid) -> Result<Vec<AuthFlow>>;
}
