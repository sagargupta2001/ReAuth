use crate::adapters::cache::cache_invalidator::CacheInvalidator;
use crate::adapters::cache::moka_cache::MokaCacheService;
use crate::adapters::eventing::in_memory_bus::InMemoryEventBus;
use crate::adapters::persistence::sqlite_rbac_repository::SqliteRbacRepository;
use crate::adapters::PluginEventGateway;
use crate::ports::event_bus::EventSubscriber;
use manager::PluginManager;
use std::sync::Arc;

pub async fn subscribe_event_listeners(
    bus: &Arc<InMemoryEventBus>,
    cache: &Arc<MokaCacheService>,
    rbac_repo: &Arc<SqliteRbacRepository>,
    plugin_manager: PluginManager,
) {
    let cache_invalidator = Arc::new(CacheInvalidator::new(cache.clone(), rbac_repo.clone()));
    bus.subscribe(cache_invalidator).await;

    let plugin_gateway = Arc::new(PluginEventGateway::new(plugin_manager));
    bus.subscribe(plugin_gateway).await;
}
