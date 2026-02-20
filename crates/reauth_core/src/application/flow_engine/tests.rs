use super::*;
use crate::domain::auth_session::{AuthenticationSession, SessionStatus};
use crate::domain::execution::lifecycle::LifecycleNode;
use crate::domain::execution::ExecutionNode;
use crate::domain::flow::models::FlowVersion;
use crate::ports::auth_session_repository::AuthSessionRepository;
use crate::ports::flow_store::FlowStore;
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Default)]
struct TestSessionRepo {
    session: Mutex<Option<AuthenticationSession>>,
    updates: Mutex<Vec<AuthenticationSession>>,
}

#[async_trait]
impl AuthSessionRepository for TestSessionRepo {
    async fn create(&self, session: &AuthenticationSession) -> Result<()> {
        *self.session.lock().unwrap() = Some(session.clone());
        Ok(())
    }

    async fn find_by_id(&self, _id: &Uuid) -> Result<Option<AuthenticationSession>> {
        Ok(self.session.lock().unwrap().clone())
    }

    async fn update(&self, session: &AuthenticationSession) -> Result<()> {
        *self.session.lock().unwrap() = Some(session.clone());
        self.updates.lock().unwrap().push(session.clone());
        Ok(())
    }

    async fn delete(&self, _id: &Uuid) -> Result<()> {
        *self.session.lock().unwrap() = None;
        Ok(())
    }
}

#[derive(Default)]
struct TestFlowStore {
    version: Mutex<Option<FlowVersion>>,
}

impl TestFlowStore {
    fn set_version(&self, version: FlowVersion) {
        *self.version.lock().unwrap() = Some(version);
    }
}

#[async_trait]
impl FlowStore for TestFlowStore {
    async fn create_draft(&self, _draft: &crate::domain::flow::models::FlowDraft) -> Result<()> {
        Ok(())
    }

    async fn update_draft(&self, _draft: &crate::domain::flow::models::FlowDraft) -> Result<()> {
        Ok(())
    }

    async fn get_draft_by_id(
        &self,
        _id: &Uuid,
    ) -> Result<Option<crate::domain::flow::models::FlowDraft>> {
        Ok(None)
    }

    async fn list_drafts(
        &self,
        _realm_id: &Uuid,
        _req: &crate::domain::pagination::PageRequest,
    ) -> Result<crate::domain::pagination::PageResponse<crate::domain::flow::models::FlowDraft>>
    {
        Ok(crate::domain::pagination::PageResponse::new(
            Vec::new(),
            0,
            1,
            20,
        ))
    }

    async fn list_all_drafts(
        &self,
        _realm_id: &Uuid,
    ) -> Result<Vec<crate::domain::flow::models::FlowDraft>> {
        Ok(Vec::new())
    }

    async fn delete_draft(&self, _id: &Uuid) -> Result<()> {
        Ok(())
    }

    async fn create_version(&self, _version: &FlowVersion) -> Result<()> {
        Ok(())
    }

    async fn get_version(&self, _id: &Uuid) -> Result<Option<FlowVersion>> {
        Ok(self.version.lock().unwrap().clone())
    }

    async fn list_versions(
        &self,
        _flow_id: &Uuid,
        _req: &crate::domain::pagination::PageRequest,
    ) -> Result<crate::domain::pagination::PageResponse<FlowVersion>> {
        Ok(crate::domain::pagination::PageResponse::new(
            Vec::new(),
            0,
            1,
            20,
        ))
    }

    async fn set_deployment(
        &self,
        _deployment: &crate::domain::flow::models::FlowDeployment,
    ) -> Result<()> {
        Ok(())
    }

    async fn get_deployment(
        &self,
        _realm_id: &Uuid,
        _flow_type: &str,
    ) -> Result<Option<crate::domain::flow::models::FlowDeployment>> {
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
}

#[derive(Clone)]
struct TestNode {
    execute_outcome: NodeOutcome,
    handle_outcome: Option<NodeOutcome>,
}

#[async_trait]
impl LifecycleNode for TestNode {
    async fn execute(&self, _session: &mut AuthenticationSession) -> Result<NodeOutcome> {
        Ok(self.execute_outcome.clone())
    }

