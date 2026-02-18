use super::FlowExecutor;
use crate::application::runtime_registry::RuntimeRegistry;
use crate::domain::auth_session::{AuthenticationSession, SessionStatus};
use crate::domain::execution::lifecycle::{LifecycleNode, NodeOutcome};
use crate::domain::execution::{ExecutionNode, ExecutionPlan, ExecutionResult, StepType};
use crate::domain::flow::models::{FlowDeployment, FlowDraft, FlowVersion};
use crate::domain::pagination::{PageRequest, PageResponse};
use crate::error::{Error, Result};
use crate::ports::auth_session_repository::AuthSessionRepository;
use crate::ports::flow_store::FlowStore;
use crate::ports::transaction_manager::Transaction;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Default)]
struct TestAuthSessionRepo {
    sessions: Mutex<HashMap<Uuid, AuthenticationSession>>,
    update_calls: Mutex<Vec<AuthenticationSession>>,
}

impl TestAuthSessionRepo {
    fn insert(&self, session: AuthenticationSession) {
        self.sessions.lock().unwrap().insert(session.id, session);
    }

    fn updates(&self) -> Vec<AuthenticationSession> {
        self.update_calls.lock().unwrap().clone()
    }
}

#[async_trait]
impl AuthSessionRepository for TestAuthSessionRepo {
    async fn create(&self, session: &AuthenticationSession) -> Result<()> {
        self.sessions
            .lock()
            .unwrap()
            .insert(session.id, session.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<AuthenticationSession>> {
        Ok(self.sessions.lock().unwrap().get(id).cloned())
    }

    async fn update(&self, session: &AuthenticationSession) -> Result<()> {
        self.sessions
            .lock()
            .unwrap()
            .insert(session.id, session.clone());
        self.update_calls.lock().unwrap().push(session.clone());
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> Result<()> {
        self.sessions.lock().unwrap().remove(id);
        Ok(())
    }
}

#[derive(Default)]
struct TestFlowStore {
    versions: Mutex<HashMap<Uuid, FlowVersion>>,
}

impl TestFlowStore {
    fn insert_version(&self, id: Uuid, version: FlowVersion) {
        self.versions.lock().unwrap().insert(id, version);
    }
}

#[async_trait]
impl FlowStore for TestFlowStore {
    async fn create_draft(&self, _draft: &FlowDraft) -> Result<()> {
        Ok(())
    }

    async fn update_draft(&self, _draft: &FlowDraft) -> Result<()> {
        Ok(())
    }

    async fn get_draft_by_id(&self, _id: &Uuid) -> Result<Option<FlowDraft>> {
        Ok(None)
    }

    async fn list_drafts(
        &self,
        _realm_id: &Uuid,
        _req: &PageRequest,
    ) -> Result<PageResponse<FlowDraft>> {
        Ok(PageResponse::new(Vec::new(), 0, 1, 10))
    }

    async fn list_all_drafts(&self, _realm_id: &Uuid) -> Result<Vec<FlowDraft>> {
        Ok(Vec::new())
    }

    async fn delete_draft(&self, _id: &Uuid) -> Result<()> {
        Ok(())
    }

    async fn create_version(&self, _version: &FlowVersion) -> Result<()> {
        Ok(())
    }

    async fn get_version(&self, id: &Uuid) -> Result<Option<FlowVersion>> {
        Ok(self.versions.lock().unwrap().get(id).cloned())
    }

    async fn list_versions(
        &self,
        _flow_id: &Uuid,
        _req: &PageRequest,
    ) -> Result<PageResponse<FlowVersion>> {
        Ok(PageResponse::new(Vec::new(), 0, 1, 10))
    }

    async fn set_deployment(&self, _deployment: &FlowDeployment) -> Result<()> {
        Ok(())
    }

    async fn get_deployment(
        &self,
        _realm_id: &Uuid,
        _flow_type: &str,
    ) -> Result<Option<FlowDeployment>> {
        Ok(None)
    }

    async fn get_latest_version_number(&self, _flow_id: &Uuid) -> Result<Option<i32>> {
        Ok(None)
    }

    async fn get_latest_version(&self, _flow_id: &Uuid) -> Result<Option<FlowVersion>> {
        Ok(None)
    }

    async fn get_deployed_version_number(
        &self,
        _realm_id: &Uuid,
        _flow_type: &str,
        _flow_id: &Uuid,
    ) -> Result<Option<i32>> {
        Ok(None)
    }

    async fn get_version_by_number(
        &self,
        _flow_id: &Uuid,
        _version_number: i32,
    ) -> Result<Option<FlowVersion>> {
        Ok(None)
    }

    async fn get_active_version(&self, _flow_id: &Uuid) -> Result<Option<FlowVersion>> {
        Ok(None)
    }

    async fn create_draft_with_tx(
        &self,
        draft: &FlowDraft,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        FlowStore::create_draft_with_tx(self, draft, tx).await
    }

    async fn update_draft_with_tx(
        &self,
        draft: &FlowDraft,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        FlowStore::update_draft_with_tx(self, draft, tx).await
    }

    async fn delete_draft_with_tx(
        &self,
        id: &Uuid,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        FlowStore::delete_draft_with_tx(self, id, tx).await
    }

    async fn create_version_with_tx(
        &self,
        version: &FlowVersion,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        FlowStore::create_version_with_tx(self, version, tx).await
    }

    async fn set_deployment_with_tx(
        &self,
        deployment: &FlowDeployment,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        FlowStore::set_deployment_with_tx(self, deployment, tx).await
    }
}

struct ScriptedNode {
    execute_outcome: Mutex<NodeOutcome>,
    handle_outcome: Mutex<NodeOutcome>,
    handle_calls: Mutex<usize>,
    execute_calls: Mutex<usize>,
}

impl ScriptedNode {
    fn new(execute_outcome: NodeOutcome, handle_outcome: NodeOutcome) -> Self {
        Self {
            execute_outcome: Mutex::new(execute_outcome),
            handle_outcome: Mutex::new(handle_outcome),
            handle_calls: Mutex::new(0),
            execute_calls: Mutex::new(0),
        }
    }

    fn handle_calls(&self) -> usize {
        *self.handle_calls.lock().unwrap()
    }

    fn execute_calls(&self) -> usize {
        *self.execute_calls.lock().unwrap()
    }
}

#[async_trait]
impl LifecycleNode for ScriptedNode {
    async fn execute(&self, _session: &mut AuthenticationSession) -> Result<NodeOutcome> {
        let mut calls = self.execute_calls.lock().unwrap();
        *calls += 1;
        Ok(self.execute_outcome.lock().unwrap().clone())
    }

    async fn handle_input(
        &self,
        _session: &mut AuthenticationSession,
        _input: serde_json::Value,
    ) -> Result<NodeOutcome> {
        let mut calls = self.handle_calls.lock().unwrap();
        *calls += 1;
        Ok(self.handle_outcome.lock().unwrap().clone())
    }
}

fn build_plan(start: &str, nodes: Vec<ExecutionNode>) -> ExecutionPlan {
    let mut map = HashMap::new();
    for node in nodes {
        map.insert(node.id.clone(), node);
    }
    ExecutionPlan {
        start_node_id: start.to_string(),
        nodes: map,
    }
}

fn build_version(id: Uuid, plan: &ExecutionPlan) -> FlowVersion {
    FlowVersion {
        id: id.to_string(),
        flow_id: Uuid::new_v4().to_string(),
        version_number: 1,
        execution_artifact: serde_json::to_string(plan).unwrap(),
        graph_json: json!({}).to_string(),
        checksum: "checksum".to_string(),
        created_at: Utc::now(),
    }
}

#[tokio::test]
async fn execute_handle_input_forces_password_success_path() {
    let realm_id = Uuid::new_v4();
    let version_id = Uuid::new_v4();

    let auth_node = ExecutionNode {
        id: "auth-password".to_string(),
        step_type: StepType::Authenticator,
        next: HashMap::new(),
        config: json!({ "auth_type": "core.auth.password" }),
    };
    let success_node = ExecutionNode {
        id: "success".to_string(),
        step_type: StepType::Terminal,
        next: HashMap::new(),
        config: json!({}),
    };
    let plan = build_plan("auth-password", vec![auth_node, success_node]);
    let version = build_version(version_id, &plan);

    let flow_store = Arc::new(TestFlowStore::default());
    flow_store.insert_version(version_id, version);

    let session = AuthenticationSession::new(realm_id, version_id, "auth-password".to_string());
    let session_id = session.id;
    let repo = Arc::new(TestAuthSessionRepo::default());
    repo.insert(session);

    let node = Arc::new(ScriptedNode::new(
        NodeOutcome::SuspendForUI {
            screen: "unused".to_string(),
            context: json!({}),
        },
        NodeOutcome::Continue {
            output: "success".to_string(),
        },
    ));
    let mut registry = RuntimeRegistry::new();
    registry.register_node("core.auth.password", node, StepType::Authenticator);

    let executor = FlowExecutor::new(repo.clone(), flow_store, Arc::new(registry));
    let result = executor
        .execute(session_id, Some(json!({ "password": "secret" })))
        .await
        .expect("execute failed");

    match result {
        ExecutionResult::Success { redirect_url } => {
            assert_eq!(redirect_url, "/");
        }
        other => panic!("unexpected result: {:?}", other),
    }

    let updates = repo.updates();
    assert_eq!(updates.last().unwrap().status, SessionStatus::Completed);
}

#[tokio::test]
async fn execute_handle_input_rejects_and_returns_ui() {
    let realm_id = Uuid::new_v4();
    let version_id = Uuid::new_v4();

    let auth_node = ExecutionNode {
        id: "auth-password".to_string(),
        step_type: StepType::Authenticator,
        next: HashMap::new(),
        config: json!({ "auth_type": "core.auth.password" }),
    };
    let plan = build_plan("auth-password", vec![auth_node]);
    let version = build_version(version_id, &plan);

    let flow_store = Arc::new(TestFlowStore::default());
    flow_store.insert_version(version_id, version);

    let session = AuthenticationSession::new(realm_id, version_id, "auth-password".to_string());
    let session_id = session.id;
    let repo = Arc::new(TestAuthSessionRepo::default());
    repo.insert(session);

    let node = Arc::new(ScriptedNode::new(
        NodeOutcome::SuspendForUI {
            screen: "login".to_string(),
            context: json!({ "error": "bad_password" }),
        },
        NodeOutcome::Reject {
            error: "bad".to_string(),
        },
    ));
    let mut registry = RuntimeRegistry::new();
    registry.register_node("core.auth.password", node, StepType::Authenticator);

    let executor = FlowExecutor::new(repo, flow_store, Arc::new(registry));
    let result = executor
        .execute(session_id, Some(json!({ "password": "wrong" })))
        .await
        .expect("execute failed");

    match result {
        ExecutionResult::Challenge { screen_id, context } => {
            assert_eq!(screen_id, "login");
            assert_eq!(context["error"], "bad_password");
        }
        other => panic!("unexpected result: {:?}", other),
    }
}

#[tokio::test]
async fn execute_reject_without_ui_is_error() {
    let realm_id = Uuid::new_v4();
    let version_id = Uuid::new_v4();

    let auth_node = ExecutionNode {
        id: "auth-password".to_string(),
        step_type: StepType::Authenticator,
        next: HashMap::new(),
        config: json!({ "auth_type": "core.auth.password" }),
    };
    let plan = build_plan("auth-password", vec![auth_node]);
    let version = build_version(version_id, &plan);

    let flow_store = Arc::new(TestFlowStore::default());
    flow_store.insert_version(version_id, version);

    let session = AuthenticationSession::new(realm_id, version_id, "auth-password".to_string());
    let session_id = session.id;
    let repo = Arc::new(TestAuthSessionRepo::default());
    repo.insert(session);

    let node = Arc::new(ScriptedNode::new(
        NodeOutcome::Continue {
            output: "success".to_string(),
        },
        NodeOutcome::Reject {
            error: "bad".to_string(),
        },
    ));
    let mut registry = RuntimeRegistry::new();
    registry.register_node("core.auth.password", node, StepType::Authenticator);

    let executor = FlowExecutor::new(repo, flow_store, Arc::new(registry));
    let err = executor
        .execute(session_id, Some(json!({ "password": "wrong" })))
        .await
        .expect_err("expected error");

    match err {
        Error::System(message) => {
            assert!(message.contains("Rejecting node did not return UI"));
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

#[tokio::test]
async fn execute_rejects_unexpected_handle_input_outcome() {
    let realm_id = Uuid::new_v4();
    let version_id = Uuid::new_v4();

    let auth_node = ExecutionNode {
        id: "auth-password".to_string(),
        step_type: StepType::Authenticator,
        next: HashMap::new(),
        config: json!({ "auth_type": "core.auth.password" }),
    };
    let plan = build_plan("auth-password", vec![auth_node]);
    let version = build_version(version_id, &plan);

    let flow_store = Arc::new(TestFlowStore::default());
    flow_store.insert_version(version_id, version);

    let session = AuthenticationSession::new(realm_id, version_id, "auth-password".to_string());
    let session_id = session.id;
    let repo = Arc::new(TestAuthSessionRepo::default());
    repo.insert(session);

    let node = Arc::new(ScriptedNode::new(
        NodeOutcome::SuspendForUI {
            screen: "unused".to_string(),
            context: json!({}),
        },
        NodeOutcome::FlowSuccess {
            user_id: Uuid::new_v4(),
        },
    ));
    let mut registry = RuntimeRegistry::new();
    registry.register_node("core.auth.password", node, StepType::Authenticator);

    let executor = FlowExecutor::new(repo, flow_store, Arc::new(registry));
    let err = executor
        .execute(session_id, Some(json!({ "password": "ok" })))
        .await
        .expect_err("expected error");

    match err {
        Error::System(message) => {
            assert!(message.contains("Unexpected outcome from handle_input"));
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

#[tokio::test]
async fn execute_rejects_handle_input_without_path() {
    let realm_id = Uuid::new_v4();
    let version_id = Uuid::new_v4();

    let auth_node = ExecutionNode {
        id: "auth-otp".to_string(),
        step_type: StepType::Authenticator,
        next: HashMap::new(),
        config: json!({ "auth_type": "core.auth.password" }),
    };
    let plan = build_plan("auth-otp", vec![auth_node]);
    let version = build_version(version_id, &plan);

    let flow_store = Arc::new(TestFlowStore::default());
    flow_store.insert_version(version_id, version);

    let session = AuthenticationSession::new(realm_id, version_id, "auth-otp".to_string());
    let session_id = session.id;
    let repo = Arc::new(TestAuthSessionRepo::default());
    repo.insert(session);

    let node = Arc::new(ScriptedNode::new(
        NodeOutcome::SuspendForUI {
            screen: "unused".to_string(),
            context: json!({}),
        },
        NodeOutcome::Continue {
            output: "unknown".to_string(),
        },
    ));
    let mut registry = RuntimeRegistry::new();
    registry.register_node("core.auth.password", node, StepType::Authenticator);

    let executor = FlowExecutor::new(repo, flow_store, Arc::new(registry));
    let err = executor
        .execute(session_id, Some(json!({ "otp": "123456" })))
        .await
        .expect_err("expected error");

    match err {
        Error::Validation(message) => {
            assert!(message.contains("No path forward"));
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

#[tokio::test]
async fn execute_errors_when_node_missing_from_plan() {
    let realm_id = Uuid::new_v4();
    let version_id = Uuid::new_v4();

    let plan = build_plan(
        "start",
        vec![ExecutionNode {
            id: "start".to_string(),
            step_type: StepType::Terminal,
            next: HashMap::new(),
            config: json!({}),
        }],
    );
    let version = build_version(version_id, &plan);

    let flow_store = Arc::new(TestFlowStore::default());
    flow_store.insert_version(version_id, version);

    let session = AuthenticationSession::new(realm_id, version_id, "missing".to_string());
    let session_id = session.id;
    let repo = Arc::new(TestAuthSessionRepo::default());
    repo.insert(session);

    let executor = FlowExecutor::new(repo, flow_store, Arc::new(RuntimeRegistry::new()));
    let err = executor
        .execute(session_id, None)
        .await
        .expect_err("expected error");

    match err {
        Error::System(message) => {
            assert!(message.contains("Node missing from graph"));
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

#[tokio::test]
async fn execute_logic_node_without_outputs_is_error() {
    let realm_id = Uuid::new_v4();
    let version_id = Uuid::new_v4();

    let plan = build_plan(
        "logic",
        vec![ExecutionNode {
            id: "logic".to_string(),
            step_type: StepType::Logic,
            next: HashMap::new(),
            config: json!({}),
        }],
    );
    let version = build_version(version_id, &plan);

    let flow_store = Arc::new(TestFlowStore::default());
    flow_store.insert_version(version_id, version);

    let session = AuthenticationSession::new(realm_id, version_id, "logic".to_string());
    let session_id = session.id;
    let repo = Arc::new(TestAuthSessionRepo::default());
    repo.insert(session);

    let executor = FlowExecutor::new(repo, flow_store, Arc::new(RuntimeRegistry::new()));
    let err = executor
        .execute(session_id, None)
        .await
        .expect_err("expected error");

    match err {
        Error::System(message) => {
            assert!(message.contains("Logic node has no output"));
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

#[tokio::test]
async fn execute_terminal_failure_sets_status() {
    let realm_id = Uuid::new_v4();
    let version_id = Uuid::new_v4();

    let plan = build_plan(
        "terminal",
        vec![ExecutionNode {
            id: "terminal".to_string(),
            step_type: StepType::Terminal,
            next: HashMap::new(),
            config: json!({ "is_failure": true }),
        }],
    );
    let version = build_version(version_id, &plan);

    let flow_store = Arc::new(TestFlowStore::default());
    flow_store.insert_version(version_id, version);

    let session = AuthenticationSession::new(realm_id, version_id, "terminal".to_string());
    let session_id = session.id;
    let repo = Arc::new(TestAuthSessionRepo::default());
    repo.insert(session);

    let executor = FlowExecutor::new(repo.clone(), flow_store, Arc::new(RuntimeRegistry::new()));
    let result = executor.execute(session_id, None).await.unwrap();

    match result {
        ExecutionResult::Failure { reason } => {
            assert!(reason.contains("Access Denied"));
        }
        other => panic!("unexpected result: {:?}", other),
    }

    let updates = repo.updates();
    assert_eq!(updates.last().unwrap().status, SessionStatus::Failed);
}

#[tokio::test]
async fn execute_terminal_success_sets_status() {
    let realm_id = Uuid::new_v4();
    let version_id = Uuid::new_v4();

    let plan = build_plan(
        "terminal",
        vec![ExecutionNode {
            id: "terminal".to_string(),
            step_type: StepType::Terminal,
            next: HashMap::new(),
            config: json!({}),
        }],
    );
    let version = build_version(version_id, &plan);

    let flow_store = Arc::new(TestFlowStore::default());
    flow_store.insert_version(version_id, version);

    let session = AuthenticationSession::new(realm_id, version_id, "terminal".to_string());
    let session_id = session.id;
    let repo = Arc::new(TestAuthSessionRepo::default());
    repo.insert(session);

    let executor = FlowExecutor::new(repo.clone(), flow_store, Arc::new(RuntimeRegistry::new()));
    let result = executor.execute(session_id, None).await.unwrap();

    match result {
        ExecutionResult::Success { redirect_url } => {
            assert_eq!(redirect_url, "/");
        }
        other => panic!("unexpected result: {:?}", other),
    }

    let updates = repo.updates();
    assert_eq!(updates.last().unwrap().status, SessionStatus::Completed);
}

#[tokio::test]
async fn execute_errors_when_worker_missing_for_input() {
    let realm_id = Uuid::new_v4();
    let version_id = Uuid::new_v4();

    let auth_node = ExecutionNode {
        id: "auth-password".to_string(),
        step_type: StepType::Authenticator,
        next: HashMap::new(),
        config: json!({ "auth_type": "core.auth.password" }),
    };
    let plan = build_plan("auth-password", vec![auth_node]);
    let version = build_version(version_id, &plan);

    let flow_store = Arc::new(TestFlowStore::default());
    flow_store.insert_version(version_id, version);

    let session = AuthenticationSession::new(realm_id, version_id, "auth-password".to_string());
    let session_id = session.id;
    let repo = Arc::new(TestAuthSessionRepo::default());
    repo.insert(session);

    let executor = FlowExecutor::new(repo, flow_store, Arc::new(RuntimeRegistry::new()));
    let err = executor
        .execute(session_id, Some(json!({ "password": "secret" })))
        .await
        .expect_err("expected error");

    match err {
        Error::System(message) => {
            assert!(message.contains("Worker not found"));
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

#[tokio::test]
async fn execute_heals_inactive_session_and_ignores_input() {
    let realm_id = Uuid::new_v4();
    let version_id = Uuid::new_v4();

    let auth_node = ExecutionNode {
        id: "auth-password".to_string(),
        step_type: StepType::Authenticator,
        next: HashMap::new(),
        config: json!({ "auth_type": "core.auth.password" }),
    };
    let plan = build_plan("auth-password", vec![auth_node]);
    let version = build_version(version_id, &plan);

    let flow_store = Arc::new(TestFlowStore::default());
    flow_store.insert_version(version_id, version);

    let mut session = AuthenticationSession::new(realm_id, version_id, "auth-password".to_string());
    session.status = SessionStatus::Failed;
    session.user_id = Some(Uuid::new_v4());
    let session_id = session.id;
    let repo = Arc::new(TestAuthSessionRepo::default());
    repo.insert(session);

    let node = Arc::new(ScriptedNode::new(
        NodeOutcome::SuspendForUI {
            screen: "login".to_string(),
            context: json!({}),
        },
        NodeOutcome::Continue {
            output: "success".to_string(),
        },
    ));
    let mut registry = RuntimeRegistry::new();
    registry.register_node("core.auth.password", node.clone(), StepType::Authenticator);

    let executor = FlowExecutor::new(repo.clone(), flow_store, Arc::new(registry));
    let result = executor
        .execute(session_id, Some(json!({ "password": "secret" })))
        .await
        .expect("execute failed");

    match result {
        ExecutionResult::Challenge { screen_id, .. } => {
            assert_eq!(screen_id, "login");
        }
        other => panic!("unexpected result: {:?}", other),
    }

    assert_eq!(node.handle_calls(), 0);
    assert_eq!(node.execute_calls(), 1);

    let updates = repo.updates();
    assert!(updates.iter().any(|s| s.status == SessionStatus::Active));
    assert!(updates.iter().any(|s| s.user_id.is_none()));
}
