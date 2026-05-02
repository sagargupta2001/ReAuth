use crate::application::audit_service::AuditService;
use crate::config::Settings;
use crate::domain::audit::NewAuditEvent;
use crate::domain::passkey_challenge::{PasskeyChallenge, PasskeyChallengeKind};
use crate::domain::passkey_runtime::{PASSKEY_REAUTH_AT_KEY, PASSKEY_REAUTH_USER_ID_KEY};
use crate::domain::realm_passkey_settings::RealmPasskeySettings;
use crate::error::{Error, Result};
use crate::ports::auth_session_repository::AuthSessionRepository;
use crate::ports::passkey_challenge_repository::PasskeyChallengeRepository;
use crate::ports::passkey_credential_repository::PasskeyCredentialRepository;
use crate::ports::realm_passkey_settings_repository::RealmPasskeySettingsRepository;
use crate::ports::realm_repository::RealmRepository;
use crate::ports::user_repository::UserRepository;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{Duration, Utc};
use p256::ecdsa::signature::Verifier as P256Verifier;
use p256::ecdsa::{Signature as P256Signature, VerifyingKey as P256VerifyingKey};
use p256::pkcs8::DecodePublicKey as P256DecodePublicKey;
use p384::ecdsa::{Signature as P384Signature, VerifyingKey as P384VerifyingKey};
use rand::RngExt;
use rsa::pkcs1v15::{Signature as RsaPkcs1v15Signature, VerifyingKey as RsaVerifyingKey};
use rsa::pkcs8::DecodePublicKey as RsaDecodePublicKey;
use rsa::sha2::Sha256 as RsaSha256;
use rsa::signature::Verifier as RsaVerifier;
use rsa::RsaPublicKey;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::sync::Arc;
use tracing::warn;
use url::Url;
use uuid::Uuid;

pub struct BeginAssertionRequest {
    pub realm_id: Uuid,
    pub auth_session_id: Uuid,
    pub identifier: Option<String>,
    pub intent: Option<String>,
}

pub struct BeginAssertionResult {
    pub challenge_id: Uuid,
    pub public_key: Value,
    pub fallback_allowed: bool,
}

pub struct VerifyAssertionRequest {
    pub realm_id: Uuid,
    pub challenge_id: Uuid,
    pub credential: Value,
}

pub struct VerifyAssertionResult {
    pub auth_session_id: Uuid,
    pub user_id: Uuid,
    pub credential_id_b64url: String,
}

pub struct BeginEnrollmentRequest {
    pub realm_id: Uuid,
    pub auth_session_id: Uuid,
}

pub struct BeginEnrollmentResult {
    pub challenge_id: Uuid,
    pub public_key: Value,
    pub user_id: Uuid,
}

pub struct VerifyEnrollmentRequest {
    pub realm_id: Uuid,
    pub challenge_id: Uuid,
    pub credential: Value,
    pub friendly_name: Option<String>,
}

pub struct VerifyEnrollmentResult {
    pub auth_session_id: Uuid,
    pub user_id: Uuid,
    pub credential_id_b64url: String,
}

pub struct PasskeyAssertionService {
    session_repo: Arc<dyn AuthSessionRepository>,
    realm_repo: Arc<dyn RealmRepository>,
    user_repo: Arc<dyn UserRepository>,
    challenge_repo: Arc<dyn PasskeyChallengeRepository>,
    credential_repo: Arc<dyn PasskeyCredentialRepository>,
    passkey_settings_repo: Arc<dyn RealmPasskeySettingsRepository>,
    audit_service: Arc<AuditService>,
    settings: Settings,
}

