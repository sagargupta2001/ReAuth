//! Event handler for performing targeted cache invalidation.

use crate::{
    domain::events::DomainEvent,
    ports::{
        cache_service::CacheService, event_bus::EventHandler, rbac_repository::RbacRepository,
    },
};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, error};

/// A listener that invalidates the cache based on domain events.
pub struct CacheInvalidator {
    cache: Arc<dyn CacheService>,
    rbac_repo: Arc<dyn RbacRepository>,
}

impl CacheInvalidator {
    pub fn new(cache: Arc<dyn CacheService>, rbac_repo: Arc<dyn RbacRepository>) -> Self {
        Self { cache, rbac_repo }
    }
}

#[async_trait]
impl EventHandler for CacheInvalidator {
    /// Handles incoming events and clears the relevant caches.
    async fn handle(&self, event: &DomainEvent) {
        match event {
            DomainEvent::UserAssignedToGroup(e) => {
                debug!("Invalidating cache for user: {}", e.user_id);
                // Simple case: Invalidate one user's permissions
                self.cache.clear_user_permissions(&e.user_id).await;
            }
            DomainEvent::UserRemovedFromGroup(e) => {
                debug!("Invalidating cache for user: {} (Removed from Group)", e.user_id);
                self.cache.clear_user_permissions(&e.user_id).await;
            }
            DomainEvent::RoleAssignedToGroup(e) => {
                debug!("Invalidating cache for group: {}", e.group_id);
                // Complex case: A role was added to a group.
                // We must invalidate *all users* in that group.
                match self.rbac_repo.find_user_ids_in_group(&e.group_id).await {
                    Ok(user_ids) => {
                        for user_id in user_ids {
                            self.cache.clear_user_permissions(&user_id).await;
                        }
                    }
                    Err(e) => error!(
                        "Failed to find users in group for cache invalidation: {}",
                        e
                    ),
                }
            }
            DomainEvent::RoleRemovedFromGroup(e) => {
                debug!("Invalidating cache for group: {} (Role Removed)", e.group_id);
                match self.rbac_repo.find_user_ids_in_group(&e.group_id).await {
                    Ok(user_ids) => {
                        for user_id in user_ids {
                            self.cache.clear_user_permissions(&user_id).await;
                        }
                    }
                    Err(e) => error!(
                        "Failed to find users in group for cache invalidation: {}",
                        e
                    ),
                }
            }
            // Invalidate on user creation to clear any "empty" cache entries
            DomainEvent::UserCreated(e) => {
                self.cache.clear_user_permissions(&e.user_id).await;
            }
            DomainEvent::RolePermissionChanged(e) => {
                debug!(
                    "Event: RolePermissionChanged. Invalidating cache for users with role: {}",
                    e.role_id
                );
                match self.rbac_repo.find_user_ids_for_role(&e.role_id).await {
                    Ok(user_ids) => {
                        for user_id in user_ids {
                            debug!("Invalidating cache for user: {}", user_id);
                            self.cache.clear_user_permissions(&user_id).await;
                        }
                    }
                    Err(e) => error!(
                        "Failed to find users for role for cache invalidation: {}",
                        e
                    ),
                }
            }
            DomainEvent::RoleCompositeChanged(e) => {
                debug!(
                    "Event: RoleCompositeChanged. Invalidating cache for users with role: {}",
                    e.parent_role_id
                );
                match self
                    .rbac_repo
                    .find_user_ids_for_role(&e.parent_role_id)
                    .await
                {
                    Ok(user_ids) => {
                        for user_id in user_ids {
                            self.cache.clear_user_permissions(&user_id).await;
                        }
                    }
                    Err(e) => error!(
                        "Failed to find users for composite role cache invalidation: {}",
                        e
                    ),
                }
            }

            DomainEvent::UserRoleAssigned(e) => {
                debug!("Invalidating cache for user: {} (Role Assigned)", e.user_id);
                self.cache.clear_user_permissions(&e.user_id).await;
            }
            DomainEvent::UserRoleRemoved(e) => {
                debug!("Invalidating cache for user: {} (Role Removed)", e.user_id);
                self.cache.clear_user_permissions(&e.user_id).await;
            }

            DomainEvent::RoleDeleted(e) => {
                debug!(
                    "Role {} deleted. Invalidating cache for {} users.",
                    e.role_id,
                    e.affected_user_ids.len()
                );

                // Since we already did the heavy lifting (finding users) in the service,
                // this loop is fast.
                for user_id in &e.affected_user_ids {
                    self.cache.clear_user_permissions(user_id).await;
                }
            }
            DomainEvent::GroupDeleted(e) => {
                debug!(
                    "Groups deleted. Invalidating cache for {} users.",
                    e.affected_user_ids.len()
                );

                for user_id in &e.affected_user_ids {
                    self.cache.clear_user_permissions(user_id).await;
                }
            }
        }
    }
}
