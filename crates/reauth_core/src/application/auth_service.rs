use crate::{
    domain::crypto::HashedPassword,
    error::{Error, Result},
    ports::user_repository::UserRepository,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

pub struct AuthService {
    user_repo: Arc<dyn UserRepository>,
}

impl AuthService {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    /// Verifies user credentials and returns the user on success.
    pub async fn login(&self, payload: LoginPayload) -> Result<String> {
        // 1. Find the user
        let user = self
            .user_repo
            .find_by_username(&payload.username)
            .await?
            .ok_or(Error::InvalidCredentials)?;

        // 2. Wrap the stored hash
        let hashed_password = HashedPassword::new(&user.hashed_password)?;

        // 3. Verify the password
        if !hashed_password.verify(&payload.password)? {
            return Err(Error::InvalidCredentials);
        }

        // 4. TODO: Create a session (This is the next step!)
        let session_token = "placeholder-session-token".to_string();

        Ok(session_token)
    }
}
