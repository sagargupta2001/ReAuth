use crate::adapters::cache::cache_invalidator::CacheInvalidator;
use crate::adapters::cache::moka_cache::MokaCacheService;
use crate::adapters::eventing::in_memory_bus::InMemoryEventBus;
use crate::ports::event_bus::EventSubscriber;
use crate::ports::rbac_repository::RbacRepository;
use std::sync::Arc;

pub async fn subscribe_event_listeners(
    bus: &Arc<InMemoryEventBus>,
    cache: &Arc<MokaCacheService>,
    rbac_repo: &Arc<dyn RbacRepository>,
) {
    let cache_invalidator = Arc::new(CacheInvalidator::new(cache.clone(), rbac_repo.clone()));
    bus.subscribe(cache_invalidator).await;
}