    async fn handle_input(
        &self,
        _session: &mut AuthenticationSession,
        _input: serde_json::Value,
    ) -> Result<NodeOutcome> {
        Ok(self.handle_outcome.clone().unwrap_or(NodeOutcome::Reject {
            error: "rejected".to_string(),
        }))
    }
}

fn build_plan(start: &str, nodes: HashMap<String, ExecutionNode>) -> ExecutionPlan {
    ExecutionPlan {
        start_node_id: start.to_string(),
        nodes,
    }
}

fn build_version(artifact: &str) -> FlowVersion {
    FlowVersion {
        id: "v1".to_string(),
        flow_id: "flow".to_string(),
        version_number: 1,
        execution_artifact: artifact.to_string(),
        graph_json: "{}".to_string(),
        checksum: "checksum".to_string(),
        created_at: chrono::Utc::now(),
    }
}

#[tokio::test]
async fn execute_returns_not_found_when_session_missing() {
    let registry = Arc::new(RuntimeRegistry::new());
    let flow_store = Arc::new(TestFlowStore::default());
    let session_repo = Arc::new(TestSessionRepo::default());
    let engine = FlowEngine::new(registry, flow_store, session_repo);

    let result = engine.execute(Uuid::new_v4(), None).await;
    assert!(matches!(result, Err(Error::NotFound(_))));
}

#[tokio::test]
async fn execute_rejects_inactive_session() {
    let registry = Arc::new(RuntimeRegistry::new());
    let flow_store = Arc::new(TestFlowStore::default());
    let session_repo = Arc::new(TestSessionRepo::default());
    let engine = FlowEngine::new(registry, flow_store, session_repo.clone());

    let mut session =
        AuthenticationSession::new(Uuid::new_v4(), Uuid::new_v4(), "start".to_string());
    session.status = SessionStatus::Completed;
    session_repo.create(&session).await.unwrap();

    let result = engine.execute(session.id, None).await;
    assert!(matches!(result, Err(Error::Validation(_))));
}

#[tokio::test]
async fn execute_errors_when_flow_version_missing() {
    let registry = Arc::new(RuntimeRegistry::new());
    let flow_store = Arc::new(TestFlowStore::default());
    let session_repo = Arc::new(TestSessionRepo::default());
    let engine = FlowEngine::new(registry, flow_store, session_repo.clone());

    let session = AuthenticationSession::new(Uuid::new_v4(), Uuid::new_v4(), "start".to_string());
    session_repo.create(&session).await.unwrap();

    let result = engine.execute(session.id, None).await;
    assert!(
        matches!(result, Err(Error::System(message)) if message.contains("Flow version missing"))
    );
}

#[tokio::test]
async fn execute_errors_on_corrupt_artifact() {
    let registry = Arc::new(RuntimeRegistry::new());
    let flow_store = Arc::new(TestFlowStore::default());
    let session_repo = Arc::new(TestSessionRepo::default());
    let engine = FlowEngine::new(registry, flow_store.clone(), session_repo.clone());

    let session = AuthenticationSession::new(Uuid::new_v4(), Uuid::new_v4(), "start".to_string());
    session_repo.create(&session).await.unwrap();
    flow_store.set_version(build_version("not-json"));

    let result = engine.execute(session.id, None).await;
    assert!(matches!(result, Err(Error::System(message)) if message.contains("Corrupt artifact")));
}

#[tokio::test]
async fn execute_returns_show_ui_on_suspend() {
    let mut registry = RuntimeRegistry::new();
    let node = Arc::new(TestNode {
        execute_outcome: NodeOutcome::SuspendForUI {
            screen: "login".to_string(),
            context: json!({"error": "none"}),
        },
        handle_outcome: None,
    });
    registry.register_node("core.auth.password", node, StepType::Authenticator);

    let plan = build_plan(
        "auth",
        [(
            "auth".to_string(),
            ExecutionNode {
                id: "auth".to_string(),
                step_type: StepType::Authenticator,
                next: HashMap::new(),
                config: json!({"auth_type": "core.auth.password"}),
            },
        )]
        .into_iter()
        .collect(),
    );

    let flow_store = Arc::new(TestFlowStore::default());
    flow_store.set_version(build_version(&serde_json::to_string(&plan).unwrap()));

    let session_repo = Arc::new(TestSessionRepo::default());
    let session = AuthenticationSession::new(Uuid::new_v4(), Uuid::new_v4(), "auth".to_string());
    session_repo.create(&session).await.unwrap();

    let engine = FlowEngine::new(Arc::new(registry), flow_store, session_repo.clone());

    let result = engine.execute(session.id, None).await.unwrap();
    match result {
        EngineResult::ShowUI { screen_id, .. } => assert_eq!(screen_id, "login"),
        other => panic!("expected ShowUI, got {other:?}"),
    }

    assert_eq!(session_repo.updates.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn execute_handles_flow_success() {
    let mut registry = RuntimeRegistry::new();
    let node = Arc::new(TestNode {
        execute_outcome: NodeOutcome::FlowSuccess {
            user_id: Uuid::new_v4(),
        },
        handle_outcome: None,
    });
    registry.register_node("core.auth.password", node, StepType::Authenticator);

    let plan = build_plan(
        "auth",
        [(
            "auth".to_string(),
            ExecutionNode {
                id: "auth".to_string(),
                step_type: StepType::Authenticator,
                next: HashMap::new(),
                config: json!({"auth_type": "core.auth.password"}),
            },
        )]
        .into_iter()
        .collect(),
    );

    let flow_store = Arc::new(TestFlowStore::default());
    flow_store.set_version(build_version(&serde_json::to_string(&plan).unwrap()));

    let session_repo = Arc::new(TestSessionRepo::default());
    let session = AuthenticationSession::new(Uuid::new_v4(), Uuid::new_v4(), "auth".to_string());
    session_repo.create(&session).await.unwrap();

    let engine = FlowEngine::new(Arc::new(registry), flow_store, session_repo.clone());

    let result = engine.execute(session.id, None).await.unwrap();
    match result {
        EngineResult::Redirect { url } => assert_eq!(url, "/"),
        other => panic!("expected Redirect, got {other:?}"),
    }

    let updated = session_repo.updates.lock().unwrap();
    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0].status, SessionStatus::Completed);
}

#[tokio::test]
async fn execute_handles_flow_failure() {
    let mut registry = RuntimeRegistry::new();
    let node = Arc::new(TestNode {
        execute_outcome: NodeOutcome::FlowFailure {
            reason: "nope".to_string(),
        },
        handle_outcome: None,
    });
    registry.register_node("core.auth.password", node, StepType::Authenticator);

    let plan = build_plan(
        "auth",
        [(
            "auth".to_string(),
            ExecutionNode {
                id: "auth".to_string(),
                step_type: StepType::Authenticator,
                next: HashMap::new(),
                config: json!({"auth_type": "core.auth.password"}),
            },
        )]
        .into_iter()
        .collect(),
    );

    let flow_store = Arc::new(TestFlowStore::default());
    flow_store.set_version(build_version(&serde_json::to_string(&plan).unwrap()));

    let session_repo = Arc::new(TestSessionRepo::default());
    let session = AuthenticationSession::new(Uuid::new_v4(), Uuid::new_v4(), "auth".to_string());
    session_repo.create(&session).await.unwrap();

    let engine = FlowEngine::new(Arc::new(registry), flow_store, session_repo.clone());

    let result = engine.execute(session.id, None).await.unwrap();
    match result {
        EngineResult::ShowUI { screen_id, context } => {
            assert_eq!(screen_id, "error");
            assert_eq!(context.get("message").unwrap(), "nope");
        }
        other => panic!("expected ShowUI, got {other:?}"),
    }

    let updated = session_repo.updates.lock().unwrap();
    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0].status, SessionStatus::Failed);
}

