use crate::config::Settings;
use crate::domain::realm_email_settings::RealmEmailSettings;
use crate::domain::realm_recovery_settings::RealmRecoverySettings;
use crate::error::{Error, Result};
use crate::ports::realm_email_settings_repository::RealmEmailSettingsRepository;
use crate::ports::realm_recovery_settings_repository::RealmRecoverySettingsRepository;
use crate::ports::realm_repository::RealmRepository;
use chrono::{DateTime, Utc};
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::Tls;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::sync::Arc;
use tracing::warn;
use uuid::Uuid;

pub struct EmailDeliveryService {
    realm_repo: Arc<dyn RealmRepository>,
    email_repo: Arc<dyn RealmEmailSettingsRepository>,
    recovery_repo: Arc<dyn RealmRecoverySettingsRepository>,
    settings: Settings,
}

pub struct RecoveryEmail {
    pub identifier: String,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub resume_path: String,
}

impl EmailDeliveryService {
    pub fn new(
        realm_repo: Arc<dyn RealmRepository>,
        email_repo: Arc<dyn RealmEmailSettingsRepository>,
        recovery_repo: Arc<dyn RealmRecoverySettingsRepository>,
        settings: Settings,
    ) -> Self {
        Self {
            realm_repo,
            email_repo,
            recovery_repo,
            settings,
        }
    }

    pub async fn send_recovery_email(
        &self,
        realm_id: &Uuid,
        request: RecoveryEmail,
    ) -> Result<bool> {
        let Some(realm) = self.realm_repo.find_by_id(realm_id).await? else {
            return Ok(false);
        };

        let settings = self
            .email_repo
            .find_by_realm_id(realm_id)
            .await?
            .unwrap_or_else(|| RealmEmailSettings::disabled(*realm_id));

        if !settings.enabled {
            return Ok(false);
        }

        if !looks_like_email(&request.identifier) {
            return Ok(false);
        }

        let Some(from_address) = settings.from_address.clone() else {
            warn!("Email delivery skipped: from_address is missing.");
            return Ok(false);
        };

        let Some(host) = settings.smtp_host.clone() else {
            warn!("Email delivery skipped: smtp_host is missing.");
            return Ok(false);
        };

        let from_addr = from_address
            .parse()
            .map_err(|err| Error::Validation(format!("Invalid from_address: {}", err)))?;
        let to_addr = request
            .identifier
            .parse()
            .map_err(|err| Error::Validation(format!("Invalid recipient address: {}", err)))?;
        let from = Mailbox::new(settings.from_name.clone(), from_addr);
        let to = Mailbox::new(None, to_addr);

        let resume_url = build_resume_url(
            &self.settings,
            &realm.name,
            &request.resume_path,
            &request.token,
        );

        let recovery_settings = self
            .recovery_repo
            .find_by_realm_id(realm_id)
            .await?
            .unwrap_or_else(|| RealmRecoverySettings::defaults(*realm_id));

        let default_subject = "Reset your password for {realm}".to_string();
        let default_body = "A password reset was requested for realm {realm}.\n\n\
Resume your recovery using this link:\n{resume_url}\n\n\
Recovery token: {token}\n\
Expires at: {expires_at}\n\n\
If you did not request this, you can ignore this email."
            .to_string();

        let subject_template = recovery_settings
            .email_subject
            .clone()
            .unwrap_or(default_subject);
        let body_template = recovery_settings.email_body.clone().unwrap_or(default_body);

        let subject = apply_template(
            &subject_template,
            &realm.name,
            &request.identifier,
            &request.token,
            &resume_url,
            &request.expires_at.to_rfc3339(),
        );
        let body = apply_template(
            &body_template,
            &realm.name,
            &request.identifier,
            &request.token,
            &resume_url,
            &request.expires_at.to_rfc3339(),
        );

        let mut message = Message::builder().from(from).to(to).subject(subject);

        if let Some(reply_to) = settings.reply_to_address.clone() {
            if let Ok(mailbox) = reply_to.parse::<Mailbox>() {
                message = message.reply_to(mailbox);
            }
        }

        let message = message
            .body(body)
            .map_err(|err| Error::Unexpected(err.into()))?;

        let mailer = build_mailer(&settings, &host)?;

        mailer
            .send(message)
            .await
            .map_err(|err| Error::Unexpected(err.into()))?;

        Ok(true)
    }
}

fn build_mailer(
    settings: &RealmEmailSettings,
    host: &str,
) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
    let port = settings.smtp_port.unwrap_or(587) as u16;

    let mut builder = match settings.smtp_security.as_str() {
        "none" => AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host).port(port),
        "tls" => {
            let tls_parameters =
                lettre::transport::smtp::client::TlsParameters::new(host.to_string())
                    .map_err(|err| Error::Unexpected(err.into()))?;
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host)
                .tls(Tls::Wrapper(tls_parameters))
                .port(port)
        }
        _ => AsyncSmtpTransport::<Tokio1Executor>::relay(host)
            .map_err(|err| Error::Unexpected(err.into()))?
            .port(port),
    };

    if let (Some(username), Some(password)) = (
        settings.smtp_username.as_ref(),
        settings.smtp_password.as_ref(),
    ) {
        let creds = Credentials::new(username.to_string(), password.to_string());
        builder = builder.credentials(creds);
    }

    Ok(builder.build())
}

fn build_resume_url(settings: &Settings, realm: &str, resume_path: &str, token: &str) -> String {
    let public_url = settings.server.public_url.trim_end_matches('/');
    let dev_url = settings.ui.dev_url.trim_end_matches('/');
    let base = if dev_url.is_empty() {
        public_url.to_string()
    } else if public_url.contains("localhost") || public_url.contains("127.0.0.1") {
        dev_url.to_string()
    } else {
        public_url.to_string()
    };
    let path = resume_path.trim_start_matches('/');
    format!(
        "{base}/#/{path}?realm={realm}&resume_token={token}",
        base = base,
        path = path,
        realm = urlencoding::encode(realm),
        token = urlencoding::encode(token)
    )
}

fn looks_like_email(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.contains('@') && trimmed.contains('.')
}

fn apply_template(
    template: &str,
    realm: &str,
    identifier: &str,
    token: &str,
    resume_url: &str,
    expires_at: &str,
) -> String {
    template
        .replace("{realm}", realm)
        .replace("{identifier}", identifier)
        .replace("{token}", token)
        .replace("{resume_url}", resume_url)
        .replace("{expires_at}", expires_at)
}
