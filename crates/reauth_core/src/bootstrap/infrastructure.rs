use crate::adapters::cache::moka_cache::MokaCacheService;
use crate::adapters::crypto::jwt_service::JwtService;
use crate::adapters::eventing::in_memory_bus::InMemoryEventBus;
use crate::config::Settings;
use std::sync::Arc;

pub fn initialize_core_infra(
    settings: &Settings,
) -> (
    Arc<InMemoryEventBus>,
    Arc<MokaCacheService>,
    Arc<JwtService>,
) {
    let event_bus = Arc::new(InMemoryEventBus::new());
    let cache = Arc::new(MokaCacheService::new());
    let jwt = Arc::new(JwtService::new(settings.auth.clone()));

    (event_bus, cache, jwt)
}
