use crate::domain::realm_recovery_settings::RealmRecoverySettings;
use crate::error::{Error, Result};
use crate::ports::realm_recovery_settings_repository::RealmRecoverySettingsRepository;
use crate::ports::realm_repository::RealmRepository;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

const MIN_TOKEN_TTL: i64 = 5;
const MAX_TOKEN_TTL: i64 = 1440;
const MAX_RATE_LIMIT: i64 = 50;
const MAX_WINDOW_MINUTES: i64 = 120;

#[derive(Debug, Deserialize)]
pub struct UpdateRealmRecoverySettingsPayload {
    pub token_ttl_minutes: Option<i64>,
    pub rate_limit_max: Option<i64>,
    pub rate_limit_window_minutes: Option<i64>,
    pub revoke_sessions_on_reset: Option<bool>,
    pub email_subject: Option<String>,
    pub email_body: Option<String>,
}

pub struct RealmRecoverySettingsService {
    realm_repo: Arc<dyn RealmRepository>,
    recovery_repo: Arc<dyn RealmRecoverySettingsRepository>,
}

impl RealmRecoverySettingsService {
    pub fn new(
        realm_repo: Arc<dyn RealmRepository>,
        recovery_repo: Arc<dyn RealmRecoverySettingsRepository>,
    ) -> Self {
        Self {
            realm_repo,
            recovery_repo,
        }
    }

    pub async fn get_settings(&self, realm_id: Uuid) -> Result<RealmRecoverySettings> {
        self.ensure_realm_exists(&realm_id).await?;
        let settings = self
            .recovery_repo
            .find_by_realm_id(&realm_id)
            .await?
            .unwrap_or_else(|| RealmRecoverySettings::defaults(realm_id));
        Ok(settings)
    }

    pub async fn update_settings(
        &self,
        realm_id: Uuid,
        payload: UpdateRealmRecoverySettingsPayload,
    ) -> Result<RealmRecoverySettings> {
        self.ensure_realm_exists(&realm_id).await?;
        let mut settings = self
            .recovery_repo
            .find_by_realm_id(&realm_id)
            .await?
            .unwrap_or_else(|| RealmRecoverySettings::defaults(realm_id));

        if let Some(value) = payload.token_ttl_minutes {
            settings.token_ttl_minutes = value;
        }
        if let Some(value) = payload.rate_limit_max {
            settings.rate_limit_max = value;
        }
        if let Some(value) = payload.rate_limit_window_minutes {
            settings.rate_limit_window_minutes = value;
        }
        if let Some(value) = payload.revoke_sessions_on_reset {
            settings.revoke_sessions_on_reset = value;
        }
        if let Some(value) = payload.email_subject {
            settings.email_subject = normalize_optional(value);
        }
        if let Some(value) = payload.email_body {
            settings.email_body = normalize_optional(value);
        }

        validate_settings(&settings)?;
        self.recovery_repo.upsert(&settings).await?;

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

fn validate_settings(settings: &RealmRecoverySettings) -> Result<()> {
    if settings.token_ttl_minutes < MIN_TOKEN_TTL || settings.token_ttl_minutes > MAX_TOKEN_TTL {
        return Err(Error::Validation(format!(
            "token_ttl_minutes must be between {} and {}",
            MIN_TOKEN_TTL, MAX_TOKEN_TTL
        )));
    }

    if settings.rate_limit_max < 0 || settings.rate_limit_max > MAX_RATE_LIMIT {
        return Err(Error::Validation(format!(
            "rate_limit_max must be between 0 and {}",
            MAX_RATE_LIMIT
        )));
    }

    if settings.rate_limit_window_minutes < 1
        || settings.rate_limit_window_minutes > MAX_WINDOW_MINUTES
    {
        return Err(Error::Validation(format!(
            "rate_limit_window_minutes must be between 1 and {}",
            MAX_WINDOW_MINUTES
        )));
    }

    Ok(())
}
