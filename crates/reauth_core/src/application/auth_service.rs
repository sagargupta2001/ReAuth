use crate::application::rbac_service::RbacService;
use crate::constants::DEFAULT_REALM_NAME;
use crate::domain::pagination::{PageRequest, PageResponse};
use crate::domain::session::RefreshToken;
use crate::domain::user::User;
use crate::ports::realm_repository::RealmRepository;
use crate::ports::session_repository::SessionRepository;
use crate::ports::token_service::{AccessTokenClaims, TokenService};
use crate::{
    error::{Error, Result},
    ports::user_repository::UserRepository,
};
use chrono::{Duration, Utc};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
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

    pub async fn create_session(
        &self,
        user: &User,
        client_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(LoginResponse, RefreshToken)> {
        // 1. Get realm from user. For now, use the default.
        let realm = self
            .realm_repo
            .find_by_name(DEFAULT_REALM_NAME)
            .await?
            .ok_or_else(|| Error::RealmNotFound(DEFAULT_REALM_NAME.to_string()))?;

        // 2. Create the Stateful Refresh Token
        let expires_at = Utc::now() + Duration::seconds(self.settings.refresh_token_ttl_secs);
        let now = Utc::now();
        let refresh_token = RefreshToken {
            id: Uuid::new_v4(),
            user_id: user.id,
            realm_id: realm.id,
            client_id: client_id.clone(),
            expires_at,
            ip_address,
            user_agent,
            created_at: now,
            last_used_at: now,
        };
        self.session_repo.save(&refresh_token).await?;

        // 3. Get user's fresh permissions
        let permissions = self
            .rbac_service
            .get_effective_permissions(&user.id)
            .await?;
        let (roles, groups) = self
            .rbac_service
            .get_user_roles_and_groups(&user.id)
            .await?;

        // 4. Create the Stateless Access Token (JWT)
        let access_token = self
            .token_service
            .create_access_token(user, refresh_token.id, &permissions, &roles, &groups)
            .await?;

        let mut id_token = None;
        if let Some(cid) = client_id {
            id_token = Some(
                self.token_service
                    .create_id_token(user, &cid, &groups)
                    .await?,
            );
        }

        Ok((
            LoginResponse {
                access_token,
                id_token,
            },
            refresh_token,
        ))
    }

    /// Validates an roles token and returns the full User.
    /// This is the core "use case" for the auth middleware.
    pub async fn validate_token_and_get_user(&self, token: &str) -> Result<User> {
        // 1. Validate the JWT
        let claims: AccessTokenClaims = self.token_service.validate_access_token(token).await?;

        // 2. Check if the session is still valid in the DB
        let session_is_valid = self.session_repo.find_by_id(&claims.sid).await?.is_some();

        if !session_is_valid {
            return Err(Error::SessionRevoked);
        }

        // 3. Fetch the user from the database
        let user = self
            .user_repo
            .find_by_id(&claims.sub)
            .await?
            .ok_or(Error::UserNotFound)?;

        Ok(user)
    }

    pub async fn refresh_session(
        &self,
        refresh_token_id: Uuid,
    ) -> Result<(LoginResponse, RefreshToken)> {
        // 1. Find and *immediately delete* the old refresh token.
        // This is the core of "token rotation." The token is now single-use.
        // We find and delete in one operation if possible, or two separate calls.
        // Let's assume `find_by_id` checks expiry.
        let old_token = self
            .session_repo
            .find_by_id(&refresh_token_id)
            .await?
            .ok_or(Error::InvalidRefreshToken)?;

        // If this fails (because another thread deleted it 1ms ago),
        // the whole function returns Err(InvalidRefreshToken).
        // The subsequent code (create new token) will NEVER run.
        self.session_repo.delete_by_id(&old_token.id).await?;

        // 2. Get the associated user and realm
        let user = self
            .user_repo
            .find_by_id(&old_token.user_id)
            .await?
            .ok_or(Error::UserNotFound)?; // The user was deleted

        let realm = self
            .realm_repo
            .find_by_id(&old_token.realm_id)
            .await?
            .ok_or(Error::RealmNotFound("".to_string()))?; // The realm was deleted

        // 3. Create a NEW Refresh Token
        let expires_at = Utc::now() + Duration::seconds(self.settings.refresh_token_ttl_secs);
        let now = Utc::now();
        let new_refresh_token = RefreshToken {
            id: Uuid::new_v4(), // New ID
            user_id: user.id,
            realm_id: realm.id,
            client_id: old_token.client_id.clone(),
            expires_at,
            ip_address: old_token.ip_address.clone(),
            user_agent: old_token.user_agent.clone(),
            created_at: now,
            last_used_at: now,
        };
        self.session_repo.save(&new_refresh_token).await?;

        // 4. Get *fresh* permissions (this is critical for RBAC)
        let permissions = self
            .rbac_service
            .get_effective_permissions(&user.id)
            .await?;
        let (roles, groups) = self
            .rbac_service
            .get_user_roles_and_groups(&user.id)
            .await?;

        // 5. Create a new Access Token (JWT) linked to the *new* session
        let access_token = self
            .token_service
            .create_access_token(&user, new_refresh_token.id, &permissions, &roles, &groups)
            .await?;

        let mut id_token = None;
        if let Some(cid) = &new_refresh_token.client_id {
            id_token = Some(
                self.token_service
                    .create_id_token(&user, cid, &groups)
                    .await?,
            );
        }

        Ok((
            LoginResponse {
                access_token,
                id_token,
            },
            new_refresh_token,
        ))
    }

    /// Logs out a user by deleting their specific refresh token session.
    pub async fn logout(&self, refresh_token_id: Uuid) -> Result<()> {
        // We delete the token from the database.
        // If it fails (e.g., database down), we return the error.
        // If it doesn't exist (already deleted), the repo usually returns Ok(()).
        self.session_repo.delete_by_id(&refresh_token_id).await
    }

    pub async fn list_sessions(
        &self,
        realm_id: Uuid,
        req: PageRequest,
    ) -> Result<PageResponse<RefreshToken>> {
        self.session_repo.list(&realm_id, &req).await
    }
}

#[cfg(test)]
mod auth_service_tests;
