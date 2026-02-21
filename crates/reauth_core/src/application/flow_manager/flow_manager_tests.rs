use super::*;
use crate::domain::auth_flow::AuthFlow;
use crate::domain::flow::models::{FlowDeployment, FlowDraft, FlowVersion};
use crate::domain::pagination::{PageRequest, PageResponse};
use crate::error::Error;
use crate::ports::flow_store::FlowStore;
use crate::ports::realm_repository::RealmRepository;
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Mutex;

struct TestFlowStore {
    drafts: Mutex<HashMap<Uuid, FlowDraft>>,
    list_drafts_result: Mutex<PageResponse<FlowDraft>>,
    list_versions_result: Mutex<PageResponse<FlowVersion>>,
    list_all_drafts_result: Mutex<Vec<FlowDraft>>,
    create_draft_calls: Mutex<Vec<FlowDraft>>,
    update_draft_calls: Mutex<Vec<FlowDraft>>,
    delete_draft_calls: Mutex<Vec<Uuid>>,
    create_version_calls: Mutex<Vec<FlowVersion>>,
    set_deployment_calls: Mutex<Vec<FlowDeployment>>,
    latest_version_number: Mutex<Option<i32>>,
    latest_version: Mutex<Option<FlowVersion>>,
    versions_by_number: Mutex<HashMap<(Uuid, i32), FlowVersion>>,
    deployed_versions: Mutex<HashMap<(Uuid, String, Uuid), i32>>,
}

impl Default for TestFlowStore {
    fn default() -> Self {
        Self {
            drafts: Mutex::new(HashMap::new()),
            list_drafts_result: Mutex::new(PageResponse::new(Vec::new(), 0, 1, 10)),
            list_versions_result: Mutex::new(PageResponse::new(Vec::new(), 0, 1, 10)),
            list_all_drafts_result: Mutex::new(Vec::new()),
            create_draft_calls: Mutex::new(Vec::new()),
            update_draft_calls: Mutex::new(Vec::new()),
            delete_draft_calls: Mutex::new(Vec::new()),
            create_version_calls: Mutex::new(Vec::new()),
            set_deployment_calls: Mutex::new(Vec::new()),
            latest_version_number: Mutex::new(None),
            latest_version: Mutex::new(None),
            versions_by_number: Mutex::new(HashMap::new()),
            deployed_versions: Mutex::new(HashMap::new()),
        }
    }
}

impl TestFlowStore {
    fn insert_draft(&self, draft: FlowDraft) {
        self.drafts.lock().unwrap().insert(draft.id, draft);
    }

    fn set_list_drafts_result(&self, result: PageResponse<FlowDraft>) {
        *self.list_drafts_result.lock().unwrap() = result;
    }

    fn set_list_versions_result(&self, result: PageResponse<FlowVersion>) {
        *self.list_versions_result.lock().unwrap() = result;
    }

    fn set_latest_version_number(&self, value: Option<i32>) {
        *self.latest_version_number.lock().unwrap() = value;
    }

    fn set_latest_version(&self, value: Option<FlowVersion>) {
        *self.latest_version.lock().unwrap() = value;
    }

    fn set_version_by_number(&self, flow_id: Uuid, version_number: i32, version: FlowVersion) {
        self.versions_by_number
            .lock()
            .unwrap()
            .insert((flow_id, version_number), version);
    }

    fn set_deployed_version(
        &self,
        realm_id: Uuid,
        flow_type: &str,
        flow_id: Uuid,
        version_number: i32,
    ) {
        self.deployed_versions
            .lock()
            .unwrap()
            .insert((realm_id, flow_type.to_string(), flow_id), version_number);
    }
}

#[async_trait]
impl FlowStore for TestFlowStore {
    async fn create_draft(&self, draft: &FlowDraft) -> Result<()> {
        self.drafts.lock().unwrap().insert(draft.id, draft.clone());
        self.create_draft_calls.lock().unwrap().push(draft.clone());
        Ok(())
    }

    async fn update_draft(&self, draft: &FlowDraft) -> Result<()> {
        self.drafts.lock().unwrap().insert(draft.id, draft.clone());
        self.update_draft_calls.lock().unwrap().push(draft.clone());
        Ok(())
    }

    async fn get_draft_by_id(&self, id: &Uuid) -> Result<Option<FlowDraft>> {
        Ok(self.drafts.lock().unwrap().get(id).cloned())
    }

