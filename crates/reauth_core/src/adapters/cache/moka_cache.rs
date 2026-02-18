//! A high-performance, in-memory cache adapter using `moka`.

use crate::ports::cache_service::CacheService;
use async_trait::async_trait;
use moka::future::Cache; // Use the async-ready cache
use std::{collections::HashSet, time::Duration};
use uuid::Uuid;

/// An adapter that implements the `CacheService` port using `moka`.
#[derive(Clone)]
pub struct MokaCacheService {
    user_permissions: Cache<Uuid, HashSet<String>>,
}

impl MokaCacheService {
    pub fn new() -> Self {
        let user_permissions = Cache::builder()
            // Time to live (TTL): 5 minutes
            .time_to_live(Duration::from_secs(300))
            // Max 10,000 users' permissions in cache
            .max_capacity(10_000)
            .build();

        Self { user_permissions }
    }
}

impl Default for MokaCacheService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CacheService for MokaCacheService {
    async fn get_user_permissions(&self, user_id: &Uuid) -> Option<HashSet<String>> {
        self.user_permissions.get(user_id).await
    }

    async fn set_user_permissions(&self, user_id: &Uuid, permissions: &HashSet<String>) {
        // We clone the permissions here because the cache needs to own its data.
        self.user_permissions
            .insert(*user_id, permissions.clone())
            .await;
    }

    async fn clear_user_permissions(&self, user_id: &Uuid) {
        self.user_permissions.invalidate(user_id).await;
    }
}
