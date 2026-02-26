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

        let hashed_password = HashedPassword::new(password)?;

        let user = User {
            id: Uuid::new_v4(),
            realm_id,
            username: username.to_string(),
            hashed_password: hashed_password.as_str().to_string(),
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
        let mut user = self.get_user_in_realm(realm_id, user_id).await?;

        // Check uniqueness if changed
        if user.username != new_username {
            if self
                .user_repo
                .find_by_username(&realm_id, &new_username)
                .await?
                .is_some()
            {
                return Err(Error::UserAlreadyExists);
            }
            user.username = new_username;
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
}
