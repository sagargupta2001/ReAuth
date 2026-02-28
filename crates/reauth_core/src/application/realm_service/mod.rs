use crate::application::flow_service::FlowService;
use crate::config::Settings;
use crate::ports::transaction_manager::{Transaction, TransactionManager};
use crate::{
    domain::realm::Realm,
    error::{Error, Result},
    ports::realm_repository::RealmRepository,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

const MAX_LOCKOUT_THRESHOLD: i64 = 50;
const MAX_LOCKOUT_DURATION_SECS: i64 = 86_400;

#[derive(Deserialize)]
pub struct CreateRealmPayload {
    pub name: String,
}

#[derive(Deserialize)]
pub struct UpdateRealmPayload {
    pub name: Option<String>,
    pub access_token_ttl_secs: Option<i64>,
    pub refresh_token_ttl_secs: Option<i64>,
    pub pkce_required_public_clients: Option<bool>,
    pub lockout_threshold: Option<i64>,
    pub lockout_duration_secs: Option<i64>,
    pub browser_flow_id: Option<Option<Uuid>>,
    pub registration_flow_id: Option<Option<Uuid>>,
    pub direct_grant_flow_id: Option<Option<Uuid>>,
    pub reset_credentials_flow_id: Option<Option<Uuid>>,
}

pub struct RealmService {
    realm_repo: Arc<dyn RealmRepository>,
    flow_service: Arc<FlowService>,
    tx_manager: Arc<dyn TransactionManager>,
}

impl RealmService {
    pub fn new(
        realm_repo: Arc<dyn RealmRepository>,
        flow_service: Arc<FlowService>,
        tx_manager: Arc<dyn TransactionManager>,
    ) -> Self {
        Self {
            realm_repo,
            flow_service,
            tx_manager,
        }
    }

    pub async fn create_realm(&self, payload: CreateRealmPayload) -> Result<Realm> {
        let settings = Settings::new()?;

        // 1. Check existence (Read can be done via Pool, no TX needed yet)
        if self.realm_repo.find_by_name(&payload.name).await?.is_some() {
            return Err(Error::RealmAlreadyExists);
        }

        // 2. Start Transaction
        let mut tx = self.tx_manager.begin().await?;

        // 3. Define the atomic operation block
        let result = async {
            let realm_id = Uuid::new_v4();

            let mut realm = Realm {
                id: realm_id,
                name: payload.name,
                access_token_ttl_secs: settings.auth.access_token_ttl_secs,
                refresh_token_ttl_secs: settings.auth.refresh_token_ttl_secs,
                pkce_required_public_clients: settings.auth.pkce_required_public_clients,
                lockout_threshold: settings.auth.lockout_threshold,
                lockout_duration_secs: settings.auth.lockout_duration_secs,
                browser_flow_id: None,
                registration_flow_id: None,
                direct_grant_flow_id: None,
                reset_credentials_flow_id: None,
            };

            // A. Create Realm (Pass TX)
            self.realm_repo.create(&realm, Some(&mut *tx)).await?;

            // B. Create Flows (Pass TX)
            let default_flows = self
                .flow_service
                .setup_default_flows_for_realm(realm_id, Some(&mut *tx))
                .await?;

            // C. Link Flows to Realm
            realm.browser_flow_id = Some(default_flows.browser_flow_id.to_string());
            realm.registration_flow_id = Some(default_flows.registration_flow_id.to_string());
            realm.direct_grant_flow_id = Some(default_flows.direct_grant_flow_id.to_string());
            realm.reset_credentials_flow_id =
                Some(default_flows.reset_credentials_flow_id.to_string());

            // D. Update Realm (Pass TX)
            self.realm_repo.update(&realm, Some(&mut *tx)).await?;

            Ok(realm)
        }
        .await;

        // 4. Commit or Rollback based on result
        match result {
            Ok(realm) => {
                self.tx_manager.commit(tx).await?;
                Ok(realm)
            }
            Err(e) => {
                self.tx_manager.rollback(tx).await?;
                Err(e)
            }
        }
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
        self.update_realm_with_tx(id, payload, None).await
    }

    pub async fn update_realm_with_tx(
        &self,
        id: Uuid,
        payload: UpdateRealmPayload,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<Realm> {
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
        if let Some(value) = payload.pkce_required_public_clients {
            realm.pkce_required_public_clients = value;
        }
        if let Some(value) = payload.lockout_threshold {
            validate_lockout_value("lockout_threshold", value, MAX_LOCKOUT_THRESHOLD)?;
            realm.lockout_threshold = value;
        }
        if let Some(value) = payload.lockout_duration_secs {
            validate_lockout_value("lockout_duration_secs", value, MAX_LOCKOUT_DURATION_SECS)?;
            realm.lockout_duration_secs = value;
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

        self.realm_repo.update(&realm, tx).await?;

        Ok(realm)
    }
}

fn validate_lockout_value(field: &str, value: i64, max: i64) -> Result<()> {
    if value < 0 {
        return Err(Error::Validation(format!(
            "{} must be greater than or equal to 0",
            field
        )));
    }
    if value > max {
        return Err(Error::Validation(format!(
            "{} must be less than or equal to {}",
            field, max
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