impl PasskeyAssertionService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        session_repo: Arc<dyn AuthSessionRepository>,
        realm_repo: Arc<dyn RealmRepository>,
        user_repo: Arc<dyn UserRepository>,
        challenge_repo: Arc<dyn PasskeyChallengeRepository>,
        credential_repo: Arc<dyn PasskeyCredentialRepository>,
        passkey_settings_repo: Arc<dyn RealmPasskeySettingsRepository>,
        audit_service: Arc<AuditService>,
        settings: Settings,
    ) -> Self {
        Self {
            session_repo,
            realm_repo,
            user_repo,
            challenge_repo,
            credential_repo,
            passkey_settings_repo,
            audit_service,
            settings,
        }
    }

    pub async fn begin_authentication(
        &self,
        request: BeginAssertionRequest,
    ) -> Result<BeginAssertionResult> {
        let session = self
            .session_repo
            .find_by_id(&request.auth_session_id)
            .await?
            .ok_or(Error::InvalidLoginSession)?;
        if session.realm_id != request.realm_id {
            return Err(Error::SecurityViolation(
                "Auth session does not belong to realm".to_string(),
            ));
        }

        let passkey_settings = self.load_passkey_settings(request.realm_id).await?;
        if !passkey_settings.enabled {
            return Err(Error::SecurityViolation(
                "Passkeys are disabled for this realm".to_string(),
            ));
        }

        let rp_id = self.resolve_rp_id()?;
        let allowed_origins = self.resolve_allowed_origins()?;

        let resolved_user = self
            .resolve_user_from_identifier(request.realm_id, request.identifier.as_deref())
            .await?;
        let allow_credentials = if let Some(user) = resolved_user.as_ref() {
            self.credential_repo
                .list_by_user(&request.realm_id, &user.id)
                .await?
        } else {
            Vec::new()
        };
        let allow_credentials_json: Vec<Value> = allow_credentials
            .iter()
            .map(|credential| {
                json!({
                    "type": "public-key",
                    "id": credential.credential_id_b64url
                })
            })
            .collect();

        let challenge_raw = random_challenge();
        let challenge_hash = hash_b64url(&challenge_raw);
        let challenge_kind = match request.intent.as_deref() {
            Some("reauth") => PasskeyChallengeKind::Reauthentication,
            _ => PasskeyChallengeKind::Authentication,
        };
        let expires_at = Utc::now() + Duration::seconds(passkey_settings.challenge_ttl_secs);
        let challenge = PasskeyChallenge::new(
            request.realm_id,
            request.auth_session_id,
            resolved_user.as_ref().map(|value| value.id),
            challenge_kind,
            challenge_hash,
            rp_id.clone(),
            serde_json::to_string(&allowed_origins)
                .map_err(|err| Error::System(format!("Failed to serialize origins: {}", err)))?,
            expires_at,
        );
        self.challenge_repo.create(&challenge).await?;

        let user_verification = if request.intent.as_deref() == Some("reauth") {
            "required"
        } else {
            "preferred"
        };
        let public_key = json!({
            "challenge": challenge_raw,
            "rpId": rp_id,
            "timeout": passkey_settings.challenge_ttl_secs * 1000,
            "userVerification": user_verification,
            "allowCredentials": allow_credentials_json
        });

        Ok(BeginAssertionResult {
            challenge_id: challenge.id,
            public_key,
            fallback_allowed: passkey_settings.allow_password_fallback,
        })
    }

    pub async fn begin_enrollment(
        &self,
        request: BeginEnrollmentRequest,
    ) -> Result<BeginEnrollmentResult> {
        let session = self
            .session_repo
            .find_by_id(&request.auth_session_id)
            .await?
            .ok_or(Error::InvalidLoginSession)?;
        if session.realm_id != request.realm_id {
            return Err(Error::SecurityViolation(
                "Auth session does not belong to realm".to_string(),
            ));
        }

        let passkey_settings = self.load_passkey_settings(request.realm_id).await?;
        if !passkey_settings.enabled {
            return Err(Error::SecurityViolation(
                "Passkeys are disabled for this realm".to_string(),
            ));
        }

        let user_id = self.resolve_session_user_id(&session)?;
        let user = self
            .user_repo
            .find_by_id(&user_id)
            .await?
            .ok_or(Error::UserNotFound)?;
        if user.realm_id != request.realm_id {
            return Err(Error::SecurityViolation(
                "Enrollment user does not belong to realm".to_string(),
            ));
        }

        let rp_id = self.resolve_rp_id()?;
        let allowed_origins = self.resolve_allowed_origins()?;
        let challenge_raw = random_challenge();
        let challenge_hash = hash_b64url(&challenge_raw);
        let expires_at = Utc::now() + Duration::seconds(passkey_settings.challenge_ttl_secs);
        let challenge = PasskeyChallenge::new(
            request.realm_id,
            request.auth_session_id,
            Some(user_id),
            PasskeyChallengeKind::Enrollment,
            challenge_hash,
            rp_id.clone(),
            serde_json::to_string(&allowed_origins)
                .map_err(|err| Error::System(format!("Failed to serialize origins: {}", err)))?,
            expires_at,
        );
        self.challenge_repo.create(&challenge).await?;

        let existing_credentials = self
            .credential_repo
            .list_by_user(&request.realm_id, &user_id)
            .await?;
        let exclude_credentials: Vec<Value> = existing_credentials
            .iter()
            .map(|credential| {
                json!({
                    "type": "public-key",
                    "id": credential.credential_id_b64url
                })
            })
            .collect();

        let user_id_bytes = user.id.as_bytes();
        let user_name = user.username.clone();
        let user_display_name = user.email.clone().unwrap_or_else(|| user_name.clone());
        let public_key = json!({
            "challenge": challenge_raw,
            "rp": {
                "id": rp_id,
                "name": request.realm_id.to_string()
            },
            "user": {
                "id": URL_SAFE_NO_PAD.encode(user_id_bytes),
                "name": user_name,
                "displayName": user_display_name
            },
            "pubKeyCredParams": [
                { "type": "public-key", "alg": -7 },
                { "type": "public-key", "alg": -257 }
            ],
            "timeout": passkey_settings.challenge_ttl_secs * 1000,
            "attestation": "none",
            "authenticatorSelection": {
                "residentKey": if passkey_settings.discoverable_preferred { "preferred" } else { "discouraged" },
                "userVerification": "preferred"
            },
            "excludeCredentials": exclude_credentials
        });

        Ok(BeginEnrollmentResult {
            challenge_id: challenge.id,
            public_key,
            user_id,
        })
    }

    pub async fn verify_authentication(
        &self,
        request: VerifyAssertionRequest,
    ) -> Result<VerifyAssertionResult> {
        let now = Utc::now();
        let consumed = self
            .challenge_repo
            .consume_if_active(&request.challenge_id, &request.realm_id, now)
            .await?
            .ok_or(Error::InvalidActionToken)?;

        let session = self
            .session_repo
            .find_by_id(&consumed.auth_session_id)
            .await?
            .ok_or(Error::InvalidLoginSession)?;
        if session.realm_id != request.realm_id {
            return Err(Error::SecurityViolation(
                "Auth session does not belong to realm".to_string(),
            ));
        }

        let credential_id = extract_credential_id(&request.credential)?;
        let credential = self
            .credential_repo
            .find_by_realm_and_credential_id(&request.realm_id, &credential_id)
            .await?
            .ok_or(Error::InvalidCredentials)?;

        let client_data_json_bytes = extract_client_data_json_bytes(&request.credential)?;
        let client_data = parse_client_data_json(&client_data_json_bytes)?;
        let client_data_challenge = client_data
            .get("challenge")
            .and_then(|value| value.as_str())
            .map(|value| value.to_string())
            .ok_or_else(|| Error::Validation("clientDataJSON.challenge is required".to_string()))?;
        if hash_b64url(&client_data_challenge) != consumed.challenge_hash {
            self.record_audit(
                request.realm_id,
                Some(credential.user_id),
                "passkey.assertion.challenge_mismatch",
                Some(consumed.id.to_string()),
                json!({"credential_id": credential_id}),
            )
            .await;
            return Err(Error::InvalidCredentials);
        }

        let origin = client_data
            .get("origin")
            .and_then(|value| value.as_str())
            .map(|value| value.to_string())
            .ok_or_else(|| Error::Validation("clientDataJSON.origin is required".to_string()))?;
        validate_origin(&origin, &consumed.allowed_origins_json)?;
        validate_client_data_type_value(&client_data, "webauthn.get")?;
        let signature = extract_signature(&request.credential)?;
        if signature.is_empty() {
            return Err(Error::Validation(
                "credential.response.signature is required".to_string(),
            ));
        }

        let auth_data = extract_authenticator_data(&request.credential)?;
        validate_rp_id_hash(&auth_data, &consumed.rp_id)?;
        let parsed_auth_data = parse_auth_data_counter_and_backup(&auth_data)?;
        if !parsed_auth_data.user_present {
            return Err(Error::SecurityViolation(
                "User presence is required for passkey assertion".to_string(),
            ));
        }
        if consumed.challenge_kind == PasskeyChallengeKind::Reauthentication
            && !parsed_auth_data.user_verified
        {
            return Err(Error::SecurityViolation(
                "User verification is required for passkey reauthentication".to_string(),
            ));
        }
        let client_data_hash = Sha256::digest(&client_data_json_bytes);
        let mut signed_data = Vec::with_capacity(auth_data.len() + client_data_hash.len());
        signed_data.extend_from_slice(&auth_data);
        signed_data.extend_from_slice(&client_data_hash);
        if let Err(err) =
            verify_webauthn_signature(&credential.public_key_cose_b64url, &signed_data, &signature)
        {
            self.record_audit(
                request.realm_id,
                Some(credential.user_id),
                "passkey.assertion.invalid_signature",
                Some(credential.id.to_string()),
                json!({"credential_id": credential_id}),
            )
            .await;
            return Err(err);
        }

        let updated = self
            .credential_repo
            .touch_assertion_state(
                &credential.id,
                parsed_auth_data.sign_count,
                parsed_auth_data.backed_up,
                now,
            )
            .await?;
        if !updated {
            self.record_audit(
                request.realm_id,
                Some(credential.user_id),
                "passkey.assertion.counter_regression",
                Some(credential.id.to_string()),
                json!({"credential_id": credential_id, "observed_sign_count": parsed_auth_data.sign_count}),
            )
            .await;
            return Err(Error::SecurityViolation(
                "Passkey sign counter regression detected".to_string(),
            ));
        }

        if consumed.challenge_kind == PasskeyChallengeKind::Reauthentication {
            let mut session = session;
            session.update_context(PASSKEY_REAUTH_AT_KEY, json!(now.to_rfc3339()));
            session.update_context(
                PASSKEY_REAUTH_USER_ID_KEY,
                json!(credential.user_id.to_string()),
            );
            self.session_repo.update(&session).await?;
        }

        self.record_audit(
            request.realm_id,
            Some(credential.user_id),
            "passkey.assertion.success",
            Some(credential.id.to_string()),
            json!({"credential_id": credential_id}),
        )
        .await;

        Ok(VerifyAssertionResult {
            auth_session_id: consumed.auth_session_id,
            user_id: credential.user_id,
            credential_id_b64url: credential.credential_id_b64url,
        })
    }

    pub async fn verify_enrollment(
        &self,
        request: VerifyEnrollmentRequest,
    ) -> Result<VerifyEnrollmentResult> {
        let now = Utc::now();
        let consumed = self
            .challenge_repo
            .consume_if_active(&request.challenge_id, &request.realm_id, now)
            .await?
            .ok_or(Error::InvalidActionToken)?;

        if consumed.challenge_kind != PasskeyChallengeKind::Enrollment {
            return Err(Error::SecurityViolation(
                "Passkey challenge kind mismatch".to_string(),
            ));
        }

        let session = self
            .session_repo
            .find_by_id(&consumed.auth_session_id)
            .await?
            .ok_or(Error::InvalidLoginSession)?;
        if session.realm_id != request.realm_id {
            return Err(Error::SecurityViolation(
                "Auth session does not belong to realm".to_string(),
            ));
        }

        let user_id = consumed
            .user_id
            .or(session.user_id)
            .or_else(|| self.resolve_session_user_id(&session).ok())
            .ok_or_else(|| {
                Error::Validation("Passkey enrollment requires an authenticated user".to_string())
            })?;
        let user = self
            .user_repo
            .find_by_id(&user_id)
            .await?
            .ok_or(Error::UserNotFound)?;
        if user.realm_id != request.realm_id {
            return Err(Error::SecurityViolation(
                "Enrollment user does not belong to realm".to_string(),
            ));
        }

        let credential_id_b64url = extract_credential_id(&request.credential)?;
        if self
            .credential_repo
            .find_by_realm_and_credential_id(&request.realm_id, &credential_id_b64url)
            .await?
            .is_some()
        {
            return Err(Error::Validation(
                "Passkey credential already exists for realm".to_string(),
            ));
        }

        let client_data_json_bytes = extract_client_data_json_bytes(&request.credential)?;
        let client_data = parse_client_data_json(&client_data_json_bytes)?;
        let client_data_challenge = client_data
            .get("challenge")
            .and_then(|value| value.as_str())
            .map(|value| value.to_string())
            .ok_or_else(|| Error::Validation("clientDataJSON.challenge is required".to_string()))?;
        if hash_b64url(&client_data_challenge) != consumed.challenge_hash {
            self.record_audit(
                request.realm_id,
                Some(user_id),
                "passkey.enrollment.challenge_mismatch",
                Some(consumed.id.to_string()),
                json!({ "credential_id": credential_id_b64url }),
            )
            .await;
            return Err(Error::InvalidCredentials);
        }

        let origin = client_data
            .get("origin")
            .and_then(|value| value.as_str())
            .map(|value| value.to_string())
            .ok_or_else(|| Error::Validation("clientDataJSON.origin is required".to_string()))?;
        validate_origin(&origin, &consumed.allowed_origins_json)?;
        validate_client_data_type_value(&client_data, "webauthn.create")?;

        let auth_data = extract_authenticator_data(&request.credential)?;
        validate_rp_id_hash(&auth_data, &consumed.rp_id)?;
        let parsed_auth_data = parse_auth_data_counter_and_backup(&auth_data)?;
        if !parsed_auth_data.user_present {
            return Err(Error::SecurityViolation(
                "User presence is required for passkey enrollment".to_string(),
            ));
        }
        if !has_attested_credential_data(&auth_data) {
            return Err(Error::Validation(
                "Missing attested credential data in enrollment response".to_string(),
            ));
        }
        if let Some(parsed_credential_id) = extract_attested_credential_id(&auth_data)? {
            if parsed_credential_id != credential_id_b64url {
                return Err(Error::SecurityViolation(
                    "Passkey credential id mismatch".to_string(),
                ));
            }
        }

        let public_key_spki = extract_registration_public_key(&request.credential)?;
        if public_key_spki.is_empty() {
            return Err(Error::Validation(
                "credential.response.publicKey is required".to_string(),
            ));
        }

        let transports = extract_transports(&request.credential);
        let aaguid = extract_attested_aaguid(&auth_data)?;
        let mut credential = crate::domain::passkey_credential::PasskeyCredential::new(
            request.realm_id,
            user_id,
            credential_id_b64url.clone(),
            public_key_spki,
        );
        credential.sign_count = parsed_auth_data.sign_count;
        credential.backed_up = parsed_auth_data.backed_up;
        credential.backup_eligible = parsed_auth_data.backup_eligible;
        credential.transports_json = serde_json::to_string(&transports)
            .map_err(|err| Error::System(format!("Failed to serialize transports: {}", err)))?;
        credential.aaguid = aaguid;
        credential.friendly_name = request
            .friendly_name
            .map(trim_to_option)
            .transpose()?
            .flatten();

        self.credential_repo.create(&credential).await?;
        self.record_audit(
            request.realm_id,
            Some(user_id),
            "passkey.enrollment.success",
            Some(credential.id.to_string()),
            json!({
                "credential_id": credential_id_b64url,
                "backup_eligible": credential.backup_eligible,
                "backed_up": credential.backed_up
            }),
        )
        .await;

        Ok(VerifyEnrollmentResult {
            auth_session_id: consumed.auth_session_id,
            user_id,
            credential_id_b64url,
        })
    }

    async fn load_passkey_settings(&self, realm_id: Uuid) -> Result<RealmPasskeySettings> {
        if self.realm_repo.find_by_id(&realm_id).await?.is_none() {
            return Err(Error::RealmNotFound(realm_id.to_string()));
        }
        Ok(self
            .passkey_settings_repo
            .find_by_realm_id(&realm_id)
            .await?
            .unwrap_or_else(|| RealmPasskeySettings::defaults(realm_id)))
    }

    async fn resolve_user_from_identifier(
        &self,
        realm_id: Uuid,
        identifier: Option<&str>,
    ) -> Result<Option<crate::domain::user::User>> {
        let Some(identifier) = identifier.map(str::trim).filter(|value| !value.is_empty()) else {
            return Ok(None);
        };

        if identifier.contains('@') {
            if let Some(user) = self.user_repo.find_by_email(&realm_id, identifier).await? {
                return Ok(Some(user));
            }
        }

        self.user_repo.find_by_username(&realm_id, identifier).await
    }

    fn resolve_session_user_id(
        &self,
        session: &crate::domain::auth_session::AuthenticationSession,
    ) -> Result<Uuid> {
        if let Some(user_id) = session.user_id {
            return Ok(user_id);
        }

        let user_id = session
            .context
            .get("user_id")
            .and_then(|value| value.as_str())
            .ok_or_else(|| {
                Error::Validation("Passkey operation requires an authenticated user".to_string())
            })?;

        Uuid::parse_str(user_id).map_err(Error::from)
    }

    fn resolve_rp_id(&self) -> Result<String> {
        let public_url = self.settings.server.public_url.trim();
        let parsed = Url::parse(public_url)
            .map_err(|err| Error::Config(config::ConfigError::Message(err.to_string())))?;
        let host = parsed
            .host_str()
            .ok_or_else(|| Error::Validation("server.public_url host is required".to_string()))?;
        Ok(host.to_string())
    }

    fn resolve_allowed_origins(&self) -> Result<Vec<String>> {
        let mut origins: BTreeSet<String> = BTreeSet::new();
        origins.insert(origin_from_url(self.settings.server.public_url.trim())?);
        for origin in &self.settings.cors.allowed_origins {
            let trimmed = origin.trim();
            if trimmed.is_empty() {
                continue;
            }
            origins.insert(origin_from_url(trimmed)?);
        }
        Ok(origins.into_iter().collect())
    }

    async fn record_audit(
        &self,
        realm_id: Uuid,
        actor_user_id: Option<Uuid>,
        action: &str,
        target_id: Option<String>,
        metadata: Value,
    ) {
        if let Err(err) = self
            .audit_service
            .record(NewAuditEvent {
                realm_id,
                actor_user_id,
                action: action.to_string(),
                target_type: "passkey".to_string(),
                target_id,
                metadata,
            })
            .await
        {
            warn!("Failed to write passkey audit event: {}", err);
        }
    }
}

