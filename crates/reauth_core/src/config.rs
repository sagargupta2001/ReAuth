use serde::Deserialize;
use std::env;
use std::path::PathBuf;

const DEFAULT_CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../config/default.toml"
));

#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub scheme: String,
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub public_url: String,
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
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_key_id: String,
    #[serde(default)]
    pub issuer: String,
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
    #[serde(default)]
    pub redirect_uris: Vec<String>,
    #[serde(default)]
    pub web_origins: Vec<String>,
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
        let mut builder = config::Config::builder()
            .add_source(config::File::from_str(
                DEFAULT_CONFIG,
                config::FileFormat::Toml,
            ))
            // Optional local defaults for dev workflows.
            .add_source(config::File::with_name("config/default").required(false));

        if let Some(config_path) = resolve_config_path() {
            builder = builder.add_source(config::File::from(config_path));
        } else if let Some(exe_config_path) = resolve_exe_config_path() {
            builder = builder.add_source(config::File::from(exe_config_path).required(false));
        }

        let s = builder
            .add_source(config::Environment::with_prefix("REAUTH").separator("__"))
            .build()?;

        let mut settings: Settings = s.try_deserialize()?;
        settings.apply_defaults();
        Ok(settings)
    }

    fn apply_defaults(&mut self) {
        let public_url = if self.server.public_url.trim().is_empty() {
            let public_host = if self.server.host == "127.0.0.1" {
                "localhost"
            } else {
                self.server.host.as_str()
            };
            format!(
                "{}://{}:{}",
                self.server.scheme, public_host, self.server.port
            )
        } else {
            self.server.public_url.trim().to_string()
        };
        self.server.public_url = public_url.clone();

        if self.auth.issuer.trim().is_empty() {
            self.auth.issuer = public_url.clone();
        }

        if self.default_oidc_client.redirect_uris.is_empty() {
            self.default_oidc_client.redirect_uris =
                build_default_urls(&public_url, &self.ui.dev_url);
        }

        if self.default_oidc_client.web_origins.is_empty() {
            self.default_oidc_client.web_origins =
                build_default_urls(&public_url, &self.ui.dev_url);
        }
    }
}

fn default_data_dir() -> String {
    "./data".to_string()
}

fn resolve_config_path() -> Option<PathBuf> {
    match env::var("REAUTH_CONFIG") {
        Ok(value) if !value.trim().is_empty() => Some(PathBuf::from(value)),
        _ => None,
    }
}

fn resolve_exe_config_path() -> Option<PathBuf> {
    let exe_path = env::current_exe().ok()?;
    let exe_dir = exe_path.parent()?;
    Some(exe_dir.join("reauth.toml"))
}

fn build_default_urls(public_url: &str, dev_url: &str) -> Vec<String> {
    let mut urls = Vec::new();
    push_unique(&mut urls, public_url);
    if !dev_url.trim().is_empty() {
        push_unique(&mut urls, dev_url);
    }
    urls
}

fn push_unique(urls: &mut Vec<String>, value: &str) {
    if !urls.iter().any(|u| u == value) {
        urls.push(value.to_string());
    }
}
