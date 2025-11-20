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
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_key_id: String,
    pub access_token_ttl_secs: i64,
    pub refresh_token_ttl_secs: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DefaultAdminConfig {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DefaultOidcClientConfig {
    pub client_id: String,
    pub redirect_uris: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: Server,
    pub ui: Ui,
    pub plugins: PluginsConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub default_admin: DefaultAdminConfig,
    pub default_oidc_client: DefaultOidcClientConfig,
}

impl Settings {
    pub fn new() -> Result<Self, config::ConfigError> {
        let s = config::Config::builder()
            .add_source(config::File::with_name("config/default"))
            .add_source(config::Environment::with_prefix("REAUTH"))
            .add_source(config::Environment::default().separator("__"))
            .build()?;
        s.try_deserialize()
    }
}