fn random_challenge() -> String {
    let mut bytes = [0_u8; 32];
    rand::rng().fill(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn origin_from_url(value: &str) -> Result<String> {
    let parsed =
        Url::parse(value).map_err(|err| Error::Validation(format!("Invalid URL: {}", err)))?;
    let host = parsed
        .host_str()
        .ok_or_else(|| Error::Validation("URL host is required".to_string()))?;
    let mut origin = format!("{}://{}", parsed.scheme(), host);
    if let Some(port) = parsed.port() {
        origin.push(':');
        origin.push_str(&port.to_string());
    }
    Ok(origin)
}

fn hash_b64url(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let digest = hasher.finalize();
    URL_SAFE_NO_PAD.encode(digest)
}

fn decode_b64url(value: &str) -> Result<Vec<u8>> {
    URL_SAFE_NO_PAD
        .decode(value.as_bytes())
        .map_err(|_| Error::Validation("Invalid base64url payload".to_string()))
}

fn extract_credential_id(credential: &Value) -> Result<String> {
    credential
        .get("id")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| Error::Validation("credential.id is required".to_string()))
}

fn extract_client_data_json_bytes(credential: &Value) -> Result<Vec<u8>> {
    let encoded = credential
        .get("response")
        .and_then(|value| value.get("clientDataJSON"))
        .and_then(|value| value.as_str())
        .ok_or_else(|| {
            Error::Validation("credential.response.clientDataJSON is required".to_string())
        })?;
    decode_b64url(encoded)
}

fn parse_client_data_json(bytes: &[u8]) -> Result<Value> {
    serde_json::from_slice(bytes)
        .map_err(|_| Error::Validation("Invalid clientDataJSON".to_string()))
}

fn validate_client_data_type_value(client_data: &Value, expected: &str) -> Result<()> {
    let value = client_data
        .get("type")
        .and_then(|item| item.as_str())
        .ok_or_else(|| Error::Validation("clientDataJSON.type is required".to_string()))?;
    if value != expected {
        return Err(Error::Validation(
            "Invalid WebAuthn clientData type".to_string(),
        ));
    }
    Ok(())
}

fn extract_authenticator_data(credential: &Value) -> Result<Vec<u8>> {
    let encoded = credential
        .get("response")
        .and_then(|value| value.get("authenticatorData"))
        .and_then(|value| value.as_str())
        .ok_or_else(|| {
            Error::Validation("credential.response.authenticatorData is required".to_string())
        })?;
    decode_b64url(encoded)
}

fn extract_signature(credential: &Value) -> Result<Vec<u8>> {
    let encoded = credential
        .get("response")
        .and_then(|value| value.get("signature"))
        .and_then(|value| value.as_str())
        .ok_or_else(|| {
            Error::Validation("credential.response.signature is required".to_string())
        })?;
    decode_b64url(encoded)
}

fn extract_registration_public_key(credential: &Value) -> Result<String> {
    credential
        .get("response")
        .and_then(|value| value.get("publicKey"))
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .ok_or_else(|| Error::Validation("credential.response.publicKey is required".to_string()))
}

fn extract_transports(credential: &Value) -> Vec<String> {
    credential
        .get("response")
        .and_then(|value| value.get("transports"))
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|value| value.as_str())
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn validate_origin(origin: &str, allowed_origins_json: &str) -> Result<()> {
    let origins: Vec<String> = serde_json::from_str(allowed_origins_json)
        .map_err(|_| Error::System("Stored passkey challenge origins are corrupted".to_string()))?;
    if origins.iter().any(|candidate| candidate == origin) {
        return Ok(());
    }
    Err(Error::SecurityViolation(
        "Assertion origin is not allowed".to_string(),
    ))
}

fn validate_rp_id_hash(auth_data: &[u8], rp_id: &str) -> Result<()> {
    if auth_data.len() < 37 {
        return Err(Error::Validation(
            "authenticatorData is too short".to_string(),
        ));
    }
    let mut hasher = Sha256::new();
    hasher.update(rp_id.as_bytes());
    let expected = hasher.finalize();
    if auth_data[0..32] != expected[..] {
        return Err(Error::SecurityViolation("RP ID hash mismatch".to_string()));
    }
    Ok(())
}

struct ParsedAuthData {
    sign_count: i64,
    backed_up: bool,
    backup_eligible: bool,
    user_present: bool,
    user_verified: bool,
}

fn parse_auth_data_counter_and_backup(auth_data: &[u8]) -> Result<ParsedAuthData> {
    if auth_data.len() < 37 {
        return Err(Error::Validation(
            "authenticatorData is too short".to_string(),
        ));
    }
    let flags = auth_data[32];
    let backed_up = (flags & 0b0001_0000) != 0;
    let backup_eligible = (flags & 0b0000_1000) != 0;
    let user_present = (flags & 0b0000_0001) != 0;
    let user_verified = (flags & 0b0000_0100) != 0;
    let counter =
        u32::from_be_bytes([auth_data[33], auth_data[34], auth_data[35], auth_data[36]]) as i64;
    Ok(ParsedAuthData {
        sign_count: counter,
        backed_up,
        backup_eligible,
        user_present,
        user_verified,
    })
}

fn has_attested_credential_data(auth_data: &[u8]) -> bool {
    if auth_data.len() < 37 {
        return false;
    }
    let flags = auth_data[32];
    (flags & 0b0100_0000) != 0
}

fn extract_attested_credential_id(auth_data: &[u8]) -> Result<Option<String>> {
    if !has_attested_credential_data(auth_data) {
        return Ok(None);
    }
    if auth_data.len() < 55 {
        return Err(Error::Validation(
            "attested credential data is too short".to_string(),
        ));
    }
    let credential_len = u16::from_be_bytes([auth_data[53], auth_data[54]]) as usize;
    let start = 55;
    let end = start + credential_len;
    if auth_data.len() < end {
        return Err(Error::Validation(
            "Invalid attested credential length".to_string(),
        ));
    }
    Ok(Some(URL_SAFE_NO_PAD.encode(&auth_data[start..end])))
}

fn extract_attested_aaguid(auth_data: &[u8]) -> Result<Option<String>> {
    if !has_attested_credential_data(auth_data) {
        return Ok(None);
    }
    if auth_data.len() < 53 {
        return Err(Error::Validation(
            "attested credential data is too short".to_string(),
        ));
    }
    let aaguid = &auth_data[37..53];
    if aaguid.iter().all(|byte| *byte == 0) {
        return Ok(None);
    }
    let encoded = hex::encode(aaguid);
    Ok(Some(format!(
        "{}-{}-{}-{}-{}",
        &encoded[0..8],
        &encoded[8..12],
        &encoded[12..16],
        &encoded[16..20],
        &encoded[20..32]
    )))
}

fn verify_webauthn_signature(
    public_key_spki_b64url: &str,
    signed_data: &[u8],
    signature: &[u8],
) -> Result<()> {
    let spki = decode_b64url(public_key_spki_b64url)?;

    if let Ok(key) = P256VerifyingKey::from_public_key_der(&spki) {
        let parsed_signature =
            P256Signature::from_der(signature).map_err(|_| Error::InvalidCredentials)?;
        key.verify(signed_data, &parsed_signature)
            .map_err(|_| Error::InvalidCredentials)?;
        return Ok(());
    }

    if let Ok(key) = P384VerifyingKey::from_public_key_der(&spki) {
        let parsed_signature =
            P384Signature::from_der(signature).map_err(|_| Error::InvalidCredentials)?;
        key.verify(signed_data, &parsed_signature)
            .map_err(|_| Error::InvalidCredentials)?;
        return Ok(());
    }

    if let Ok(key) = RsaPublicKey::from_public_key_der(&spki) {
        let verifier = RsaVerifyingKey::<RsaSha256>::new(key);
        let parsed_signature =
            RsaPkcs1v15Signature::try_from(signature).map_err(|_| Error::InvalidCredentials)?;
        verifier
            .verify(signed_data, &parsed_signature)
            .map_err(|_| Error::InvalidCredentials)?;
        return Ok(());
    }

    Err(Error::Validation(
        "Unsupported passkey public key algorithm".to_string(),
    ))
}

fn trim_to_option(value: String) -> Result<Option<String>> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.len() > 64 {
        return Err(Error::Validation(
            "Passkey friendly name must be 64 characters or fewer".to_string(),
        ));
    }
    Ok(Some(trimmed.to_string()))
}