#[tokio::test]
async fn execute_handles_reject_from_handle_input() {
    let mut registry = RuntimeRegistry::new();
    let node = Arc::new(TestNode {
        execute_outcome: NodeOutcome::SuspendForUI {
            screen: "login".to_string(),
            context: json!({"error": "invalid"}),
        },
        handle_outcome: Some(NodeOutcome::Reject {
            error: "invalid".to_string(),
        }),
    });
    registry.register_node("core.auth.password", node, StepType::Authenticator);

    let plan = build_plan(
        "auth",
        [(
            "auth".to_string(),
            ExecutionNode {
                id: "auth".to_string(),
                step_type: StepType::Authenticator,
                next: HashMap::new(),
                config: json!({"auth_type": "core.auth.password"}),
            },
        )]
        .into_iter()
        .collect(),
    );

    let flow_store = Arc::new(TestFlowStore::default());
    flow_store.set_version(build_version(&serde_json::to_string(&plan).unwrap()));

    let session_repo = Arc::new(TestSessionRepo::default());
    let session = AuthenticationSession::new(Uuid::new_v4(), Uuid::new_v4(), "auth".to_string());
    session_repo.create(&session).await.unwrap();

    let engine = FlowEngine::new(Arc::new(registry), flow_store, session_repo.clone());

    let result = engine
        .execute(session.id, Some(json!({"username": "a"})))
        .await
        .unwrap();

    match result {
        EngineResult::ShowUI { screen_id, context } => {
            assert_eq!(screen_id, "login");
            assert_eq!(context.get("error").unwrap(), "invalid");
        }
        other => panic!("expected ShowUI, got {other:?}"),
    }

    assert_eq!(session_repo.updates.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn execute_handles_continue_then_suspend() {
    let mut registry = RuntimeRegistry::new();
    let node = Arc::new(TestNode {
        execute_outcome: NodeOutcome::SuspendForUI {
            screen: "next".to_string(),
            context: json!({}),
        },
        handle_outcome: Some(NodeOutcome::Continue {
            output: "ok".to_string(),
        }),
    });
    registry.register_node("core.auth.password", node, StepType::Authenticator);

    let mut nodes = HashMap::new();
    nodes.insert(
        "auth".to_string(),
        ExecutionNode {
            id: "auth".to_string(),
            step_type: StepType::Authenticator,
            next: [("ok".to_string(), "next".to_string())]
                .into_iter()
                .collect(),
            config: json!({"auth_type": "core.auth.password"}),
        },
    );
    nodes.insert(
        "next".to_string(),
        ExecutionNode {
            id: "next".to_string(),
            step_type: StepType::Authenticator,
            next: HashMap::new(),
            config: json!({"auth_type": "core.auth.password"}),
        },
    );

    let plan = build_plan("auth", nodes);

    let flow_store = Arc::new(TestFlowStore::default());
    flow_store.set_version(build_version(&serde_json::to_string(&plan).unwrap()));

    let session_repo = Arc::new(TestSessionRepo::default());
    let session = AuthenticationSession::new(Uuid::new_v4(), Uuid::new_v4(), "auth".to_string());
    session_repo.create(&session).await.unwrap();

    let engine = FlowEngine::new(Arc::new(registry), flow_store, session_repo.clone());

    let result = engine
        .execute(session.id, Some(json!({"username": "a"})))
        .await
        .unwrap();

    match result {
        EngineResult::ShowUI { screen_id, .. } => assert_eq!(screen_id, "next"),
        other => panic!("expected ShowUI, got {other:?}"),
    }
}

#[tokio::test]
async fn execute_handles_logic_node_without_worker() {
    let mut nodes = HashMap::new();
    nodes.insert(
        "start".to_string(),
        ExecutionNode {
            id: "start".to_string(),
            step_type: StepType::Logic,
            next: [("default".to_string(), "auth".to_string())]
                .into_iter()
                .collect(),
            config: json!({}),
        },
    );
    nodes.insert(
        "auth".to_string(),
        ExecutionNode {
            id: "auth".to_string(),
            step_type: StepType::Authenticator,
            next: HashMap::new(),
            config: json!({"auth_type": "core.auth.password"}),
        },
    );

    let plan = build_plan("start", nodes);

    let flow_store = Arc::new(TestFlowStore::default());
    flow_store.set_version(build_version(&serde_json::to_string(&plan).unwrap()));

    let session_repo = Arc::new(TestSessionRepo::default());
    let session = AuthenticationSession::new(Uuid::new_v4(), Uuid::new_v4(), "start".to_string());
    session_repo.create(&session).await.unwrap();

    let mut registry_mut = RuntimeRegistry::new();
    let node = Arc::new(TestNode {
        execute_outcome: NodeOutcome::SuspendForUI {
            screen: "login".to_string(),
            context: json!({}),
        },
        handle_outcome: None,
    });
    registry_mut.register_node("core.auth.password", node, StepType::Authenticator);

    let engine = FlowEngine::new(Arc::new(registry_mut), flow_store, session_repo.clone());

    let result = engine.execute(session.id, None).await.unwrap();
    match result {
        EngineResult::ShowUI { screen_id, .. } => assert_eq!(screen_id, "login"),
        other => panic!("expected ShowUI, got {other:?}"),
    }
}

#[tokio::test]
async fn execute_errors_when_terminal_has_no_worker() {
    let registry = Arc::new(RuntimeRegistry::new());

    let mut nodes = HashMap::new();
    nodes.insert(
        "terminal".to_string(),
        ExecutionNode {
            id: "terminal".to_string(),
            step_type: StepType::Terminal,
            next: HashMap::new(),
            config: json!({}),
        },
    );

    let plan = build_plan("terminal", nodes);

    let flow_store = Arc::new(TestFlowStore::default());
    flow_store.set_version(build_version(&serde_json::to_string(&plan).unwrap()));

    let session_repo = Arc::new(TestSessionRepo::default());
    let session =
        AuthenticationSession::new(Uuid::new_v4(), Uuid::new_v4(), "terminal".to_string());
    session_repo.create(&session).await.unwrap();

    let engine = FlowEngine::new(registry, flow_store, session_repo);

    let result = engine.execute(session.id, None).await;
    assert!(matches!(result, Err(Error::System(message)) if message.contains("No worker")));
}
