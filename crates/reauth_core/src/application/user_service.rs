use std::sync::Arc;
use uuid::Uuid;

use crate::domain::crypto::HashedPassword;
use crate::domain::events::{DomainEvent, UserCreated};
use crate::domain::pagination::{PageRequest, PageResponse};
use crate::ports::event_bus::EventPublisher;
use crate::{
    domain::user::User,
    error::{Error, Result},
    ports::user_repository::UserRepository,
};

/// A service that handles user-related application logic.
/// It depends on the `UserRepository` port, not a concrete database implementation.
pub struct UserService {
    user_repo: Arc<dyn UserRepository>,
    event_bus: Arc<dyn EventPublisher>,
}

impl UserService {
    pub fn new(user_repo: Arc<dyn UserRepository>, event_bus: Arc<dyn EventPublisher>) -> Self {
        Self {
            user_repo,
            event_bus,
        }
    }

    pub async fn create_user(&self, username: &str, password: &str) -> Result<User> {
        // Business logic: check if user already exists
        if self.user_repo.find_by_username(username).await?.is_some() {
            return Err(Error::UserAlreadyExists);
        }

        let hashed_password = HashedPassword::new(password)?;

        let user = User {
            id: Uuid::new_v4(),
            username: username.to_string(),
            hashed_password: hashed_password.as_str().to_string(),
        };

        self.user_repo.save(&user).await?;

        self.event_bus
            .publish(DomainEvent::UserCreated(UserCreated {
                user_id: user.id,
                username: user.username.clone(),
            }))
            .await;

        Ok(user)
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        self.user_repo.find_by_username(username).await
    }

    pub async fn list_users(&self, req: PageRequest) -> Result<PageResponse<User>> {
        self.user_repo.list(&req).await
    }

    pub async fn get_user(&self, id: Uuid) -> Result<User> {
        self.user_repo
            .find_by_id(&id)
            .await?
            .ok_or(Error::UserNotFound)
    }

    pub async fn update_username(&self, id: Uuid, new_username: String) -> Result<User> {
        let mut user = self.get_user(id).await?;
        // Check uniqueness if changed
        if user.username != new_username {
            if self
                .user_repo
                .find_by_username(&new_username)
                .await?
                .is_some()
            {
                return Err(Error::UserAlreadyExists);
            }
            user.username = new_username;
            self.user_repo.update(&user).await?;
        }
        Ok(user)
    }
}
