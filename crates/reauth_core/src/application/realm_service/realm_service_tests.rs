use super::{CreateRealmPayload, RealmService, UpdateRealmPayload};
use crate::application::flow_service::FlowService;
use crate::domain::auth_flow::AuthFlow;
use crate::domain::realm::Realm;
use crate::error::{Error, Result};
use crate::ports::flow_repository::FlowRepository;
use crate::ports::realm_repository::RealmRepository;
use crate::ports::transaction_manager::{Transaction, TransactionManager};
use async_trait::async_trait;
use std::any::Any;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

struct TestFlowRepo;

#[async_trait]
impl FlowRepository for TestFlowRepo {
    async fn find_flow_by_name(&self, _realm_id: &Uuid, _name: &str) -> Result<Option<AuthFlow>> {
        Ok(None)
    }

    async fn find_flow_by_id(&self, _flow_id: &Uuid) -> Result<Option<AuthFlow>> {
        Ok(None)
    }

    async fn create_flow<'a>(
        &self,
        _flow: &AuthFlow,
        _tx: Option<&'a mut dyn Transaction>,
    ) -> Result<()> {
        Ok(())
    }

    async fn list_flows_by_realm(&self, _realm_id: &Uuid) -> Result<Vec<AuthFlow>> {
        Ok(Vec::new())
    }
}

#[derive(Default)]
struct TestRealmRepo {
    realm: Mutex<Option<Realm>>,
    update_calls: Mutex<Vec<(Realm, bool)>>,
    find_by_name: Mutex<Option<Realm>>,
    list_all: Mutex<Vec<Realm>>,
}

impl TestRealmRepo {
    fn set_realm(&self, realm: Realm) {
        *self.realm.lock().unwrap() = Some(realm);
    }

    fn set_find_by_name(&self, realm: Option<Realm>) {
        *self.find_by_name.lock().unwrap() = realm;
    }

    fn set_list_all(&self, realms: Vec<Realm>) {
        *self.list_all.lock().unwrap() = realms;
    }

    fn update_calls(&self) -> Vec<(Realm, bool)> {
        self.update_calls.lock().unwrap().clone()
    }
}

#[async_trait]
impl RealmRepository for TestRealmRepo {
    async fn create<'a>(&self, _realm: &Realm, _tx: Option<&'a mut dyn Transaction>) -> Result<()> {
        Ok(())
    }

    async fn find_by_id(&self, _id: &Uuid) -> Result<Option<Realm>> {
        Ok(self.realm.lock().unwrap().clone())
    }

    async fn find_by_name(&self, _name: &str) -> Result<Option<Realm>> {
        Ok(self.find_by_name.lock().unwrap().clone())
    }

    async fn list_all(&self) -> Result<Vec<Realm>> {
        Ok(self.list_all.lock().unwrap().clone())
    }

    async fn update<'a>(&self, realm: &Realm, tx: Option<&'a mut dyn Transaction>) -> Result<()> {
        self.realm.lock().unwrap().replace(realm.clone());
        self.update_calls
            .lock()
            .unwrap()
            .push((realm.clone(), tx.is_some()));
        Ok(())
    }

    async fn list_flows_by_realm(&self, _realm_id: &Uuid) -> Result<Vec<AuthFlow>> {
        Ok(Vec::new())
    }

    async fn update_flow_binding<'a>(
        &self,
        _realm_id: &Uuid,
        _slot: &str,
        _flow_id: &Uuid,
        _tx: Option<&'a mut dyn Transaction>,
    ) -> Result<()> {
        Ok(())
    }
}

struct TestTx;

impl Transaction for TestTx {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

#[derive(Default)]
struct TestTxManager {
    begin_calls: Mutex<usize>,
    commit_calls: Mutex<usize>,
    rollback_calls: Mutex<usize>,
}

#[async_trait]
impl TransactionManager for TestTxManager {
    async fn begin(&self) -> Result<Box<dyn Transaction>> {
        *self.begin_calls.lock().unwrap() += 1;
        Ok(Box::new(TestTx))
    }

    async fn commit(&self, _tx: Box<dyn Transaction>) -> Result<()> {
        *self.commit_calls.lock().unwrap() += 1;
        Ok(())
    }

