use crate::config::Settings;
use crate::constants;
use manager::{ManagerConfig, PluginManager};
use std::env;
use std::path::{Path, PathBuf};
use tracing::info;

pub fn determine_plugins_path() -> anyhow::Result<PathBuf> {
    let exe_path = env::current_exe()?;

    let is_dev = exe_path
        .ancestors()
        .any(|p| p.ends_with(constants::DEV_ENVIRONMENT_DIR));

    let path = if is_dev {
        PathBuf::from(constants::PLUGINS_DIR)
    } else {
        let mut prod = exe_path;
        prod.pop();
        prod.join(constants::PLUGINS_DIR)
    };

    info!("Loading plugins from: {:?}", path);
    Ok(path)
}

pub fn initialize_plugins(settings: &Settings, plugins_path: &Path) -> PluginManager {
    let cfg = ManagerConfig {
        handshake_timeout_secs: settings.plugins.handshake_timeout_secs,
    };

    PluginManager::new(cfg, plugins_path.to_path_buf())
}
