//! A high-performance, in-memory cache adapter using `moka`.

use crate::ports::cache_service::CacheService;
use async_trait::async_trait;
use moka::future::Cache; // Use the async-ready cache
use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use uuid::Uuid;

/// An adapter that implements the `CacheService` port using `moka`.
#[derive(Clone)]
pub struct MokaCacheService {
    user_permissions: Cache<Uuid, HashSet<String>>,
    metrics: Arc<CacheMetrics>,
    max_capacity: u64,
}

#[derive(Default)]
struct CacheMetrics {
    hits: AtomicU64,
    misses: AtomicU64,
}

const USER_PERMISSIONS_MAX_CAPACITY: u64 = 10_000;
const USER_PERMISSIONS_NAMESPACE: &str = "user_permissions";

impl MokaCacheService {
    pub fn new() -> Self {
        let user_permissions = Cache::builder()
            // Time to live (TTL): 5 minutes
            .time_to_live(Duration::from_secs(300))
            // Max 10,000 users' permissions in cache
            .max_capacity(USER_PERMISSIONS_MAX_CAPACITY)
            .build();

        Self {
            user_permissions,
            metrics: Arc::new(CacheMetrics::default()),
            max_capacity: USER_PERMISSIONS_MAX_CAPACITY,
        }
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
        let value = self.user_permissions.get(user_id).await;
        if value.is_some() {
            self.metrics.hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.metrics.misses.fetch_add(1, Ordering::Relaxed);
        }
        value
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

    async fn clear_all(&self) {
        self.user_permissions.invalidate_all();
    }

    async fn clear_namespace(&self, namespace: &str) {
        if namespace == USER_PERMISSIONS_NAMESPACE {
            self.user_permissions.invalidate_all();
        }
    }

    async fn stats(&self) -> crate::ports::cache_service::CacheStats {
        let hits = self.metrics.hits.load(Ordering::Relaxed);
        let misses = self.metrics.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        let hit_rate = if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        };
        crate::ports::cache_service::CacheStats {
            namespace: "overall".to_string(),
            hit_rate,
            entry_count: self.user_permissions.entry_count(),
            max_capacity: self.max_capacity,
        }
    }

    async fn stats_by_namespace(&self) -> Vec<crate::ports::cache_service::CacheStats> {
        let hits = self.metrics.hits.load(Ordering::Relaxed);
        let misses = self.metrics.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        let hit_rate = if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        };

        vec![crate::ports::cache_service::CacheStats {
            namespace: USER_PERMISSIONS_NAMESPACE.to_string(),
            hit_rate,
            entry_count: self.user_permissions.entry_count(),
            max_capacity: self.max_capacity,
        }]
    }
}
