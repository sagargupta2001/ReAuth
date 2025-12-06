use crate::application::flow_service::FlowService;
use crate::config::Settings;
use crate::{
    domain::realm::Realm,
    error::{Error, Result},
    ports::realm_repository::RealmRepository,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateRealmPayload {
    pub name: String,
}

#[derive(Deserialize)]
pub struct UpdateRealmPayload {
    pub name: Option<String>,
    pub access_token_ttl_secs: Option<i64>,
    pub refresh_token_ttl_secs: Option<i64>,
    pub browser_flow_id: Option<Option<Uuid>>,
    pub registration_flow_id: Option<Option<Uuid>>,
    pub direct_grant_flow_id: Option<Option<Uuid>>,
    pub reset_credentials_flow_id: Option<Option<Uuid>>,
}

pub struct RealmService {
    realm_repo: Arc<dyn RealmRepository>,
    flow_service: Arc<FlowService>,
}

impl RealmService {
    pub fn new(realm_repo: Arc<dyn RealmRepository>, flow_service: Arc<FlowService>) -> Self {
        Self {
            realm_repo,
            flow_service,
        }
    }

    pub async fn create_realm(&self, payload: CreateRealmPayload) -> Result<Realm> {
        let settings = Settings::new()?;

        if self.realm_repo.find_by_name(&payload.name).await?.is_some() {
            return Err(Error::RealmAlreadyExists);
        }

        // Generate ID first
        let realm_id = Uuid::new_v4();

        // Create Default Flows ---
        // We create the flows *before* inserting the realm so we can link them immediately.
        // (Or create realm first, then flows, then update realm - but immediate link is cleaner)
        // Note: FK constraints might require Realm to exist first.
        // Let's assume we create Realm -> Create Flows -> Update Realm (safest)

        let mut realm = Realm {
            id: realm_id,
            name: payload.name,
            access_token_ttl_secs: settings.auth.access_token_ttl_secs,
            refresh_token_ttl_secs: settings.auth.refresh_token_ttl_secs,
            browser_flow_id: None,
            registration_flow_id: None,
            direct_grant_flow_id: None,
            reset_credentials_flow_id: None,
        };

        // Save Realm (so FKs work)
        self.realm_repo.create(&realm).await?;

        // Create Flows
        let default_flows = self
            .flow_service
            .setup_default_flows_for_realm(realm_id)
            .await?;

        // Link Flows to Realm
        realm.browser_flow_id = Some(default_flows.browser_flow_id.to_string());
        realm.registration_flow_id = Some(default_flows.registration_flow_id.to_string());
        realm.direct_grant_flow_id = Some(default_flows.direct_grant_flow_id.to_string());
        realm.reset_credentials_flow_id = Some(default_flows.reset_credentials_flow_id.to_string());

        // Update Realm with links
        self.realm_repo.update(&realm).await?;

        Ok(realm)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Realm>> {
        self.realm_repo.find_by_id(&id).await
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<Realm>> {
        self.realm_repo.find_by_name(name).await
    }

    pub async fn list_realms(&self) -> Result<Vec<Realm>> {
        self.realm_repo.list_all().await
    }

    pub async fn update_realm(&self, id: Uuid, payload: UpdateRealmPayload) -> Result<Realm> {
        let mut realm = self
            .realm_repo
            .find_by_id(&id)
            .await?
            .ok_or(Error::RealmNotFound(id.to_string()))?;

        if let Some(name) = payload.name {
            realm.name = name;
        }
        if let Some(ttl) = payload.access_token_ttl_secs {
            realm.access_token_ttl_secs = ttl;
        }
        if let Some(ttl) = payload.refresh_token_ttl_secs {
            realm.refresh_token_ttl_secs = ttl;
        }

        if let Some(val) = payload.browser_flow_id {
            realm.browser_flow_id = val.map(|id| id.to_string());
        }
        if let Some(val) = payload.registration_flow_id {
            realm.registration_flow_id = val.map(|id| id.to_string());
        }
        if let Some(val) = payload.direct_grant_flow_id {
            realm.direct_grant_flow_id = val.map(|id| id.to_string());
        }
        if let Some(val) = payload.reset_credentials_flow_id {
            realm.reset_credentials_flow_id = val.map(|id| id.to_string());
        }

        self.realm_repo.update(&realm).await?;
        Ok(realm)
    }
}
