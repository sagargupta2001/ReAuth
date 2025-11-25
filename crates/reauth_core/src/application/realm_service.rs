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
}

pub struct RealmService {
    realm_repo: Arc<dyn RealmRepository>,
}

impl RealmService {
    pub fn new(realm_repo: Arc<dyn RealmRepository>) -> Self {
        Self { realm_repo }
    }

    pub async fn create_realm(&self, payload: CreateRealmPayload) -> Result<Realm> {
        let settings = Settings::new()?;

        if self.realm_repo.find_by_name(&payload.name).await?.is_some() {
            return Err(Error::RealmAlreadyExists);
        }
        let realm = Realm {
            id: Uuid::new_v4(),
            name: payload.name,
            // Default TTLs, can be made configurable
            access_token_ttl_secs: settings.auth.access_token_ttl_secs,
            refresh_token_ttl_secs: settings.auth.refresh_token_ttl_secs,
        };
        self.realm_repo.create(&realm).await?;
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

        self.realm_repo.update(&realm).await?;
        Ok(realm)
    }
}
