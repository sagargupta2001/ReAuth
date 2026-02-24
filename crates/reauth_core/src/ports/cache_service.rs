//! Defines the Port for a generic key-value cache service.

use async_trait::async_trait;
use std::collections::HashSet;
use uuid::Uuid;

/// A Port for a simple, asynchronous key-value cache.
/// This is used primarily for caching user permissions.
#[async_trait]
pub trait CacheService: Send + Sync {
    /// Retrieves a user's permission set from the cache.
    async fn get_user_permissions(&self, user_id: &Uuid) -> Option<HashSet<String>>;

    /// Stores a user's permission set in the cache.
    async fn set_user_permissions(&self, user_id: &Uuid, permissions: &HashSet<String>);

    /// Removes a user's permission set from the cache (invalidation).
    async fn clear_user_permissions(&self, user_id: &Uuid);

    /// Clears all cached data.
    async fn clear_all(&self);

    /// Clears a specific cache namespace.
    async fn clear_namespace(&self, namespace: &str);

    /// Returns stats for observability dashboards.
    async fn stats(&self) -> CacheStats;

    /// Returns stats for each cache namespace.
    async fn stats_by_namespace(&self) -> Vec<CacheStats>;
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CacheStats {
    pub namespace: String,
    pub hit_rate: f64,
    pub entry_count: u64,
    pub max_capacity: u64,
}
