use crate::application::rbac_service::RbacService;
use crate::domain::session::RefreshToken;
use crate::ports::realm_repository::RealmRepository;
use crate::ports::session_repository::SessionRepository;
use crate::ports::token_service::TokenService;
use crate::{
    domain::crypto::HashedPassword,
    error::{Error, Result},
    ports::user_repository::UserRepository,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub access_token: String,
}

pub struct AuthService {
    user_repo: Arc<dyn UserRepository>,
    realm_repo: Arc<dyn RealmRepository>,
    session_repo: Arc<dyn SessionRepository>,
    token_service: Arc<dyn TokenService>,
    rbac_service: Arc<RbacService>,
    settings: crate::config::AuthConfig,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        realm_repo: Arc<dyn RealmRepository>,
        session_repo: Arc<dyn SessionRepository>,
        token_service: Arc<dyn TokenService>,
        rbac_service: Arc<RbacService>,
        settings: crate::config::AuthConfig,
    ) -> Self {
        Self {
            user_repo,
            realm_repo,
            session_repo,
            token_service,
            rbac_service,
            settings,
        }
    }

    pub async fn login(&self, payload: LoginPayload) -> Result<(LoginResponse, RefreshToken)> {
        // Find user and verify password
        let user = self
            .user_repo
            .find_by_username(&payload.username)
            .await?
            .ok_or(Error::InvalidCredentials)?;

        let hashed_password = HashedPassword::from_hash(&user.hashed_password)?;

        if !hashed_password.verify(&payload.password)? {
            return Err(Error::InvalidCredentials);
        }

        // TODO: Get realm from user. For now, use a default.
        let realm = self.realm_repo.find_by_name("default").await?.unwrap();

        // Create the Stateful Refresh Token
        let expires_at =
            chrono::Utc::now() + chrono::Duration::seconds(realm.refresh_token_ttl_secs);
        let refresh_token = RefreshToken {
            id: Uuid::new_v4(),
            user_id: user.id,
            realm_id: realm.id,
            expires_at,
        };
        self.session_repo.save(&refresh_token).await?;

        // Get user's permissions (from the service we already built)
        let permissions = self
            .rbac_service
            .get_effective_permissions(&user.id)
            .await?;

        // Create the Stateless Access Token (JWT)
        let access_token = self
            .token_service
            .create_access_token(&user, refresh_token.id, &permissions)
            .await?;

        // 5. Return the tokens
        Ok((LoginResponse { access_token }, refresh_token))
    }
}
