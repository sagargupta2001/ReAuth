use std::sync::Arc;
use uuid::Uuid;

use crate::domain::crypto::HashedPassword;
use crate::domain::events::{DomainEvent, UserCreated};
use crate::domain::pagination::{PageRequest, PageResponse};
use crate::domain::user_email::UserEmail;
use crate::ports::event_bus::EventPublisher;
use crate::ports::outbox_repository::OutboxRepository;
use crate::ports::transaction_manager::TransactionManager;
use crate::ports::user_email_repository::UserEmailRepository;
use crate::{
    domain::user::{User, UserListFilters, EMPTY_METADATA_JSON},
    error::{Error, Result},
    ports::user_repository::UserRepository,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

pub const USER_METADATA_MAX_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserMetadataVisibility {
    Public,
    Private,
    Unsafe,
}

#[derive(Debug, Serialize)]
pub struct AdminUserMetadataResponse {
    pub public_metadata: Value,
    pub private_metadata: Value,
    pub unsafe_metadata: Value,
}

#[derive(Debug, Serialize)]
pub struct SelfUserMetadataResponse {
    pub public_metadata: Value,
    pub unsafe_metadata: Value,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct UserMetadataUpdateResponse {
    pub public_metadata: Value,
    pub private_metadata: Value,
    pub unsafe_metadata: Value,
    pub updated_at: Option<DateTime<Utc>>,
}

pub struct UserService {
    user_repo: Arc<dyn UserRepository>,
    user_email_repo: Arc<dyn UserEmailRepository>,
    event_bus: Arc<dyn EventPublisher>,
    outbox_repo: Arc<dyn OutboxRepository>,
    tx_manager: Arc<dyn TransactionManager>,
}

impl UserService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        user_email_repo: Arc<dyn UserEmailRepository>,
        event_bus: Arc<dyn EventPublisher>,
        outbox_repo: Arc<dyn OutboxRepository>,
        tx_manager: Arc<dyn TransactionManager>,
    ) -> Self {
        Self {
            user_repo,
            user_email_repo,
            event_bus,
            outbox_repo,
            tx_manager,
        }
    }

    /// Create a new user. If `email` is supplied it is stored as the primary email
    /// in `user_emails` within the same transaction.
    pub async fn create_user(
        &self,
        realm_id: Uuid,
        username: &str,
        password: &str,
        email: Option<&str>,
        _ignore_password_policies: bool,
    ) -> Result<User> {
        if self
            .user_repo
            .find_by_username(&realm_id, username)
            .await?
            .is_some()
        {
            return Err(Error::UsernameAlreadyExists);
        }

        let normalized_email = normalize_optional_email(email);
        if let Some(email_value) = normalized_email.as_deref() {
            if self
                .user_email_repo
                .find_by_email(&realm_id, email_value)
                .await?
                .is_some()
            {
                return Err(Error::EmailAlreadyExists);
            }
        }

        let hashed_password = HashedPassword::new(password)?;

        let user = User {
            id: Uuid::new_v4(),
            realm_id,
            username: username.to_string(),
            first_name: None,
            last_name: None,
            hashed_password: hashed_password.as_str().to_string(),
            public_metadata_json: EMPTY_METADATA_JSON.to_string(),
            private_metadata_json: EMPTY_METADATA_JSON.to_string(),
            unsafe_metadata_json: EMPTY_METADATA_JSON.to_string(),
            force_password_reset: false,
            password_login_disabled: false,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
            last_sign_in_at: None,
            locked_until: None,
            banned_at: None,
        };

        let event = DomainEvent::UserCreated(UserCreated {
            user_id: user.id,
            username: user.username.clone(),
        });

        let mut tx = self.tx_manager.begin().await?;
        let result: Result<()> = async {
            self.user_repo.save(&user, Some(&mut *tx)).await?;

            if let Some(email_val) = normalized_email.as_deref() {
                let user_email =
                    UserEmail::new(user.id, realm_id, email_val.to_string(), true, false);
                self.user_email_repo
                    .save(&user_email, Some(&mut *tx))
                    .await?;
            }

            let envelope = event.to_envelope(Uuid::new_v4(), Utc::now(), Some(realm_id), None);
            self.outbox_repo.insert(&envelope, Some(&mut *tx)).await?;
            Ok(())
        }
        .await;

        match result {
            Ok(()) => {
                self.tx_manager.commit(tx).await?;
                self.event_bus.publish(event).await;
            }
            Err(err) => {
                self.tx_manager.rollback(tx).await?;
                return Err(err);
            }
        }

        Ok(user)
    }

    pub async fn list_users(
        &self,
        realm_id: Uuid,
        req: PageRequest,
        filters: UserListFilters,
    ) -> Result<PageResponse<User>> {
        self.user_repo.list(&realm_id, &req, &filters).await
    }

    pub async fn find_by_username(&self, realm_id: &Uuid, username: &str) -> Result<Option<User>> {
        self.user_repo.find_by_username(realm_id, username).await
    }

    pub async fn find_by_identifier(
        &self,
        realm_id: &Uuid,
        identifier: &str,
    ) -> Result<Option<User>> {
        if let Some(user) = self
            .user_repo
            .find_by_username(realm_id, identifier)
            .await?
        {
            return Ok(Some(user));
        }
        self.user_repo.find_by_email(realm_id, identifier).await
    }

    pub async fn find_by_email(&self, realm_id: &Uuid, email: &str) -> Result<Option<User>> {
        self.user_repo.find_by_email(realm_id, email).await
    }

    pub async fn update_last_sign_in(&self, realm_id: Uuid, user_id: Uuid) -> Result<()> {
        let mut user = self.get_user_in_realm(realm_id, user_id).await?;
        user.last_sign_in_at = Some(Utc::now());
        self.user_repo.update(&user, None).await?;
        Ok(())
    }

    pub async fn count_users_in_realm(&self, realm_id: Uuid) -> Result<i64> {
        self.user_repo.count_in_realm(&realm_id).await
    }

    pub async fn get_user(&self, id: Uuid) -> Result<User> {
        self.user_repo
            .find_by_id(&id)
            .await?
            .ok_or(Error::UserNotFound)
    }

    pub async fn update_username(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        new_username: String,
    ) -> Result<User> {
        self.update_profile(realm_id, user_id, Some(new_username), None, None)
            .await
    }

    /// Update mutable profile fields. Emails and phone numbers are managed via sub-resource services.
    pub async fn update_profile(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        new_username: Option<String>,
        new_first_name: Option<Option<String>>,
        new_last_name: Option<Option<String>>,
    ) -> Result<User> {
        let mut user = self.get_user_in_realm(realm_id, user_id).await?;
        let mut changed = false;

        if let Some(username) = new_username {
            if user.username != username {
                if self
                    .user_repo
                    .find_by_username(&realm_id, &username)
                    .await?
                    .is_some()
                {
                    return Err(Error::UserAlreadyExists);
                }
                user.username = username;
                changed = true;
            }
        }

        if let Some(first_name) = new_first_name {
            let first_name = normalize_optional_profile_text(first_name);
            if user.first_name != first_name {
                user.first_name = first_name;
                changed = true;
            }
        }

        if let Some(last_name) = new_last_name {
            let last_name = normalize_optional_profile_text(last_name);
            if user.last_name != last_name {
                user.last_name = last_name;
                changed = true;
            }
        }

        if changed {
            user.updated_at = Some(Utc::now());
            self.user_repo.update(&user, None).await?;
        }
        Ok(user)
    }

    pub async fn get_user_in_realm(&self, realm_id: Uuid, user_id: Uuid) -> Result<User> {
        let user = self
            .user_repo
            .find_by_id(&user_id)
            .await?
            .ok_or(Error::UserNotFound)?;

        if user.realm_id != realm_id {
            return Err(Error::UserNotFound);
        }

        Ok(user)
    }

    pub async fn get_admin_metadata(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        include_private: bool,
    ) -> Result<AdminUserMetadataResponse> {
        let user = self.get_user_in_realm(realm_id, user_id).await?;
        Ok(admin_metadata_response(&user, include_private))
    }

    pub async fn get_self_metadata(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
    ) -> Result<SelfUserMetadataResponse> {
        let user = self.get_user_in_realm(realm_id, user_id).await?;
        Ok(SelfUserMetadataResponse {
            public_metadata: parse_metadata_json(&user.public_metadata_json),
            unsafe_metadata: parse_metadata_json(&user.unsafe_metadata_json),
            updated_at: user.updated_at,
        })
    }

    pub async fn update_metadata(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        visibility: UserMetadataVisibility,
        metadata: Value,
    ) -> Result<UserMetadataUpdateResponse> {
        let metadata_json = validate_metadata_object(metadata)?;
        let mut user = self.get_user_in_realm(realm_id, user_id).await?;

        match visibility {
            UserMetadataVisibility::Public => user.public_metadata_json = metadata_json,
            UserMetadataVisibility::Private => user.private_metadata_json = metadata_json,
            UserMetadataVisibility::Unsafe => user.unsafe_metadata_json = metadata_json,
        }

        user.updated_at = Some(Utc::now());
        self.user_repo.update(&user, None).await?;

        Ok(UserMetadataUpdateResponse {
            public_metadata: parse_metadata_json(&user.public_metadata_json),
            private_metadata: parse_metadata_json(&user.private_metadata_json),
            unsafe_metadata: parse_metadata_json(&user.unsafe_metadata_json),
            updated_at: user.updated_at,
        })
    }

    pub async fn update_password(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        new_password: &str,
    ) -> Result<User> {
        let mut user = self.get_user_in_realm(realm_id, user_id).await?;
        let hashed_password = HashedPassword::new(new_password)?;
        user.hashed_password = hashed_password.as_str().to_string();
        user.force_password_reset = false;
        self.user_repo.update(&user, None).await?;
        Ok(user)
    }

    pub async fn update_credential_policy(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        force_password_reset: Option<bool>,
        password_login_disabled: Option<bool>,
    ) -> Result<User> {
        let mut user = self.get_user_in_realm(realm_id, user_id).await?;
        let mut changed = false;

        if let Some(value) = force_password_reset {
            if user.force_password_reset != value {
                user.force_password_reset = value;
                changed = true;
            }
        }

        if let Some(value) = password_login_disabled {
            if user.password_login_disabled != value {
                user.password_login_disabled = value;
                changed = true;
            }
        }

        if changed {
            self.user_repo.update(&user, None).await?;
        }

        Ok(user)
    }

    pub async fn lock_user(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        duration_secs: i64,
    ) -> Result<User> {
        let mut user = self.get_user_in_realm(realm_id, user_id).await?;
        let duration_secs = duration_secs.max(1);
        user.locked_until = Some(Utc::now() + chrono::Duration::seconds(duration_secs));
        user.updated_at = Some(Utc::now());
        self.user_repo.update(&user, None).await?;
        Ok(user)
    }

    pub async fn ban_user(&self, realm_id: Uuid, user_id: Uuid) -> Result<User> {
        let mut user = self.get_user_in_realm(realm_id, user_id).await?;
        user.banned_at = Some(Utc::now());
        user.updated_at = Some(Utc::now());
        self.user_repo.update(&user, None).await?;
        Ok(user)
    }

    pub async fn get_primary_email(&self, user_id: &Uuid) -> Result<Option<String>> {
        Ok(self
            .user_email_repo
            .find_primary(user_id)
            .await?
            .map(|e| e.email))
    }

    pub async fn delete_users(&self, realm_id: &Uuid, user_ids: &[Uuid]) -> Result<u64> {
        if user_ids.is_empty() {
            return Ok(0);
        }
        let count = self.user_repo.delete_users(realm_id, user_ids).await?;
        Ok(count)
    }
}

