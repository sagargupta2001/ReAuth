use crate::domain::realm_security_headers::RealmSecurityHeaders;
use crate::error::{Error, Result};
use crate::ports::realm_repository::RealmRepository;
use crate::ports::realm_security_headers_repository::RealmSecurityHeadersRepository;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

const MAX_HEADER_LENGTH: usize = 4096;

#[derive(Debug, Deserialize)]
pub struct UpdateRealmSecurityHeadersPayload {
    pub x_frame_options: Option<String>,
    pub content_security_policy: Option<String>,
    pub x_content_type_options: Option<String>,
    pub referrer_policy: Option<String>,
    pub strict_transport_security: Option<String>,
}

pub struct RealmSecurityHeadersService {
    realm_repo: Arc<dyn RealmRepository>,
    headers_repo: Arc<dyn RealmSecurityHeadersRepository>,
}

impl RealmSecurityHeadersService {
    pub fn new(
        realm_repo: Arc<dyn RealmRepository>,
        headers_repo: Arc<dyn RealmSecurityHeadersRepository>,
    ) -> Self {
        Self {
            realm_repo,
            headers_repo,
        }
    }

    pub async fn get_settings(&self, realm_id: Uuid) -> Result<RealmSecurityHeaders> {
        self.ensure_realm_exists(&realm_id).await?;
        let settings = self
            .headers_repo
            .find_by_realm_id(&realm_id)
            .await?
            .unwrap_or_else(|| RealmSecurityHeaders::defaults(realm_id));
        Ok(settings)
    }

    pub async fn update_settings(
        &self,
        realm_id: Uuid,
        payload: UpdateRealmSecurityHeadersPayload,
    ) -> Result<RealmSecurityHeaders> {
        self.ensure_realm_exists(&realm_id).await?;
        let mut settings = self
            .headers_repo
            .find_by_realm_id(&realm_id)
            .await?
            .unwrap_or_else(|| RealmSecurityHeaders::defaults(realm_id));

        if let Some(value) = payload.x_frame_options {
            settings.x_frame_options = normalize_optional(value);
        }
        if let Some(value) = payload.content_security_policy {
            settings.content_security_policy = normalize_optional(value);
        }
        if let Some(value) = payload.x_content_type_options {
            settings.x_content_type_options = normalize_optional(value);
        }
        if let Some(value) = payload.referrer_policy {
            settings.referrer_policy = normalize_optional(value);
        }
        if let Some(value) = payload.strict_transport_security {
            settings.strict_transport_security = normalize_optional(value);
        }

        validate_settings(&settings)?;
        self.headers_repo.upsert(&settings).await?;

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

fn validate_settings(settings: &RealmSecurityHeaders) -> Result<()> {
    let fields = [
        ("x_frame_options", settings.x_frame_options.as_deref()),
        (
            "content_security_policy",
            settings.content_security_policy.as_deref(),
        ),
        (
            "x_content_type_options",
            settings.x_content_type_options.as_deref(),
        ),
        ("referrer_policy", settings.referrer_policy.as_deref()),
        (
            "strict_transport_security",
            settings.strict_transport_security.as_deref(),
        ),
    ];

    for (label, value) in fields {
        if let Some(value) = value {
            if value.len() > MAX_HEADER_LENGTH {
                return Err(Error::Validation(format!(
                    "{} must be <= {} characters",
                    label, MAX_HEADER_LENGTH
                )));
            }
            if value.contains('\n') || value.contains('\r') {
                return Err(Error::Validation(format!(
                    "{} must not contain newline characters",
                    label
                )));
            }
        }
    }

    Ok(())
}
