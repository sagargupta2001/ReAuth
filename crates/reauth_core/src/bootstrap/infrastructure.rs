use crate::adapters::cache::moka_cache::MokaCacheService;
use crate::adapters::crypto::jwt_service::JwtService;
use crate::adapters::crypto::key_manager::KeyManager;
use crate::adapters::eventing::in_memory_bus::InMemoryEventBus;
use crate::config::Settings;
use crate::error;
use std::sync::Arc;

/// Initializes core infrastructure services (Event Bus, Cache, Crypto/JWT).
/// Returns a Result because loading keys from disk can fail.
pub fn initialize_core_infra(
    settings: &Settings,
) -> error::Result<(
    Arc<InMemoryEventBus>,
    Arc<MokaCacheService>,
    Arc<JwtService>, // Using concrete type here is fine for bootstrap
)> {
    // 1. Load or create RSA keys
    // Now settings.database.data_dir exists!
    let key_pair = KeyManager::get_or_create_keys(&settings.database.data_dir)?;

    // 2. Construct services
    let event_bus = Arc::new(InMemoryEventBus::new());
    let cache = Arc::new(MokaCacheService::new());

    // 3. Initialize JWT Service with the loaded keys
    // We use `?` to propagate any key decoding errors
    let jwt = Arc::new(JwtService::new(settings.auth.clone(), key_pair)?);

    Ok((event_bus, cache, jwt))
}