fn normalize_optional_email(email: Option<&str>) -> Option<String> {
    email
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
}

fn normalize_optional_profile_text(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn parse_metadata_json(raw: &str) -> Value {
    serde_json::from_str(raw).unwrap_or_else(|_| serde_json::json!({}))
}

pub fn admin_metadata_response(user: &User, include_private: bool) -> AdminUserMetadataResponse {
    AdminUserMetadataResponse {
        public_metadata: parse_metadata_json(&user.public_metadata_json),
        private_metadata: if include_private {
            parse_metadata_json(&user.private_metadata_json)
        } else {
            serde_json::json!({})
        },
        unsafe_metadata: parse_metadata_json(&user.unsafe_metadata_json),
    }
}

fn validate_metadata_object(metadata: Value) -> Result<String> {
    if !metadata.is_object() {
        let mut fields = HashMap::new();
        fields.insert(
            "metadata".to_string(),
            "Metadata must be a JSON object.".to_string(),
        );
        return Err(Error::FieldsValidation {
            message: "Validation failed".to_string(),
            fields,
        });
    }

    let metadata_json =
        serde_json::to_string(&metadata).map_err(|e| Error::Unexpected(e.into()))?;
    if metadata_json.len() > USER_METADATA_MAX_BYTES {
        let mut fields = HashMap::new();
        fields.insert(
            "metadata".to_string(),
            format!(
                "Metadata must be at most {} bytes.",
                USER_METADATA_MAX_BYTES
            ),
        );
        return Err(Error::FieldsValidation {
            message: "Validation failed".to_string(),
            fields,
        });
    }

    Ok(metadata_json)
}