    async fn list_drafts(
        &self,
        _realm_id: &Uuid,
        _req: &PageRequest,
    ) -> Result<PageResponse<FlowDraft>> {
        let page = self.list_drafts_result.lock().unwrap();
        Ok(PageResponse::new(
            page.data.clone(),
            page.meta.total,
            page.meta.page,
            page.meta.per_page,
        ))
    }

    async fn list_all_drafts(&self, _realm_id: &Uuid) -> Result<Vec<FlowDraft>> {
        let explicit = self.list_all_drafts_result.lock().unwrap();
        if explicit.is_empty() {
            Ok(self.drafts.lock().unwrap().values().cloned().collect())
        } else {
            Ok(explicit.clone())
        }
    }

    async fn delete_draft(&self, id: &Uuid) -> Result<()> {
        self.drafts.lock().unwrap().remove(id);
        self.delete_draft_calls.lock().unwrap().push(*id);
        Ok(())
    }

    async fn create_version(&self, _version: &FlowVersion) -> Result<()> {
        Ok(())
    }

    async fn get_version(&self, _id: &Uuid) -> Result<Option<FlowVersion>> {
        Ok(None)
    }

    async fn list_versions(
        &self,
        _flow_id: &Uuid,
        _req: &PageRequest,
    ) -> Result<PageResponse<FlowVersion>> {
        let page = self.list_versions_result.lock().unwrap();
        Ok(PageResponse::new(
            page.data.clone(),
            page.meta.total,
            page.meta.page,
            page.meta.per_page,
        ))
    }

    async fn set_deployment(&self, deployment: &FlowDeployment) -> Result<()> {
        self.set_deployment_calls
            .lock()
            .unwrap()
            .push(deployment.clone());
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
        Ok(*self.latest_version_number.lock().unwrap())
    }

    async fn get_latest_version(&self, _flow_id: &Uuid) -> Result<Option<FlowVersion>> {
        Ok(self.latest_version.lock().unwrap().clone())
    }

    async fn get_deployed_version_number(
        &self,
        realm_id: &Uuid,
        flow_type: &str,
        flow_id: &Uuid,
    ) -> Result<Option<i32>> {
        Ok(self
            .deployed_versions
            .lock()
            .unwrap()
            .get(&(*realm_id, flow_type.to_string(), *flow_id))
            .cloned())
    }

    async fn get_version_by_number(
        &self,
        flow_id: &Uuid,
        version_number: i32,
    ) -> Result<Option<FlowVersion>> {
        Ok(self
            .versions_by_number
            .lock()
            .unwrap()
            .get(&(*flow_id, version_number))
            .cloned())
    }

    async fn get_active_version(&self, _flow_id: &Uuid) -> Result<Option<FlowVersion>> {
        Ok(None)
    }

    async fn create_version_with_tx(
        &self,
        version: &FlowVersion,
        _tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        self.create_version_calls
            .lock()
            .unwrap()
            .push(version.clone());
        Ok(())
    }

    async fn set_deployment_with_tx(
        &self,
        deployment: &FlowDeployment,
        _tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        self.set_deployment_calls
            .lock()
            .unwrap()
            .push(deployment.clone());
        Ok(())
    }

    async fn delete_draft_with_tx(
        &self,
        id: &Uuid,
        _tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        self.drafts.lock().unwrap().remove(id);
        self.delete_draft_calls.lock().unwrap().push(*id);
        Ok(())
    }
}

#[derive(Default)]
struct TestFlowRepo {
    flows: Mutex<HashMap<Uuid, AuthFlow>>,
    create_calls: Mutex<Vec<AuthFlow>>,
}

impl TestFlowRepo {
    fn insert_flow(&self, flow: AuthFlow) {
        self.flows.lock().unwrap().insert(flow.id, flow);
    }

    fn create_calls(&self) -> Vec<AuthFlow> {
        self.create_calls.lock().unwrap().clone()
    }
}

