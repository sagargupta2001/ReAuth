use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub scheme: String,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Ui {
    pub dev_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PluginsConfig {
    pub handshake_timeout_secs: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: Server,
    pub ui: Ui,
    pub plugins: PluginsConfig,
}


impl Settings {
    pub fn new() -> Result<Self, config::ConfigError> {
        let s = config::Config::builder()
            .add_source(config::File::with_name("config/default"))
            .add_source(config::Environment::with_prefix("REAUTH"))
            .build()?;
        s.try_deserialize()
    }
}