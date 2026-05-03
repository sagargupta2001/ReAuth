use std::sync::Arc;
use uuid::Uuid;

use crate::domain::crypto::HashedPassword;
use crate::domain::events::{DomainEvent, UserCreated};
use crate::domain::pagination::{PageRequest, PageResponse};
use crate::ports::event_bus::EventPublisher;
use crate::ports::outbox_repository::OutboxRepository;
use crate::ports::transaction_manager::TransactionManager;
use crate::{
    domain::user::User,
    error::{Error, Result},
    ports::user_repository::UserRepository,
};
use chrono::Utc;

/// A service that handles user-related application logic.
/// It depends on the `UserRepository` port, not a concrete database implementation.
pub struct UserService {
    user_repo: Arc<dyn UserRepository>,
    event_bus: Arc<dyn EventPublisher>,
    outbox_repo: Arc<dyn OutboxRepository>,
    tx_manager: Arc<dyn TransactionManager>,
}

impl UserService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        event_bus: Arc<dyn EventPublisher>,
        outbox_repo: Arc<dyn OutboxRepository>,
        tx_manager: Arc<dyn TransactionManager>,
    ) -> Self {
        Self {
            user_repo,
            event_bus,
            outbox_repo,
            tx_manager,
        }
    }

    pub async fn create_user(
        &self,
        realm_id: Uuid,
        username: &str,
        password: &str,
        email: Option<&str>,
        ignore_password_policies: bool,
    ) -> Result<User> {
        // Check uniqueness WITHIN the realm
        if self
            .user_repo
            .find_by_username(&realm_id, username)
            .await?
            .is_some()
        {
            return Err(Error::UserAlreadyExists);
        }

        let normalized_email = normalize_optional_email(email);
        if let Some(email_value) = normalized_email.as_deref() {
            if self
                .user_repo
                .find_by_email(&realm_id, email_value)
                .await?
                .is_some()
            {
                return Err(Error::UserAlreadyExists);
            }
        }

        let hashed_password = HashedPassword::new(password)?;

        let user = User {
            id: Uuid::new_v4(),
            realm_id,
            username: username.to_string(),
            email: normalized_email,
            hashed_password: hashed_password.as_str().to_string(),
            force_password_reset: !ignore_password_policies,
            password_login_disabled: false,
            created_at: Some(Utc::now()),
            last_sign_in_at: None,
        };

        let mut tx = self.tx_manager.begin().await?;

        let event = DomainEvent::UserCreated(UserCreated {
            user_id: user.id,
            username: user.username.clone(),
        });
        let result = async {
            self.user_repo.save(&user, Some(&mut *tx)).await?;
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

    pub async fn list_users(&self, realm_id: Uuid, req: PageRequest) -> Result<PageResponse<User>> {
        self.user_repo.list(&realm_id, &req).await
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
        self.update_profile(realm_id, user_id, Some(new_username), None)
            .await
    }

    pub async fn update_profile(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        new_username: Option<String>,
        new_email: Option<Option<String>>,
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

        if let Some(email_update) = new_email {
            let normalized_email = normalize_optional_email(email_update.as_deref());
            if user.email != normalized_email {
                if let Some(email_value) = normalized_email.as_deref() {
                    if let Some(existing) =
                        self.user_repo.find_by_email(&realm_id, email_value).await?
                    {
                        if existing.id != user.id {
                            return Err(Error::UserAlreadyExists);
                        }
                    }
                }
                user.email = normalized_email;
                changed = true;
            }
        }

        if changed {
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
            // We return "UserNotFound" instead of "Forbidden" to prevent
            // leaking information about users in other realms.
            return Err(Error::UserNotFound);
        }

        Ok(user)
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
    pub async fn delete_users(&self, realm_id: &Uuid, user_ids: &[Uuid]) -> Result<u64> {
        if user_ids.is_empty() {
            return Ok(0);
        }

        // Ideally we should verify all users belong to the realm first,
        // but the repository method `delete_users` already scopes the deletion
        // with `WHERE realm_id = ?`. If a user ID doesn't belong to the realm,
        // it simply won't be deleted.
        
        let count = self.user_repo.delete_users(realm_id, user_ids).await?;
        
        // TODO: Emit UserDeleted events for outbox/audit log if needed
        
        Ok(count)
    }
}

fn normalize_optional_email(email: Option<&str>) -> Option<String> {
    email
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
