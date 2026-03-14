use crate::domain::realm_email_settings::RealmEmailSettings;
use crate::error::{Error, Result};
use crate::ports::realm_email_settings_repository::RealmEmailSettingsRepository;
use crate::ports::realm_repository::RealmRepository;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

const VALID_SMTP_SECURITY: [&str; 3] = ["starttls", "tls", "none"];

#[derive(Debug, Deserialize)]
pub struct UpdateRealmEmailSettingsPayload {
    pub enabled: Option<bool>,
    pub from_address: Option<String>,
    pub from_name: Option<String>,
    pub reply_to_address: Option<String>,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<i64>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_security: Option<String>,
}

pub struct RealmEmailSettingsService {
    realm_repo: Arc<dyn RealmRepository>,
    email_repo: Arc<dyn RealmEmailSettingsRepository>,
}

impl RealmEmailSettingsService {
    pub fn new(
        realm_repo: Arc<dyn RealmRepository>,
        email_repo: Arc<dyn RealmEmailSettingsRepository>,
    ) -> Self {
        Self {
            realm_repo,
            email_repo,
        }
    }

    pub async fn get_settings(&self, realm_id: Uuid) -> Result<RealmEmailSettings> {
        self.ensure_realm_exists(&realm_id).await?;
        let settings = self
            .email_repo
            .find_by_realm_id(&realm_id)
            .await?
            .unwrap_or_else(|| RealmEmailSettings::disabled(realm_id));
        Ok(settings)
    }

    pub async fn update_settings(
        &self,
        realm_id: Uuid,
        payload: UpdateRealmEmailSettingsPayload,
    ) -> Result<RealmEmailSettings> {
        self.ensure_realm_exists(&realm_id).await?;
        let mut settings = self
            .email_repo
            .find_by_realm_id(&realm_id)
            .await?
            .unwrap_or_else(|| RealmEmailSettings::disabled(realm_id));

        if let Some(enabled) = payload.enabled {
            settings.enabled = enabled;
        }
        if let Some(from_address) = payload.from_address {
            settings.from_address = normalize_optional(from_address);
        }
        if let Some(from_name) = payload.from_name {
            settings.from_name = normalize_optional(from_name);
        }
        if let Some(reply_to_address) = payload.reply_to_address {
            settings.reply_to_address = normalize_optional(reply_to_address);
        }
        if let Some(smtp_host) = payload.smtp_host {
            settings.smtp_host = normalize_optional(smtp_host);
        }
        if let Some(smtp_port) = payload.smtp_port {
            settings.smtp_port = Some(smtp_port);
        }
        if let Some(smtp_username) = payload.smtp_username {
            settings.smtp_username = normalize_optional(smtp_username);
        }
        if let Some(smtp_password) = payload.smtp_password {
            let normalized = smtp_password.trim().to_string();
            if !normalized.is_empty() {
                settings.smtp_password = Some(normalized);
            }
        }
        if let Some(security) = payload.smtp_security {
            let normalized = security.trim().to_lowercase();
            if !normalized.is_empty() {
                settings.smtp_security = normalized;
            }
        }

        validate_settings(&settings)?;
        self.email_repo.upsert(&settings).await?;

        Ok(settings)
    }

    async fn ensure_realm_exists(&self, realm_id: &Uuid) -> Result<()> {
        if self.realm_repo.find_by_id(realm_id).await?.is_none() {
            return Err(Error::RealmNotFound(realm_id.to_string()));
        }
        Ok(())
    }
}

fn normalize_optional(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn validate_settings(settings: &RealmEmailSettings) -> Result<()> {
    if !VALID_SMTP_SECURITY.contains(&settings.smtp_security.as_str()) {
        return Err(Error::Validation(format!(
            "smtp_security must be one of {}",
            VALID_SMTP_SECURITY.join(", ")
        )));
    }

    if !settings.enabled {
        return Ok(());
    }

    if settings.smtp_host.as_deref().unwrap_or("").is_empty() {
        return Err(Error::Validation(
            "smtp_host is required when email is enabled".to_string(),
        ));
    }

    if settings.from_address.as_deref().unwrap_or("").is_empty() {
        return Err(Error::Validation(
            "from_address is required when email is enabled".to_string(),
        ));
    }

    if let Some(port) = settings.smtp_port {
        if !(1..=65535).contains(&port) {
            return Err(Error::Validation(
                "smtp_port must be between 1 and 65535".to_string(),
            ));
        }
    }

    Ok(())
}
