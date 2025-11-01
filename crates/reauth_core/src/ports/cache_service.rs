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
    async fn set_user_permissions(
        &self,
        user_id: &Uuid,
        permissions: &HashSet<String>,
    );

    /// Removes a user's permission set from the cache (invalidation).
    async fn clear_user_permissions(&self, user_id: &Uuid);
}