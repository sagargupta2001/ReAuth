use crate::domain::realm_passkey_settings::RealmPasskeySettings;
use crate::error::{Error, Result};
use crate::ports::realm_passkey_settings_repository::RealmPasskeySettingsRepository;
use crate::ports::realm_repository::RealmRepository;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

const MIN_CHALLENGE_TTL_SECS: i64 = 30;
const MAX_CHALLENGE_TTL_SECS: i64 = 600;
const MIN_REAUTH_MAX_AGE_SECS: i64 = 0;
const MAX_REAUTH_MAX_AGE_SECS: i64 = 86_400;

#[derive(Debug, Deserialize)]
pub struct UpdateRealmPasskeySettingsPayload {
    pub enabled: Option<bool>,
    pub allow_password_fallback: Option<bool>,
    pub discoverable_preferred: Option<bool>,
    pub challenge_ttl_secs: Option<i64>,
    pub reauth_max_age_secs: Option<i64>,
}

pub struct RealmPasskeySettingsService {
    realm_repo: Arc<dyn RealmRepository>,
    passkey_repo: Arc<dyn RealmPasskeySettingsRepository>,
}

impl RealmPasskeySettingsService {
    pub fn new(
        realm_repo: Arc<dyn RealmRepository>,
        passkey_repo: Arc<dyn RealmPasskeySettingsRepository>,
    ) -> Self {
        Self {
            realm_repo,
            passkey_repo,
        }
    }

    pub async fn get_settings(&self, realm_id: Uuid) -> Result<RealmPasskeySettings> {
        self.ensure_realm_exists(&realm_id).await?;
        Ok(self
            .passkey_repo
            .find_by_realm_id(&realm_id)
            .await?
            .unwrap_or_else(|| RealmPasskeySettings::defaults(realm_id)))
    }

    pub async fn update_settings(
        &self,
        realm_id: Uuid,
        payload: UpdateRealmPasskeySettingsPayload,
    ) -> Result<RealmPasskeySettings> {
        self.ensure_realm_exists(&realm_id).await?;
        let mut settings = self
            .passkey_repo
            .find_by_realm_id(&realm_id)
            .await?
            .unwrap_or_else(|| RealmPasskeySettings::defaults(realm_id));

        if let Some(value) = payload.enabled {
            settings.enabled = value;
        }
        if let Some(value) = payload.allow_password_fallback {
            settings.allow_password_fallback = value;
        }
        if let Some(value) = payload.discoverable_preferred {
            settings.discoverable_preferred = value;
        }
        if let Some(value) = payload.challenge_ttl_secs {
            settings.challenge_ttl_secs = value;
        }
        if let Some(value) = payload.reauth_max_age_secs {
            settings.reauth_max_age_secs = value;
        }

        validate_settings(&settings)?;
        self.passkey_repo.upsert(&settings).await?;
        Ok(settings)
    }

    async fn ensure_realm_exists(&self, realm_id: &Uuid) -> Result<()> {
        if self.realm_repo.find_by_id(realm_id).await?.is_none() {
            return Err(Error::RealmNotFound(realm_id.to_string()));
        }
        Ok(())
    }
}

fn validate_settings(settings: &RealmPasskeySettings) -> Result<()> {
    if settings.challenge_ttl_secs < MIN_CHALLENGE_TTL_SECS
        || settings.challenge_ttl_secs > MAX_CHALLENGE_TTL_SECS
    {
        return Err(Error::Validation(format!(
            "challenge_ttl_secs must be between {} and {}",
            MIN_CHALLENGE_TTL_SECS, MAX_CHALLENGE_TTL_SECS
        )));
    }

    if settings.reauth_max_age_secs < MIN_REAUTH_MAX_AGE_SECS
        || settings.reauth_max_age_secs > MAX_REAUTH_MAX_AGE_SECS
    {
        return Err(Error::Validation(format!(
            "reauth_max_age_secs must be between {} and {}",
            MIN_REAUTH_MAX_AGE_SECS, MAX_REAUTH_MAX_AGE_SECS
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_settings_rejects_short_challenge_ttl() {
        let mut settings = RealmPasskeySettings::defaults(Uuid::new_v4());
        settings.challenge_ttl_secs = 5;
        let err = validate_settings(&settings).expect_err("expected validation failure");
        assert!(err.to_string().contains("challenge_ttl_secs"));
    }

    #[test]
    fn validate_settings_rejects_large_reauth_window() {
        let mut settings = RealmPasskeySettings::defaults(Uuid::new_v4());
        settings.reauth_max_age_secs = 100_000;
        let err = validate_settings(&settings).expect_err("expected validation failure");
        assert!(err.to_string().contains("reauth_max_age_secs"));
    }
}