#[async_trait]
impl FlowRepository for TestFlowRepo {
    async fn find_flow_by_name(&self, realm_id: &Uuid, name: &str) -> Result<Option<AuthFlow>> {
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
        _tx: Option<&'a mut dyn Transaction>,
    ) -> Result<()> {
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
struct TestRealmRepo {
    update_calls: Mutex<Vec<(Uuid, String, Uuid)>>,
}

impl TestRealmRepo {
    fn update_calls(&self) -> Vec<(Uuid, String, Uuid)> {
        self.update_calls.lock().unwrap().clone()
    }
}

#[async_trait]
impl RealmRepository for TestRealmRepo {
    async fn create<'a>(
        &self,
        _realm: &crate::domain::realm::Realm,
        _tx: Option<&'a mut dyn Transaction>,
    ) -> Result<()> {
        Ok(())
    }

    async fn find_by_id(&self, _id: &Uuid) -> Result<Option<crate::domain::realm::Realm>> {
        Ok(None)
    }

    async fn find_by_name(&self, _name: &str) -> Result<Option<crate::domain::realm::Realm>> {
        Ok(None)
    }

    async fn list_all(&self) -> Result<Vec<crate::domain::realm::Realm>> {
        Ok(Vec::new())
    }

    async fn update<'a>(
        &self,
        _realm: &crate::domain::realm::Realm,
        _tx: Option<&'a mut dyn Transaction>,
    ) -> Result<()> {
        Ok(())
    }

    async fn list_flows_by_realm(&self, _realm_id: &Uuid) -> Result<Vec<AuthFlow>> {
        Ok(Vec::new())
    }

    async fn update_flow_binding<'a>(
        &self,
        realm_id: &Uuid,
        slot: &str,
        flow_id: &Uuid,
        _tx: Option<&'a mut dyn Transaction>,
    ) -> Result<()> {
        self.update_calls
            .lock()
            .unwrap()
            .push((*realm_id, slot.to_string(), *flow_id));
        Ok(())
    }
}

#[allow(dead_code)]
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

fn registry_for_publish() -> RuntimeRegistry {
    let mut registry = RuntimeRegistry::new();
    registry.register_definition("core.start", crate::domain::execution::StepType::Logic);
    registry.register_definition(
        "core.terminal.allow",
        crate::domain::execution::StepType::Terminal,
    );
    registry
}

fn sample_graph_json() -> String {
    json!({
        "nodes": [
            {"id": "start", "type": "core.start"},
            {"id": "end", "type": "core.terminal.allow"}
        ],
        "edges": [
            {"source": "start", "target": "end"}
        ]
    })
    .to_string()
}