    async fn rollback(&self, _tx: Box<dyn Transaction>) -> Result<()> {
        *self.rollback_calls.lock().unwrap() += 1;
        Ok(())
    }
}

fn build_service(realm_repo: Arc<TestRealmRepo>) -> RealmService {
    let flow_repo = Arc::new(TestFlowRepo);
    let flow_service = Arc::new(FlowService::new(flow_repo));
    let tx_manager = Arc::new(TestTxManager::default());
    RealmService::new(realm_repo, flow_service, tx_manager)
}

fn base_realm() -> Realm {
    Realm {
        id: Uuid::new_v4(),
        name: "example".to_string(),
        access_token_ttl_secs: 300,
        refresh_token_ttl_secs: 900,
        browser_flow_id: None,
        registration_flow_id: None,
        direct_grant_flow_id: None,
        reset_credentials_flow_id: None,
    }
}

#[tokio::test]
async fn update_realm_errors_when_missing() {
    let realm_repo = Arc::new(TestRealmRepo::default());
    let service = build_service(realm_repo);

    let err = service
        .update_realm(
            Uuid::new_v4(),
            UpdateRealmPayload {
                name: Some("new".to_string()),
                access_token_ttl_secs: None,
                refresh_token_ttl_secs: None,
                browser_flow_id: None,
                registration_flow_id: None,
                direct_grant_flow_id: None,
                reset_credentials_flow_id: None,
            },
        )
        .await
        .expect_err("expected error");

    match err {
        Error::RealmNotFound(_) => {}
        other => panic!("unexpected error: {:?}", other),
    }
}

#[tokio::test]
async fn update_realm_updates_selected_fields() {
    let realm_repo = Arc::new(TestRealmRepo::default());
    let mut realm = base_realm();
    realm.browser_flow_id = Some(Uuid::new_v4().to_string());
    realm_repo.set_realm(realm.clone());

    let service = build_service(realm_repo.clone());
    let new_browser = Uuid::new_v4();
    let result = service
        .update_realm(
            realm.id,
            UpdateRealmPayload {
                name: Some("updated".to_string()),
                access_token_ttl_secs: Some(111),
                refresh_token_ttl_secs: Some(222),
                browser_flow_id: Some(Some(new_browser)),
                registration_flow_id: Some(None),
                direct_grant_flow_id: None,
                reset_credentials_flow_id: None,
            },
        )
        .await
        .expect("update failed");

    assert_eq!(result.name, "updated");
    assert_eq!(result.access_token_ttl_secs, 111);
    assert_eq!(result.refresh_token_ttl_secs, 222);
    assert_eq!(result.browser_flow_id, Some(new_browser.to_string()));
    assert_eq!(result.registration_flow_id, None);

    let updates = realm_repo.update_calls();
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].0.name, "updated");
}

#[tokio::test]
async fn update_realm_with_tx_forwards_transaction() {
    let realm_repo = Arc::new(TestRealmRepo::default());
    let realm = base_realm();
    let realm_id = realm.id;
    realm_repo.set_realm(realm);

    let service = build_service(realm_repo.clone());
    let mut tx = TestTx;

    service
        .update_realm_with_tx(
            realm_id,
            UpdateRealmPayload {
                name: Some("with-tx".to_string()),
                access_token_ttl_secs: None,
                refresh_token_ttl_secs: None,
                browser_flow_id: None,
                registration_flow_id: None,
                direct_grant_flow_id: None,
                reset_credentials_flow_id: None,
            },
            Some(&mut tx),
        )
        .await
        .expect("update failed");

    let updates = realm_repo.update_calls();
    assert_eq!(updates.len(), 1);
    assert!(updates[0].1);
}

#[tokio::test]
async fn passthrough_read_methods_delegate_to_repo() {
    let realm_repo = Arc::new(TestRealmRepo::default());
    let realm = base_realm();
    let realm_id = realm.id;
    realm_repo.set_realm(realm.clone());
    realm_repo.set_find_by_name(Some(realm.clone()));
    realm_repo.set_list_all(vec![realm.clone()]);

    let service = build_service(realm_repo);

    let by_id = service.find_by_id(realm_id).await.unwrap();
    assert_eq!(by_id.unwrap().id, realm_id);

    let by_name = service.find_by_name(&realm.name).await.unwrap();
    assert_eq!(by_name.unwrap().id, realm_id);

    let list = service.list_realms().await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, realm_id);
}

#[tokio::test]
async fn create_realm_rejects_duplicate_name() {
    let realm_repo = Arc::new(TestRealmRepo::default());
    realm_repo.set_find_by_name(Some(base_realm()));
    let service = build_service(realm_repo);

    let err = service
        .create_realm(CreateRealmPayload {
            name: "existing".to_string(),
        })
        .await
        .expect_err("expected error");

    match err {
        Error::RealmAlreadyExists => {}
        other => panic!("unexpected error: {:?}", other),
    }
}
