use crate::application::audit_service::AuditService;
use crate::domain::audit::NewAuditEvent;
use crate::domain::oauth_start_attempt::OAuthStartAttempt;
use crate::domain::realm_idp_settings::RealmIdpSettings;
use crate::error::{Error, Result};
use crate::ports::oauth_start_attempt_repository::OAuthStartAttemptRepository;
use crate::ports::realm_idp_settings_repository::RealmIdpSettingsRepository;
use crate::ports::realm_repository::RealmRepository;
use chrono::{Duration, Utc};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

const MAX_RATE_LIMIT: i64 = 50;
const MAX_WINDOW_MINUTES: i64 = 120;

#[derive(Debug, Deserialize)]
pub struct UpdateRealmIdpSettingsPayload {
    pub oauth_start_rate_limit_max: Option<i64>,
    pub oauth_start_rate_limit_window_minutes: Option<i64>,
}

pub struct RealmIdpSettingsService {
    realm_repo: Arc<dyn RealmRepository>,
    settings_repo: Arc<dyn RealmIdpSettingsRepository>,
    oauth_start_attempt_repo: Arc<dyn OAuthStartAttemptRepository>,
    audit_service: Arc<AuditService>,
}

impl RealmIdpSettingsService {
    pub fn new(
        realm_repo: Arc<dyn RealmRepository>,
        settings_repo: Arc<dyn RealmIdpSettingsRepository>,
        oauth_start_attempt_repo: Arc<dyn OAuthStartAttemptRepository>,
        audit_service: Arc<AuditService>,
    ) -> Self {
        Self {
            realm_repo,
            settings_repo,
            oauth_start_attempt_repo,
            audit_service,
        }
    }

    pub async fn get_settings(&self, realm_id: Uuid) -> Result<RealmIdpSettings> {
        self.ensure_realm_exists(&realm_id).await?;
        let settings = self
            .settings_repo
            .find_by_realm_id(&realm_id)
            .await?
            .unwrap_or_else(|| RealmIdpSettings::defaults(realm_id));
        Ok(settings)
    }

    pub async fn update_settings(
        &self,
        realm_id: Uuid,
        payload: UpdateRealmIdpSettingsPayload,
    ) -> Result<RealmIdpSettings> {
        self.ensure_realm_exists(&realm_id).await?;
        let mut settings = self
            .settings_repo
            .find_by_realm_id(&realm_id)
            .await?
            .unwrap_or_else(|| RealmIdpSettings::defaults(realm_id));

        if let Some(value) = payload.oauth_start_rate_limit_max {
            settings.oauth_start_rate_limit_max = value;
        }
        if let Some(value) = payload.oauth_start_rate_limit_window_minutes {
            settings.oauth_start_rate_limit_window_minutes = value;
        }

        validate_settings(&settings)?;
        self.settings_repo.upsert(&settings).await?;
        Ok(settings)
    }

    pub async fn enforce_oauth_start_rate_limit(
        &self,
        realm_id: Uuid,
        provider_id: Uuid,
        provider_alias: &str,
        ip_address: &str,
    ) -> Result<()> {
        let settings = self.get_settings(realm_id).await?;
        if settings.oauth_start_rate_limit_max == 0 {
            return Ok(());
        }

        let now = Utc::now();
        let window = Duration::minutes(settings.oauth_start_rate_limit_window_minutes);
        let existing = self
            .oauth_start_attempt_repo
            .find(&realm_id, &provider_id, ip_address)
            .await?;

        let mut attempt = existing.unwrap_or_else(|| OAuthStartAttempt {
            realm_id,
            provider_id,
            ip_address: ip_address.to_string(),
            window_started_at: now,
            attempt_count: 0,
            updated_at: now,
        });

        if now >= attempt.window_started_at + window {
            attempt.window_started_at = now;
            attempt.attempt_count = 0;
        }

        if attempt.attempt_count >= settings.oauth_start_rate_limit_max {
            let retry_after_seconds = ((attempt.window_started_at + window) - now)
                .num_seconds()
                .max(0);
            self.audit_service
                .record(NewAuditEvent {
                    realm_id,
                    actor_user_id: None,
                    action: "idp_start_rate_limited".to_string(),
                    target_type: "identity_provider".to_string(),
                    target_id: Some(provider_id.to_string()),
                    metadata: json!({
                        "provider_alias": provider_alias,
                        "ip_address": ip_address,
                        "attempt_count": attempt.attempt_count,
                        "rate_limit_max": settings.oauth_start_rate_limit_max,
                        "rate_limit_window_minutes": settings.oauth_start_rate_limit_window_minutes,
                        "retry_after_seconds": retry_after_seconds,
                    }),
                })
                .await?;
            return Err(Error::RateLimited(
                "Too many OAuth sign-in attempts for this identity provider. Try again later."
                    .to_string(),
            ));
        }

        attempt.attempt_count += 1;
        attempt.updated_at = now;
        self.oauth_start_attempt_repo.upsert(&attempt).await?;
        Ok(())
    }

    async fn ensure_realm_exists(&self, realm_id: &Uuid) -> Result<()> {
        if self.realm_repo.find_by_id(realm_id).await?.is_none() {
            return Err(Error::RealmNotFound(realm_id.to_string()));
        }
        Ok(())
    }
}

fn validate_settings(settings: &RealmIdpSettings) -> Result<()> {
    if settings.oauth_start_rate_limit_max < 0
        || settings.oauth_start_rate_limit_max > MAX_RATE_LIMIT
    {
        return Err(Error::Validation(format!(
            "oauth_start_rate_limit_max must be between 0 and {}",
            MAX_RATE_LIMIT
        )));
    }

    if settings.oauth_start_rate_limit_window_minutes < 1
        || settings.oauth_start_rate_limit_window_minutes > MAX_WINDOW_MINUTES
    {
        return Err(Error::Validation(format!(
            "oauth_start_rate_limit_window_minutes must be between 1 and {}",
            MAX_WINDOW_MINUTES
        )));
    }

    Ok(())
}