fn build_draft(realm_id: Uuid, flow_id: Uuid, flow_type: &str, graph: String) -> FlowDraft {
    FlowDraft {
        id: flow_id,
        realm_id,
        name: "Draft".to_string(),
        description: Some("desc".to_string()),
        graph_json: graph,
        flow_type: flow_type.to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn build_flow_meta(realm_id: Uuid, flow_id: Uuid, flow_type: &str, built_in: bool) -> AuthFlow {
    AuthFlow {
        id: flow_id,
        realm_id,
        name: "Flow".to_string(),
        alias: "Flow".to_string(),
        description: Some("desc".to_string()),
        r#type: flow_type.to_string(),
        built_in,
    }
}

fn build_version(flow_id: Uuid, version_number: i32, graph_json: String) -> FlowVersion {
    FlowVersion {
        id: Uuid::new_v4().to_string(),
        flow_id: flow_id.to_string(),
        version_number,
        execution_artifact: "artifact".to_string(),
        graph_json,
        checksum: "checksum".to_string(),
        created_at: Utc::now(),
    }
}

fn build_manager(
    flow_store: Arc<TestFlowStore>,
    flow_repo: Arc<TestFlowRepo>,
    realm_repo: Arc<TestRealmRepo>,
    registry: RuntimeRegistry,
) -> FlowManager {
    FlowManager::new(flow_store, flow_repo, realm_repo, Arc::new(registry))
}

#[tokio::test]
async fn list_drafts_returns_page_from_store() {
    let flow_store = Arc::new(TestFlowStore::default());
    let draft = build_draft(
        Uuid::new_v4(),
        Uuid::new_v4(),
        "browser",
        sample_graph_json(),
    );
    flow_store.set_list_drafts_result(PageResponse::new(vec![draft.clone()], 1, 1, 20));

    let manager = build_manager(
        flow_store.clone(),
        Arc::new(TestFlowRepo::default()),
        Arc::new(TestRealmRepo::default()),
        RuntimeRegistry::new(),
    );

    let page = manager
        .list_drafts(draft.realm_id, PageRequest::default())
        .await
        .unwrap();

    assert_eq!(page.data.len(), 1);
    assert_eq!(page.data[0].id, draft.id);
}

#[tokio::test]
async fn create_draft_persists_defaults() {
    let flow_store = Arc::new(TestFlowStore::default());
    let manager = build_manager(
        flow_store.clone(),
        Arc::new(TestFlowRepo::default()),
        Arc::new(TestRealmRepo::default()),
        RuntimeRegistry::new(),
    );

    let realm_id = Uuid::new_v4();
    let created = manager
        .create_draft(
            realm_id,
            CreateDraftRequest {
                name: "Flow".to_string(),
                description: None,
                flow_type: "browser".to_string(),
            },
        )
        .await
        .unwrap();

    assert_eq!(created.realm_id, realm_id);
    assert_eq!(created.graph_json, "{}");
    let stored = flow_store.drafts.lock().unwrap().get(&created.id).cloned();
    assert!(stored.is_some());
}

#[tokio::test]
async fn get_draft_returns_existing_draft() {
    let flow_store = Arc::new(TestFlowStore::default());
    let flow_repo = Arc::new(TestFlowRepo::default());
    let manager = build_manager(
        flow_store.clone(),
        flow_repo,
        Arc::new(TestRealmRepo::default()),
        RuntimeRegistry::new(),
    );

    let draft = build_draft(
        Uuid::new_v4(),
        Uuid::new_v4(),
        "browser",
        sample_graph_json(),
    );
    flow_store.insert_draft(draft.clone());

    let fetched = manager.get_draft(draft.id).await.unwrap();
    assert_eq!(fetched.id, draft.id);
}

#[tokio::test]
async fn get_draft_restores_from_latest_version() {
    let flow_store = Arc::new(TestFlowStore::default());
    let flow_repo = Arc::new(TestFlowRepo::default());
    let manager = build_manager(
        flow_store.clone(),
        flow_repo.clone(),
        Arc::new(TestRealmRepo::default()),
        RuntimeRegistry::new(),
    );

    let flow_id = Uuid::new_v4();
    let realm_id = Uuid::new_v4();
    let version = build_version(flow_id, 2, "{\"nodes\":[],\"edges\":[]}".to_string());
    flow_store.set_latest_version(Some(version.clone()));

    let meta = build_flow_meta(realm_id, flow_id, "browser", true);
    flow_repo.insert_flow(meta.clone());

    let draft = manager.get_draft(flow_id).await.unwrap();
    assert_eq!(draft.graph_json, version.graph_json);
    assert_eq!(draft.realm_id, meta.realm_id);
    assert!(flow_store.drafts.lock().unwrap().contains_key(&flow_id));
}

#[tokio::test]
async fn get_draft_falls_back_to_default_graph() {
    let flow_store = Arc::new(TestFlowStore::default());
    let flow_repo = Arc::new(TestFlowRepo::default());
    let manager = build_manager(
        flow_store.clone(),
        flow_repo.clone(),
        Arc::new(TestRealmRepo::default()),
        RuntimeRegistry::new(),
    );

    let flow_id = Uuid::new_v4();
    let realm_id = Uuid::new_v4();
    let meta = build_flow_meta(realm_id, flow_id, "browser", false);
    flow_repo.insert_flow(meta.clone());

    let draft = manager.get_draft(flow_id).await.unwrap();
    assert_eq!(
        draft.graph_json,
        FlowManager::generate_default_graph("browser")
    );
    assert!(flow_store.drafts.lock().unwrap().contains_key(&flow_id));
}

#[tokio::test]
async fn get_draft_errors_when_flow_missing() {
    let manager = build_manager(
        Arc::new(TestFlowStore::default()),
        Arc::new(TestFlowRepo::default()),
        Arc::new(TestRealmRepo::default()),
        RuntimeRegistry::new(),
    );

    let err = manager.get_draft(Uuid::new_v4()).await.unwrap_err();
    assert!(matches!(err, Error::FlowNotFound(_)));
}

#[tokio::test]
async fn update_draft_updates_fields() {
    let flow_store = Arc::new(TestFlowStore::default());
    let manager = build_manager(
        flow_store.clone(),
        Arc::new(TestFlowRepo::default()),
        Arc::new(TestRealmRepo::default()),
        RuntimeRegistry::new(),
    );

    let draft = build_draft(
        Uuid::new_v4(),
        Uuid::new_v4(),
        "browser",
        sample_graph_json(),
    );
    flow_store.insert_draft(draft.clone());

    let updated = manager
        .update_draft(
            draft.id,
            UpdateDraftRequest {
                name: Some("Updated".to_string()),
                description: Some("New".to_string()),
                graph_json: Some(json!({"nodes":[],"edges":[]})),
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.name, "Updated");
    assert_eq!(updated.description.as_deref(), Some("New"));
    assert!(updated.graph_json.contains("nodes"));
}

#[tokio::test]
async fn publish_flow_creates_version_and_deployment_and_binding() {
    let flow_store = Arc::new(TestFlowStore::default());
    let flow_repo = Arc::new(TestFlowRepo::default());
    let realm_repo = Arc::new(TestRealmRepo::default());
    let registry = registry_for_publish();

    let realm_id = Uuid::new_v4();
    let flow_id = Uuid::new_v4();
    flow_store.insert_draft(build_draft(
        realm_id,
        flow_id,
        "browser",
        sample_graph_json(),
    ));
    flow_store.set_latest_version_number(Some(2));

    let manager = build_manager(
        flow_store.clone(),
        flow_repo.clone(),
        realm_repo.clone(),
        registry,
    );

    let version = manager.publish_flow(realm_id, flow_id).await.unwrap();

    assert_eq!(version.version_number, 3);
    assert_eq!(flow_repo.create_calls().len(), 1);
    assert_eq!(flow_store.create_version_calls.lock().unwrap().len(), 1);
    assert_eq!(flow_store.set_deployment_calls.lock().unwrap().len(), 1);
    assert!(flow_store
        .delete_draft_calls
        .lock()
        .unwrap()
        .contains(&flow_id));

    let binding_calls = realm_repo.update_calls();
    assert_eq!(binding_calls.len(), 1);
    assert_eq!(binding_calls[0].1, "browser_flow_id");
}

#[tokio::test]
async fn publish_flow_rejects_corrupt_draft_json() {
    let flow_store = Arc::new(TestFlowStore::default());
    let flow_repo = Arc::new(TestFlowRepo::default());
    let realm_repo = Arc::new(TestRealmRepo::default());

    let realm_id = Uuid::new_v4();
    let flow_id = Uuid::new_v4();
    flow_store.insert_draft(build_draft(
        realm_id,
        flow_id,
        "browser",
        "not-json".to_string(),
    ));

    let manager = build_manager(flow_store, flow_repo, realm_repo, registry_for_publish());

    let err = manager.publish_flow(realm_id, flow_id).await.unwrap_err();
    assert!(
        matches!(err, Error::Validation(message) if message.contains("Draft JSON is corrupted"))
    );
}

#[tokio::test]
async fn publish_flow_propagates_compile_errors() {
    let flow_store = Arc::new(TestFlowStore::default());
    let flow_repo = Arc::new(TestFlowRepo::default());
    let realm_repo = Arc::new(TestRealmRepo::default());

    let realm_id = Uuid::new_v4();
    let flow_id = Uuid::new_v4();
    let graph = json!({
        "nodes": [{"id": "start", "type": "core.unknown"}],
        "edges": []
    })
    .to_string();
    flow_store.insert_draft(build_draft(realm_id, flow_id, "browser", graph));

    let manager = build_manager(flow_store, flow_repo, realm_repo, registry_for_publish());

    let err = manager.publish_flow(realm_id, flow_id).await.unwrap_err();
    assert!(matches!(err, Error::Validation(message) if message.contains("Unknown node type")));
}

#[tokio::test]
async fn get_deployed_version_returns_store_value() {
    let flow_store = Arc::new(TestFlowStore::default());
    let realm_id = Uuid::new_v4();
    let flow_id = Uuid::new_v4();
    flow_store.set_deployed_version(realm_id, "browser", flow_id, 7);

    let manager = build_manager(
        flow_store,
        Arc::new(TestFlowRepo::default()),
        Arc::new(TestRealmRepo::default()),
        RuntimeRegistry::new(),
    );

    let version = manager
        .get_deployed_version(&realm_id, "browser", &flow_id)
        .await
        .unwrap();

    assert_eq!(version, Some(7));
}

#[tokio::test]
async fn is_flow_built_in_reads_metadata() {
    let flow_store = Arc::new(TestFlowStore::default());
    let flow_repo = Arc::new(TestFlowRepo::default());
    let flow_id = Uuid::new_v4();
    flow_repo.insert_flow(build_flow_meta(Uuid::new_v4(), flow_id, "browser", true));

    let manager = build_manager(
        flow_store,
        flow_repo,
        Arc::new(TestRealmRepo::default()),
        RuntimeRegistry::new(),
    );

    let result = manager.is_flow_built_in(&flow_id).await.unwrap();
    assert!(result);
}

#[tokio::test]
async fn list_flow_versions_returns_page_from_store() {
    let flow_store = Arc::new(TestFlowStore::default());
    let flow_id = Uuid::new_v4();
    let version = build_version(flow_id, 1, "{}".to_string());
    flow_store.set_list_versions_result(PageResponse::new(vec![version.clone()], 1, 1, 20));

    let manager = build_manager(
        flow_store,
        Arc::new(TestFlowRepo::default()),
        Arc::new(TestRealmRepo::default()),
        RuntimeRegistry::new(),
    );

    let page = manager
        .list_flow_versions(flow_id, PageRequest::default())
        .await
        .unwrap();

    assert_eq!(page.data.len(), 1);
    assert_eq!(page.data[0].id, version.id);
}

#[tokio::test]
async fn rollback_flow_updates_deployment() {
    let flow_store = Arc::new(TestFlowStore::default());
    let flow_repo = Arc::new(TestFlowRepo::default());
    let realm_id = Uuid::new_v4();
    let flow_id = Uuid::new_v4();

    let version = build_version(flow_id, 2, "{}".to_string());
    flow_store.set_version_by_number(flow_id, 2, version.clone());
    flow_repo.insert_flow(build_flow_meta(realm_id, flow_id, "browser", false));

    let manager = build_manager(
        flow_store.clone(),
        flow_repo,
        Arc::new(TestRealmRepo::default()),
        RuntimeRegistry::new(),
    );

    manager.rollback_flow(realm_id, flow_id, 2).await.unwrap();

    let deployments = flow_store.set_deployment_calls.lock().unwrap();
    assert_eq!(deployments.len(), 1);
    assert_eq!(deployments[0].active_version_id, version.id);
}

#[tokio::test]
async fn rollback_flow_errors_when_version_missing() {
    let manager = build_manager(
        Arc::new(TestFlowStore::default()),
        Arc::new(TestFlowRepo::default()),
        Arc::new(TestRealmRepo::default()),
        RuntimeRegistry::new(),
    );

    let err = manager
        .rollback_flow(Uuid::new_v4(), Uuid::new_v4(), 99)
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Unexpected(_)));
}

#[tokio::test]
async fn rollback_flow_errors_when_flow_missing() {
    let flow_store = Arc::new(TestFlowStore::default());
    let flow_id = Uuid::new_v4();
    flow_store.set_version_by_number(flow_id, 1, build_version(flow_id, 1, "{}".to_string()));

    let manager = build_manager(
        flow_store,
        Arc::new(TestFlowRepo::default()),
        Arc::new(TestRealmRepo::default()),
        RuntimeRegistry::new(),
    );

    let err = manager
        .rollback_flow(Uuid::new_v4(), flow_id, 1)
        .await
        .unwrap_err();
    assert!(matches!(err, Error::FlowNotFound(_)));
}

#[tokio::test]
async fn restore_draft_from_version_updates_graph() {
    let flow_store = Arc::new(TestFlowStore::default());
    let flow_repo = Arc::new(TestFlowRepo::default());
    let realm_id = Uuid::new_v4();
    let flow_id = Uuid::new_v4();

    let version = build_version(flow_id, 1, "{\"nodes\":[],\"edges\":[]}".to_string());
    flow_store.set_version_by_number(flow_id, 1, version.clone());

    let draft = build_draft(realm_id, flow_id, "browser", sample_graph_json());
    flow_store.insert_draft(draft.clone());

    let manager = build_manager(
        flow_store.clone(),
        flow_repo,
        Arc::new(TestRealmRepo::default()),
        RuntimeRegistry::new(),
    );

    manager
        .restore_draft_from_version(realm_id, flow_id, 1)
        .await
        .unwrap();

    let updated = flow_store.update_draft_calls.lock().unwrap();
    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0].graph_json, version.graph_json);
}

#[tokio::test]
async fn restore_draft_from_version_errors_when_missing() {
    let manager = build_manager(
        Arc::new(TestFlowStore::default()),
        Arc::new(TestFlowRepo::default()),
        Arc::new(TestRealmRepo::default()),
        RuntimeRegistry::new(),
    );

    let err = manager
        .restore_draft_from_version(Uuid::new_v4(), Uuid::new_v4(), 2)
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Unexpected(_)));
}
