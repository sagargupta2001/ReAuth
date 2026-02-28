use super::*;
use crate::error::Error;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Default)]
struct TestFlowRepo {
    flows: Mutex<HashMap<Uuid, AuthFlow>>,
    create_calls: Mutex<Vec<AuthFlow>>,
    create_tx_calls: Mutex<usize>,
    find_flow_error: Mutex<Option<Error>>,
    create_flow_error: Mutex<Option<Error>>,
}

impl TestFlowRepo {
    fn insert_flow(&self, flow: AuthFlow) {
        self.flows.lock().unwrap().insert(flow.id, flow);
    }

    fn set_find_flow_error(&self, error: Error) {
        *self.find_flow_error.lock().unwrap() = Some(error);
    }

    fn set_create_flow_error(&self, error: Error) {
        *self.create_flow_error.lock().unwrap() = Some(error);
    }

    fn create_calls(&self) -> Vec<AuthFlow> {
        self.create_calls.lock().unwrap().clone()
    }

    fn create_tx_calls(&self) -> usize {
        *self.create_tx_calls.lock().unwrap()
    }
}

#[async_trait]
impl FlowRepository for TestFlowRepo {
    async fn find_flow_by_name(&self, realm_id: &Uuid, name: &str) -> Result<Option<AuthFlow>> {
        if let Some(err) = self.find_flow_error.lock().unwrap().take() {
            return Err(err);
        }

        Ok(self
            .flows
            .lock()
            .unwrap()
            .values()
            .find(|flow| flow.realm_id == *realm_id && flow.name == name)
            .cloned())
    }

    async fn find_flow_by_id(&self, flow_id: &Uuid) -> Result<Option<AuthFlow>> {
        Ok(self.flows.lock().unwrap().get(flow_id).cloned())
    }

    async fn create_flow<'a>(
        &self,
        flow: &AuthFlow,
        tx: Option<&'a mut dyn Transaction>,
    ) -> Result<()> {
        if let Some(err) = self.create_flow_error.lock().unwrap().take() {
            return Err(err);
        }

        if tx.is_some() {
            *self.create_tx_calls.lock().unwrap() += 1;
        }

        self.flows.lock().unwrap().insert(flow.id, flow.clone());
        self.create_calls.lock().unwrap().push(flow.clone());
        Ok(())
    }

    async fn list_flows_by_realm(&self, realm_id: &Uuid) -> Result<Vec<AuthFlow>> {
        Ok(self
            .flows
            .lock()
            .unwrap()
            .values()
            .filter(|flow| flow.realm_id == *realm_id)
            .cloned()
            .collect())
    }
}

#[derive(Default)]
struct DummyTx;

impl Transaction for DummyTx {
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

fn build_flow(realm_id: Uuid, name: &str, alias: &str, type_: &str) -> AuthFlow {
    AuthFlow {
        id: Uuid::new_v4(),
        realm_id,
        name: name.to_string(),
        alias: alias.to_string(),
        description: Some(format!("Default {alias} flow")),
        r#type: type_.to_string(),
        built_in: true,
    }
}

#[tokio::test]
async fn list_flows_returns_repo_data() {
    let repo = Arc::new(TestFlowRepo::default());
    let service = FlowService::new(repo.clone());
    let realm_id = Uuid::new_v4();

    let flow = build_flow(realm_id, "browser-login", "browser", "browser");
    repo.insert_flow(flow.clone());

    let flows = service.list_flows(realm_id).await.unwrap();
    assert_eq!(flows.len(), 1);
    assert_eq!(flows[0].id, flow.id);
}

#[tokio::test]
async fn setup_default_flows_creates_all_defaults() {
    let repo = Arc::new(TestFlowRepo::default());
    let service = FlowService::new(repo.clone());
    let realm_id = Uuid::new_v4();

    let defaults = service
        .setup_default_flows_for_realm(realm_id, None)
        .await
        .unwrap();

    let calls = repo.create_calls();
    assert_eq!(calls.len(), 4);

    let mut by_name: HashMap<String, AuthFlow> = HashMap::new();
    for flow in calls {
        by_name.insert(flow.name.clone(), flow);
    }

    let browser = by_name.get("browser-login").unwrap();
    assert_eq!(browser.alias, "Browser Login".to_string());
    assert_eq!(browser.r#type, "browser");
    assert!(browser.built_in);

    let direct = by_name.get("direct-grant").unwrap();
    assert_eq!(direct.alias, "Direct Grant".to_string());
    assert_eq!(direct.r#type, "direct");

    let registration = by_name.get("registration").unwrap();
    assert_eq!(registration.alias, "Registration".to_string());
    assert_eq!(registration.r#type, "registration");

    let reset = by_name.get("reset-credentials").unwrap();
    assert_eq!(reset.alias, "Reset Credentials".to_string());
    assert_eq!(reset.r#type, "reset");

    assert_eq!(defaults.browser_flow_id, browser.id);
    assert_eq!(defaults.direct_grant_flow_id, direct.id);
    assert_eq!(defaults.registration_flow_id, registration.id);
    assert_eq!(defaults.reset_credentials_flow_id, reset.id);
}

#[tokio::test]
async fn setup_default_flows_reuses_existing_flows() {
    let repo = Arc::new(TestFlowRepo::default());
    let service = FlowService::new(repo.clone());
    let realm_id = Uuid::new_v4();

    let existing = build_flow(realm_id, "browser-login", "browser", "browser");
    let existing_id = existing.id;
    repo.insert_flow(existing);

    let defaults = service
        .setup_default_flows_for_realm(realm_id, None)
        .await
        .unwrap();

    assert_eq!(defaults.browser_flow_id, existing_id);

    let calls = repo.create_calls();
    assert_eq!(calls.len(), 3);
    assert!(!calls.iter().any(|flow| flow.name == "browser-login"));
}

#[tokio::test]
async fn setup_default_flows_passes_transaction_to_repo() {
    let repo = Arc::new(TestFlowRepo::default());
    let service = FlowService::new(repo.clone());
    let realm_id = Uuid::new_v4();

    let mut tx = DummyTx;

    service
        .setup_default_flows_for_realm(realm_id, Some(&mut tx))
        .await
        .unwrap();

    assert_eq!(repo.create_tx_calls(), 4);
}

#[tokio::test]
async fn setup_default_flows_propagates_find_errors() {
    let repo = Arc::new(TestFlowRepo::default());
    let service = FlowService::new(repo.clone());

    repo.set_find_flow_error(Error::System("find failed".to_string()));

    let result = service
        .setup_default_flows_for_realm(Uuid::new_v4(), None)
        .await;

    assert!(matches!(result, Err(Error::System(message)) if message.contains("find failed")));
}

#[tokio::test]
async fn setup_default_flows_propagates_create_errors() {
    let repo = Arc::new(TestFlowRepo::default());
    let service = FlowService::new(repo.clone());

    repo.set_create_flow_error(Error::System("create failed".to_string()));

    let result = service
        .setup_default_flows_for_realm(Uuid::new_v4(), None)
        .await;

    assert!(matches!(result, Err(Error::System(message)) if message.contains("create failed")));
}
