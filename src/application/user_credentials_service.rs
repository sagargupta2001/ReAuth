use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

use crate::application::audit_service::AuditService;
use crate::application::user_service::UserService;
use crate::domain::audit::NewAuditEvent;
use crate::domain::realm_passkey_settings::RealmPasskeySettings;
use crate::error::{Error, Result};
use crate::ports::federated_identity_repository::FederatedIdentityRepository;
use crate::ports::identity_provider_repository::IdentityProviderRepository;
use crate::ports::passkey_credential_repository::PasskeyCredentialRepository;
use crate::ports::realm_passkey_settings_repository::RealmPasskeySettingsRepository;
use crate::ports::realm_repository::RealmRepository;

#[derive(Debug, Clone, Serialize)]
pub struct UserPasswordCredentialSummary {
    pub configured: bool,
    pub force_reset_on_next_login: bool,
    pub password_login_disabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserPasskeyCredentialSummary {
    pub id: Uuid,
    pub credential_id_b64url: String,
    pub friendly_name: Option<String>,
    pub backed_up: bool,
    pub backup_eligible: bool,
    pub sign_count: i64,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserFederatedIdentitySummary {
    pub id: Uuid,
    pub provider_alias: String,
    pub provider_display_name: String,
    pub subject: String,
    pub external_email: Option<String>,
    pub linked_via: String,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserCredentialsSummary {
    pub user_id: Uuid,
    pub password: UserPasswordCredentialSummary,
    pub passkeys: Vec<UserPasskeyCredentialSummary>,
    pub federated_identities: Vec<UserFederatedIdentitySummary>,
}

pub struct UserCredentialsService {
    user_service: Arc<UserService>,
    passkey_credential_repo: Arc<dyn PasskeyCredentialRepository>,
    passkey_settings_repo: Arc<dyn RealmPasskeySettingsRepository>,
    realm_repo: Arc<dyn RealmRepository>,
    federated_identity_repo: Arc<dyn FederatedIdentityRepository>,
    identity_provider_repo: Arc<dyn IdentityProviderRepository>,
    audit_service: Arc<AuditService>,
}

impl UserCredentialsService {
    pub fn new(
        user_service: Arc<UserService>,
        passkey_credential_repo: Arc<dyn PasskeyCredentialRepository>,
        passkey_settings_repo: Arc<dyn RealmPasskeySettingsRepository>,
        realm_repo: Arc<dyn RealmRepository>,
        federated_identity_repo: Arc<dyn FederatedIdentityRepository>,
        identity_provider_repo: Arc<dyn IdentityProviderRepository>,
        audit_service: Arc<AuditService>,
    ) -> Self {
        Self {
            user_service,
            passkey_credential_repo,
            passkey_settings_repo,
            realm_repo,
            federated_identity_repo,
            identity_provider_repo,
            audit_service,
        }
    }

    pub async fn list_credentials(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
    ) -> Result<UserCredentialsSummary> {
        let user = self
            .user_service
            .get_user_in_realm(realm_id, user_id)
            .await?;
        let passkeys = self
            .passkey_credential_repo
            .list_by_user(&realm_id, &user.id)
            .await?;
        let federated_identities = self
            .federated_identity_repo
            .list_by_user(&realm_id, &user.id)
            .await?;
        let providers = self
            .identity_provider_repo
            .list_by_realm(&realm_id)
            .await?
            .into_iter()
            .map(|provider| (provider.id, provider))
            .collect::<std::collections::HashMap<_, _>>();

        let passkeys = passkeys
            .into_iter()
            .map(|credential| UserPasskeyCredentialSummary {
                id: credential.id,
                credential_id_b64url: credential.credential_id_b64url,
                friendly_name: credential.friendly_name,
                backed_up: credential.backed_up,
                backup_eligible: credential.backup_eligible,
                sign_count: credential.sign_count,
                created_at: credential.created_at,
                last_used_at: credential.last_used_at,
            })
            .collect();
        let federated_identities = federated_identities
            .into_iter()
            .map(|identity| {
                let provider = providers.get(&identity.provider_id);
                UserFederatedIdentitySummary {
                    id: identity.id,
                    provider_alias: provider
                        .map(|provider| provider.alias.clone())
                        .unwrap_or_else(|| "unknown".to_string()),
                    provider_display_name: provider
                        .map(|provider| provider.display_name.clone())
                        .unwrap_or_else(|| "Unknown provider".to_string()),
                    subject: identity.subject,
                    external_email: identity.external_email,
                    linked_via: identity.linked_via,
                    last_login_at: identity.last_login_at,
                }
            })
            .collect();

        Ok(UserCredentialsSummary {
            user_id: user.id,
            password: UserPasswordCredentialSummary {
                configured: !user.hashed_password.trim().is_empty(),
                force_reset_on_next_login: user.force_password_reset,
                password_login_disabled: user.password_login_disabled,
            },
            passkeys,
            federated_identities,
        })
    }

    pub async fn update_password(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        new_password: &str,
    ) -> Result<()> {
        self.user_service
            .update_password(realm_id, user_id, new_password)
            .await?;
        Ok(())
    }

    pub async fn revoke_passkey(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        credential_id: Uuid,
    ) -> Result<()> {
        // Ensure the user belongs to this realm.
        self.user_service
            .get_user_in_realm(realm_id, user_id)
            .await?;
        let deleted = self
            .passkey_credential_repo
            .delete_by_id_for_user(&realm_id, &user_id, &credential_id)
            .await?;
        if !deleted {
            return Err(Error::NotFound("Passkey credential not found".to_string()));
        }
        Ok(())
    }

    pub async fn rename_passkey(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        credential_id: Uuid,
        friendly_name: Option<String>,
    ) -> Result<()> {
        self.user_service
            .get_user_in_realm(realm_id, user_id)
            .await?;
        let updated = self
            .passkey_credential_repo
            .update_friendly_name_for_user(&realm_id, &user_id, &credential_id, friendly_name)
            .await?;
        if !updated {
            return Err(Error::NotFound("Passkey credential not found".to_string()));
        }
        Ok(())
    }

    pub async fn unlink_federated_identity(
        &self,
        realm_id: Uuid,
        actor_user_id: Option<Uuid>,
        user_id: Uuid,
        federated_identity_id: Uuid,
    ) -> Result<()> {
        let user = self
            .user_service
            .get_user_in_realm(realm_id, user_id)
            .await?;
        let realm = self
            .realm_repo
            .find_by_id(&realm_id)
            .await?
            .ok_or_else(|| Error::NotFound("Realm not found".to_string()))?;
        let passkeys = self
            .passkey_credential_repo
            .list_by_user(&realm_id, &user_id)
            .await?;
        let federated_identities = self
            .federated_identity_repo
            .list_by_user(&realm_id, &user_id)
            .await?;
        let target = federated_identities
            .iter()
            .find(|identity| identity.id == federated_identity_id)
            .cloned()
            .ok_or_else(|| Error::NotFound("Federated identity not found".to_string()))?;

        let remaining_federated_count = federated_identities
            .iter()
            .filter(|identity| identity.id != federated_identity_id)
            .count();
        let has_password_login =
            !user.hashed_password.trim().is_empty() && !user.password_login_disabled;

        // Prevent orphaning the account by removing its final usable sign-in method.
        if realm.idp_minimum_remaining_factor
            && !has_password_login
            && passkeys.is_empty()
            && remaining_federated_count == 0
        {
            return Err(Error::Conflict(
                "Cannot unlink the last sign-in method for this user. Configure a password or passkey first.".to_string(),
            ));
        }

        let deleted = self
            .federated_identity_repo
            .delete_by_id_for_user(&realm_id, &user_id, &federated_identity_id)
            .await?;
        if !deleted {
            return Err(Error::NotFound("Federated identity not found".to_string()));
        }

        let provider = self
            .identity_provider_repo
            .find_by_id(&target.provider_id)
            .await?;
        self.audit_service
            .record(NewAuditEvent {
                realm_id,
                actor_user_id,
                action: "idp_user_unlinked".to_string(),
                target_type: "identity_provider".to_string(),
                target_id: Some(target.provider_id.to_string()),
                metadata: json!({
                    "user_id": user_id,
                    "federated_identity_id": target.id,
                    "provider_alias": provider.as_ref().map(|provider| provider.alias.clone()),
                    "subject": target.subject,
                    "linked_via": target.linked_via,
                }),
            })
            .await?;

        Ok(())
    }

    pub async fn update_password_policy(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        force_reset_on_next_login: Option<bool>,
        password_login_disabled: Option<bool>,
    ) -> Result<()> {
        if force_reset_on_next_login.is_none() && password_login_disabled.is_none() {
            return Err(Error::Validation(
                "No password policy updates provided".to_string(),
            ));
        }

        if password_login_disabled == Some(true) {
            self.validate_disable_password_login_policy(realm_id, user_id)
                .await?;
        }

        self.user_service
            .update_credential_policy(
                realm_id,
                user_id,
                force_reset_on_next_login,
                password_login_disabled,
            )
            .await?;
        Ok(())
    }

    async fn validate_disable_password_login_policy(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
    ) -> Result<()> {
        let settings = self
            .passkey_settings_repo
            .find_by_realm_id(&realm_id)
            .await?
            .unwrap_or_else(|| RealmPasskeySettings::defaults(realm_id));
        if !settings.enabled {
            return Err(Error::Validation(
                "Cannot disable password login unless realm passkeys are enabled".to_string(),
            ));
        }

        let passkeys = self
            .passkey_credential_repo
            .list_by_user(&realm_id, &user_id)
            .await?;
        if passkeys.is_empty() {
            return Err(Error::Validation(
                "Cannot disable password login for a user with no enrolled passkeys".to_string(),
            ));
        }

        Ok(())
    }
}
