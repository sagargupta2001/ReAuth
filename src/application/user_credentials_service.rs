use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::application::user_service::UserService;
use crate::domain::realm_passkey_settings::RealmPasskeySettings;
use crate::error::{Error, Result};
use crate::ports::passkey_credential_repository::PasskeyCredentialRepository;
use crate::ports::realm_passkey_settings_repository::RealmPasskeySettingsRepository;

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
pub struct UserCredentialsSummary {
    pub user_id: Uuid,
    pub password: UserPasswordCredentialSummary,
    pub passkeys: Vec<UserPasskeyCredentialSummary>,
}

pub struct UserCredentialsService {
    user_service: Arc<UserService>,
    passkey_credential_repo: Arc<dyn PasskeyCredentialRepository>,
    passkey_settings_repo: Arc<dyn RealmPasskeySettingsRepository>,
}

impl UserCredentialsService {
    pub fn new(
        user_service: Arc<UserService>,
        passkey_credential_repo: Arc<dyn PasskeyCredentialRepository>,
        passkey_settings_repo: Arc<dyn RealmPasskeySettingsRepository>,
    ) -> Self {
        Self {
            user_service,
            passkey_credential_repo,
            passkey_settings_repo,
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

        Ok(UserCredentialsSummary {
            user_id: user.id,
            password: UserPasswordCredentialSummary {
                configured: !user.hashed_password.trim().is_empty(),
                force_reset_on_next_login: user.force_password_reset,
                password_login_disabled: user.password_login_disabled,
            },
            passkeys,
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
