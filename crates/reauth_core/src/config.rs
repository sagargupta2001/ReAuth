use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};
use url::Url;

const DEFAULT_CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../config/default.toml"
));

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Server {
    pub scheme: String,
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub public_url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Ui {
    pub dev_url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct CorsConfig {
    #[serde(default)]
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PluginsConfig {
    pub handshake_timeout_secs: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_key_id: String,
    #[serde(default)]
    pub issuer: String,
    pub access_token_ttl_secs: i64,
    pub refresh_token_ttl_secs: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DefaultAdminConfig {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DefaultOidcClientConfig {
    pub client_id: String,
    #[serde(default)]
    pub redirect_uris: Vec<String>,
    #[serde(default)]
    pub web_origins: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub server: Server,
    pub ui: Ui,
    #[serde(default)]
    pub cors: CorsConfig,
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
        settings.validate()?;
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

        self.apply_database_defaults();
    }

    fn apply_database_defaults(&mut self) {
        let url = self.database.url.trim();
        if url.is_empty() || url == "sqlite:data/reauth.db" || url == "sqlite:./data/reauth.db" {
            let data_dir = self.database.data_dir.trim();
            if !data_dir.is_empty() {
                let db_path = Path::new(data_dir).join("reauth.db");
                self.database.url = format!("sqlite:{}", db_path.to_string_lossy());
            }
        }
    }

    fn validate(&self) -> Result<(), config::ConfigError> {
        let scheme = self.server.scheme.trim();
        if scheme.is_empty() {
            return Err(config::ConfigError::Message(
                "server.scheme must not be empty".to_string(),
            ));
        }
        if scheme != "http" && scheme != "https" {
            return Err(config::ConfigError::Message(format!(
                "server.scheme must be http or https (got {})",
                scheme
            )));
        }
        if self.server.host.trim().is_empty() {
            return Err(config::ConfigError::Message(
                "server.host must not be empty".to_string(),
            ));
        }

        validate_url("server.public_url", &self.server.public_url)?;
        if !self.ui.dev_url.trim().is_empty() {
            validate_url("ui.dev_url", &self.ui.dev_url)?;
        }

        validate_url_list("cors.allowed_origins", &self.cors.allowed_origins)?;
        validate_url_list(
            "default_oidc_client.web_origins",
            &self.default_oidc_client.web_origins,
        )?;

        Ok(())
    }

    pub fn redacted(&self) -> Self {
        let mut redacted = self.clone();
        redacted.auth.jwt_secret = "<redacted>".to_string();
        redacted.default_admin.password = "<redacted>".to_string();
        redacted
    }
}

fn default_data_dir() -> String {
    env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("data")))
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|| "./data".to_string())
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

fn validate_url(field: &str, value: &str) -> Result<(), config::ConfigError> {
    if value.trim().is_empty() {
        return Err(config::ConfigError::Message(format!(
            "{} must not be empty",
            field
        )));
    }
    Url::parse(value).map_err(|e| {
        config::ConfigError::Message(format!("{} must be a valid URL: {}", field, e))
    })?;
    Ok(())
}

fn validate_url_list(field: &str, values: &[String]) -> Result<(), config::ConfigError> {
    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(config::ConfigError::Message(format!(
                "{} must not contain empty entries",
                field
            )));
        }
        Url::parse(trimmed).map_err(|e| {
            config::ConfigError::Message(format!(
                "{} entry '{}' must be a valid URL: {}",
                field, trimmed, e
            ))
        })?;
    }
    Ok(())
}
