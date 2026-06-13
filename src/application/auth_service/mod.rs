use crate::application::rbac_service::RbacService;
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
use tracing::instrument;
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

/// A session row enriched with the owning user's display fields, for the admin
/// Sessions console. Flattens the full refresh token so the raw-JSON view and
/// existing fields (id, client_id, step_up_at, …) stay intact.
#[derive(Serialize)]
pub struct SessionView {
    #[serde(flatten)]
    pub token: RefreshToken,
    pub username: Option<String>,
}

pub struct AuthService {
    user_repo: Arc<dyn UserRepository>,
    realm_repo: Arc<dyn RealmRepository>,
    session_repo: Arc<dyn SessionRepository>,
    token_service: Arc<dyn TokenService>,
    rbac_service: Arc<RbacService>,
    settings: crate::config::AuthConfig,
    security: crate::config::SecurityConfig,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        realm_repo: Arc<dyn RealmRepository>,
        session_repo: Arc<dyn SessionRepository>,
        token_service: Arc<dyn TokenService>,
        rbac_service: Arc<RbacService>,
        settings: crate::config::AuthConfig,
        security: crate::config::SecurityConfig,
    ) -> Self {
        Self {
            user_repo,
            realm_repo,
            session_repo,
            token_service,
            rbac_service,
            settings,
            security,
        }
    }

    #[instrument(skip_all, fields(telemetry = "span"))]
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
            .find_by_id(&user.realm_id)
            .await?
            .ok_or_else(|| Error::RealmNotFound(user.realm_id.to_string()))?;

        // 1.5 Update last sign in
        let mut updated_user = user.clone();
        updated_user.last_sign_in_at = Some(Utc::now());
        if let Err(e) = self.user_repo.update(&updated_user, None).await {
            tracing::error!(
                "Failed to update last_sign_in_at for user {}: {}",
                user.id,
                e
            );
        }

        // 1.6 Revoke any existing active tokens for the same user+client combo
        // This prevents stale sessions from accumulating on repeated logins.
        if let Some(ref cid) = client_id {
            let _ = self
                .session_repo
                .revoke_by_user_and_client(&user.realm_id, &user.id, cid)
                .await;
        } else {
            // Root SSO token — revoke previous root tokens (client_id IS NULL)
            let _ = self
                .session_repo
                .revoke_root_tokens_for_user(&user.realm_id, &user.id)
                .await;
        }

        // 2. Create the Stateful Refresh Token
        let expires_at = Utc::now() + Duration::seconds(self.settings.refresh_token_ttl_secs);
        let now = Utc::now();
        let refresh_token = RefreshToken {
            id: Uuid::new_v4(),
            family_id: Uuid::new_v4(),
            user_id: user.id,
            realm_id: realm.id,
            client_id: client_id.clone(),
            expires_at,
            ip_address,
            user_agent,
            created_at: now,
            last_used_at: now,
            revoked_at: None,
            replaced_by: None,
            step_up_at: None,
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
    #[instrument(skip_all, fields(telemetry = "span"))]
    pub async fn validate_token_and_get_user(&self, token: &str) -> Result<User> {
        Ok(self.validate_token_and_get_session(token).await?.0)
    }

    /// Validates an access token and returns the User together with the current
    /// session id (the `sid` claim / live refresh-token id). The session id is
    /// the trusted source for caller-scoped actions like "revoke other sessions".
    #[instrument(skip_all, fields(telemetry = "span"))]
    pub async fn validate_token_and_get_session(&self, token: &str) -> Result<(User, Uuid)> {
        // 1. Validate the JWT
        let claims: AccessTokenClaims = self.token_service.validate_access_token(token).await?;

        // 2. Check if the session is still valid in the DB
        let session = self.session_repo.find_by_id(&claims.sid).await?;
        let session = match session {
            Some(session) => session,
            None => return Err(Error::SessionRevoked),
        };

        // 2b. Forced re-authentication (step-up). When immediate invalidation is
        // enabled, reject any access token issued before the step-up timestamp so
        // the user is challenged on their next request rather than at next refresh.
        if self.security.immediate_step_up_invalidation {
            if let Some(step_up_at) = session.step_up_at {
                if (claims.iat as i64) < step_up_at.timestamp() {
                    return Err(Error::ReauthRequired);
                }
            }
        }

        // 3. Fetch the user from the database
        let user = self
            .user_repo
            .find_by_id(&claims.sub)
            .await?
            .ok_or(Error::UserNotFound)?;

        Ok((user, claims.sid))
    }

    #[instrument(skip_all, fields(telemetry = "span"))]
    pub async fn refresh_session(
        &self,
        refresh_token_id: Uuid,
    ) -> Result<(LoginResponse, RefreshToken)> {
        // 1. Find the token (including revoked/replaced) to detect reuse.
        let old_token = self
            .session_repo
            .find_by_id_any(&refresh_token_id)
            .await?
            .ok_or(Error::InvalidRefreshToken)?;

        // Reject expired tokens.
        if old_token.expires_at <= Utc::now() {
            return Err(Error::InvalidRefreshToken);
        }

        // Reuse detection: if this token was already rotated or revoked, kill the family.
        if old_token.revoked_at.is_some() || old_token.replaced_by.is_some() {
            self.session_repo
                .revoke_family(&old_token.family_id)
                .await?;
            return Err(Error::SecurityViolation(
                "Refresh token reuse detected".to_string(),
            ));
        }

        // Forced re-authentication (step-up): a session marked for step-up must
        // not silently refresh. The client is forced into an interactive login
        // flow, which mints a fresh family with step_up_at cleared.
        if old_token.step_up_at.is_some() {
            return Err(Error::ReauthRequired);
        }

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
            family_id: old_token.family_id,
            user_id: user.id,
            realm_id: realm.id,
            client_id: old_token.client_id.clone(),
            expires_at,
            ip_address: old_token.ip_address.clone(),
            user_agent: old_token.user_agent.clone(),
            created_at: now,
            last_used_at: now,
            revoked_at: None,
            replaced_by: None,
            step_up_at: None,
        };
        // Mark the old token as replaced (rotation).
        self.session_repo
            .mark_replaced(&old_token.id, &new_refresh_token.id)
            .await?;
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
        self.session_repo.delete_by_id(&refresh_token_id).await
    }

    /// Logs out by revoking the cookie's token, then also revokes any tokens
    /// for the specified client so cross-origin OIDC sessions are cleaned up.
    pub async fn logout_with_client(&self, refresh_token_id: Uuid, client_id: &str) -> Result<()> {
        if let Ok(Some(token)) = self.session_repo.find_by_id_any(&refresh_token_id).await {
            let _ = self.session_repo.delete_by_id(&refresh_token_id).await;
            let _ = self
                .session_repo
                .revoke_by_user_and_client(&token.realm_id, &token.user_id, client_id)
                .await;
        } else {
            let _ = self.session_repo.delete_by_id(&refresh_token_id).await;
        }
        Ok(())
    }

    pub async fn list_sessions(
        &self,
        realm_id: Uuid,
        req: PageRequest,
    ) -> Result<PageResponse<SessionView>> {
        let page = self.session_repo.list(&realm_id, &req).await?;

        // Enrich each session with its owner's display fields. The page is
        // bounded (per_page <= 100) and users repeat across rows, so we look up
        // each distinct user at most once.
        use std::collections::hash_map::Entry;
        let mut users: std::collections::HashMap<Uuid, User> = std::collections::HashMap::new();
        for token in &page.data {
            if let Entry::Vacant(slot) = users.entry(token.user_id) {
                if let Some(user) = self.user_repo.find_by_id(&token.user_id).await? {
                    slot.insert(user);
                }
            }
        }

        let data = page
            .data
            .into_iter()
            .map(|token| {
                let user = users.get(&token.user_id);
                SessionView {
                    username: user.map(|u| u.username.clone()),
                    token,
                }
            })
            .collect();

        Ok(PageResponse {
            data,
            meta: page.meta,
        })
    }

    /// Revoke an explicit set of sessions within a realm. Any id matching
    /// `exclude` (the caller's current session) is dropped so a bulk action
    /// never logs out the caller. Returns the number of sessions revoked.
    pub async fn revoke_sessions(
        &self,
        realm_id: Uuid,
        ids: &[Uuid],
        exclude: Option<Uuid>,
    ) -> Result<u64> {
        let targets: Vec<Uuid> = ids
            .iter()
            .copied()
            .filter(|id| Some(*id) != exclude)
            .collect();
        if targets.is_empty() {
            return Ok(0);
        }
        self.session_repo.revoke_many(&realm_id, &targets).await
    }

    /// Revoke all of a user's active sessions except their current one.
    pub async fn revoke_other_sessions(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        current_sid: Uuid,
    ) -> Result<u64> {
        self.session_repo
            .revoke_others_for_user(&realm_id, &user_id, &current_sid)
            .await
    }

    /// Revoke every active session for a user in a realm (admin-wide eviction).
    pub async fn revoke_user_sessions(&self, realm_id: Uuid, user_id: Uuid) -> Result<u64> {
        self.session_repo
            .revoke_user_sessions(&realm_id, &user_id)
            .await
    }

    /// Mark a session for forced re-authentication. Returns true if a matching
    /// active session in the realm was updated.
    pub async fn request_step_up(&self, realm_id: Uuid, id: Uuid) -> Result<bool> {
        self.session_repo.request_step_up(&realm_id, &id).await
    }
}

#[cfg(test)]
mod tests;
