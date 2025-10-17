use crate::{
    domain::user::User,
    error::{Error, Result},
    ports::user_repository::UserRepository,
};
use std::sync::Arc;
use uuid::Uuid;

/// A service that handles user-related application logic.
/// It depends on the `UserRepository` port, not a concrete database implementation.
pub struct UserService {
    user_repo: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    pub async fn create_user(&self, username: &str, role: &str) -> Result<User> {
        // Business logic: check if user already exists
        if self.user_repo.find_by_username(username).await?.is_some() {
            return Err(Error::UserAlreadyExists);
        }

        let user = User {
            id: Uuid::new_v4().to_string(),
            username: username.to_string(),
            role: role.to_string(),
        };

        self.user_repo.save(&user).await?;
        Ok(user)
    }
}