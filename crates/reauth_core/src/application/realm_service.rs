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

pub struct RealmService {
    realm_repo: Arc<dyn RealmRepository>,
}

impl RealmService {
    pub fn new(realm_repo: Arc<dyn RealmRepository>) -> Self {
        Self { realm_repo }
    }

    pub async fn create_realm(&self, payload: CreateRealmPayload) -> Result<Realm> {
        if self.realm_repo.find_by_name(&payload.name).await?.is_some() {
            return Err(Error::RealmAlreadyExists);
        }
        let realm = Realm {
            id: Uuid::new_v4(),
            name: payload.name,
            // Default TTLs, can be made configurable
            access_token_ttl_secs: 900,
            refresh_token_ttl_secs: 604800,
        };
        self.realm_repo.create(&realm).await?;
        Ok(realm)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Realm>> {
        self.realm_repo.find_by_id(&id).await
    }
}
