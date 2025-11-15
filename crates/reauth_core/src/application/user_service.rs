use std::sync::Arc;
use uuid::Uuid;

use crate::domain::crypto::HashedPassword;
use crate::domain::events::{DomainEvent, UserCreated};
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
}
