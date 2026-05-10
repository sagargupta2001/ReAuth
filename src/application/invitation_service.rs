use std::sync::Arc;

use base64::Engine;
use chrono::{Duration, Utc};
use rand::distr::{Alphanumeric, SampleString};
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::application::flow_executor::FlowExecutor;
use crate::application::user_service::UserService;
use crate::domain::auth_session::AuthenticationSession;
use crate::domain::execution::{ExecutionPlan, ExecutionResult};
use crate::domain::invitation::{Invitation, InvitationStatus};
use crate::domain::pagination::{PageRequest, PageResponse};
use crate::error::{Error, Result};
use crate::ports::auth_session_repository::AuthSessionRepository;
use crate::ports::flow_store::FlowStore;
use crate::ports::invitation_repository::InvitationRepository;
use crate::ports::realm_repository::RealmRepository;
use tracing::warn;

pub struct InvitationService {
    invitation_repo: Arc<dyn InvitationRepository>,
    realm_repo: Arc<dyn RealmRepository>,
    auth_session_repo: Arc<dyn AuthSessionRepository>,
    flow_store: Arc<dyn FlowStore>,
    flow_executor: Arc<FlowExecutor>,
    user_service: Arc<UserService>,
}

impl InvitationService {
    pub fn new(
        invitation_repo: Arc<dyn InvitationRepository>,
        realm_repo: Arc<dyn RealmRepository>,
        auth_session_repo: Arc<dyn AuthSessionRepository>,
        flow_store: Arc<dyn FlowStore>,
        flow_executor: Arc<FlowExecutor>,
        user_service: Arc<UserService>,
    ) -> Self {
        Self {
            invitation_repo,
            realm_repo,
            auth_session_repo,
            flow_store,
            flow_executor,
            user_service,
        }
    }

    pub async fn create_invitation(
        &self,
        realm_id: Uuid,
        email: &str,
        expiry_days: i64,
        invited_by_user_id: Option<Uuid>,
    ) -> Result<Invitation> {
        if expiry_days < 1 {
            return Err(Error::Validation(
                "expiry_days must be greater than or equal to 1".to_string(),
            ));
        }

        self.expire_pending(realm_id).await?;

        let normalized_email = normalize_email(email)?;

        if self
            .user_service
            .find_by_email(&realm_id, &normalized_email)
            .await?
            .is_some()
        {
            return Err(Error::Validation(
                "A user with this email already exists in this realm".to_string(),
            ));
        }

        if self
            .invitation_repo
            .find_pending_by_email(&realm_id, &normalized_email)
            .await?
            .is_some()
        {
            return Err(Error::Validation(
                "An active invitation already exists for this email".to_string(),
            ));
        }

        let now = Utc::now();
        let expires_at = now + Duration::days(expiry_days);
        let mut invitation = Invitation {
            id: Uuid::new_v4(),
            realm_id,
            email: normalized_email.clone(),
            email_normalized: normalized_email,
            status: InvitationStatus::Pending,
            token_hash: hash_token(&generate_invitation_token()),
            expiry_days,
            expires_at,
            invited_by_user_id,
            accepted_user_id: None,
            accepted_at: None,
            revoked_at: None,
            resend_count: 0,
            last_sent_at: None,
            created_at: now,
            updated_at: now,
        };

        let new_token_hash = self.dispatch_invitation_action(&invitation).await?;
        invitation.token_hash = new_token_hash;
        invitation.last_sent_at = Some(Utc::now());
        invitation.updated_at = Utc::now();

        self.invitation_repo.create(&invitation, None).await?;
        Ok(invitation)
    }

    pub async fn list_invitations(
        &self,
        realm_id: Uuid,
        req: PageRequest,
        statuses: Vec<InvitationStatus>,
    ) -> Result<PageResponse<Invitation>> {
        self.expire_pending(realm_id).await?;
        self.invitation_repo.list(&realm_id, &req, &statuses).await
    }

    pub async fn resend_invitation(
        &self,
        realm_id: Uuid,
        invitation_id: Uuid,
    ) -> Result<Invitation> {
        self.expire_pending(realm_id).await?;

        let Some(mut invitation) = self
            .invitation_repo
            .find_by_id(&realm_id, &invitation_id)
            .await?
        else {
            return Err(Error::NotFound("Invitation not found".to_string()));
        };

        if invitation.status != InvitationStatus::Pending || invitation.is_expired() {
            return Err(Error::Validation(
                "Only pending non-expired invitations can be resent".to_string(),
            ));
        }

        let realm = self
            .realm_repo
            .find_by_id(&realm_id)
            .await?
            .ok_or_else(|| Error::RealmNotFound(realm_id.to_string()))?;
        if invitation.resend_count >= realm.invitation_resend_limit {
            return Err(Error::Validation(
                "Invitation resend limit reached for this realm".to_string(),
            ));
        }

        let token_hash = self.dispatch_invitation_action(&invitation).await?;

        invitation.token_hash = token_hash;
        invitation.resend_count += 1;
        invitation.last_sent_at = Some(Utc::now());
        invitation.updated_at = Utc::now();
        self.invitation_repo.update(&invitation, None).await?;

        Ok(invitation)
    }

    pub async fn revoke_invitation(
        &self,
        realm_id: Uuid,
        invitation_id: Uuid,
    ) -> Result<Invitation> {
        self.expire_pending(realm_id).await?;

        let Some(mut invitation) = self
            .invitation_repo
            .find_by_id(&realm_id, &invitation_id)
            .await?
        else {
            return Err(Error::NotFound("Invitation not found".to_string()));
        };

        if invitation.status != InvitationStatus::Pending {
            return Err(Error::Validation(
                "Only pending invitations can be revoked".to_string(),
            ));
        }

        invitation.status = InvitationStatus::Revoked;
        invitation.revoked_at = Some(Utc::now());
        invitation.updated_at = Utc::now();
        self.invitation_repo.update(&invitation, None).await?;

        Ok(invitation)
    }

    pub async fn accept_invitation(
        &self,
        realm_id: Uuid,
        token: &str,
        username: &str,
        password: &str,
    ) -> Result<()> {
        self.expire_pending(realm_id).await?;

        let token_hash = hash_token(token);
        let Some(mut invitation) = self.invitation_repo.find_by_token_hash(&token_hash).await?
        else {
            return Err(Error::InvalidActionToken);
        };

        if invitation.realm_id != realm_id {
            return Err(Error::SecurityViolation(
                "Invitation token does not belong to this realm".to_string(),
            ));
        }
        if invitation.status != InvitationStatus::Pending || invitation.is_expired() {
            return Err(Error::InvalidActionToken);
        }

        let (initial, session_id) = self.flow_executor.resume_action(realm_id, token).await?;
        let final_result = match initial {
            ExecutionResult::Challenge { .. } | ExecutionResult::AwaitingAction { .. } => {
                self.flow_executor
                    .execute(
                        session_id,
                        Some(json!({
                            "username": username,
                            "password": password,
                            "email": invitation.email_normalized.clone(),
                        })),
                    )
                    .await?
            }
            other => other,
        };

        match final_result {
            ExecutionResult::Success { .. } => {}
            ExecutionResult::Challenge { context, .. }
            | ExecutionResult::AwaitingAction { context, .. } => {
                let message = context
                    .get("error")
                    .and_then(|value| value.as_str())
                    .unwrap_or("Invitation acceptance requires additional flow steps");
                return Err(Error::Validation(message.to_string()));
            }
            ExecutionResult::Failure { reason } => {
                return Err(Error::Validation(reason));
            }
            ExecutionResult::Continue => {
                return Err(Error::System(
                    "Invitation flow did not reach a terminal state".to_string(),
                ));
            }
        }

        let final_session = self
            .auth_session_repo
            .find_by_id(&session_id)
            .await?
            .ok_or_else(|| {
                Error::System("Invitation session missing after execution".to_string())
            })?;
        let user_id = final_session
            .user_id
            .ok_or_else(|| Error::System("Invitation flow completed without user".to_string()))?;
        let user = self.user_service.get_user(user_id).await?;

        invitation.status = InvitationStatus::Accepted;
        invitation.accepted_user_id = Some(user.id);
        invitation.accepted_at = Some(Utc::now());
        invitation.updated_at = Utc::now();
        self.invitation_repo.update(&invitation, None).await?;

        if let Err(err) = self
            .flow_executor
            .consume_action_token(realm_id, token)
            .await
        {
            warn!(
                "Failed to mark invitation action token consumed for invitation {}: {}",
                invitation.id, err
            );
        }

        Ok(())
    }

    async fn expire_pending(&self, realm_id: Uuid) -> Result<()> {
        let _ = self
            .invitation_repo
            .expire_pending_before(&realm_id, Utc::now())
            .await?;
        Ok(())
    }

    async fn dispatch_invitation_action(&self, invitation: &Invitation) -> Result<String> {
        let (version_id, plan) = self
            .resolve_invitation_flow_version(invitation.realm_id)
            .await?;

        let mut session =
            AuthenticationSession::new(invitation.realm_id, version_id, plan.start_node_id);
        session.context = json!({
            "invitation_id": invitation.id.to_string(),
            "invitation_email": invitation.email_normalized.clone(),
            "invitation_token_hash": invitation.token_hash.clone(),
            "invitation_expires_at": invitation.expires_at.to_rfc3339(),
            "invitation_expiry_days": invitation.expiry_days,
        });
        self.auth_session_repo.create(&session).await?;

        let result = self.flow_executor.execute(session.id, None).await?;
        let context = match result {
            ExecutionResult::AwaitingAction { context, .. } => context,
            ExecutionResult::Failure { reason } => {
                return Err(Error::Validation(format!(
                    "Invitation flow failed before issuing invitation email: {}",
                    reason
                )));
            }
            ExecutionResult::Challenge { screen_id, .. } => {
                return Err(Error::Validation(format!(
                    "Invitation flow reached interactive step '{}' before issuing invitation email. Publish an invitation flow with 'Issue Invitation Token' before interactive steps.",
                    screen_id
                )));
            }
            ExecutionResult::Success { .. } | ExecutionResult::Continue => {
                return Err(Error::System(
                    "Invitation flow did not suspend for async delivery".to_string(),
                ));
            }
        };

        let delivered = context
            .get("delivery")
            .and_then(|value| value.as_str())
            .is_some_and(|value| value == "email");
        if !delivered {
            return Err(Error::Validation(
                "Invitation email delivery is not configured for this realm".to_string(),
            ));
        }

        let token = context
            .get("resume_token")
            .and_then(|value| value.as_str())
            .ok_or_else(|| {
                Error::System("Invitation flow did not return a resume token".to_string())
            })?;

        Ok(hash_token(token))
    }

    async fn resolve_invitation_flow_version(
        &self,
        realm_id: Uuid,
    ) -> Result<(Uuid, ExecutionPlan)> {
        let realm = self
            .realm_repo
            .find_by_id(&realm_id)
            .await?
            .ok_or_else(|| Error::RealmNotFound(realm_id.to_string()))?;
        let flow_id = realm
            .invitation_flow_id
            .as_deref()
            .ok_or_else(|| Error::Validation("Realm has no invitation flow configured".to_string()))
            .and_then(|value| {
                Uuid::parse_str(value).map_err(|_| Error::System("Invalid Flow ID".into()))
            })?;

        let version_num = self
            .flow_store
            .get_deployed_version_number(&realm_id, "invitation", &flow_id)
            .await?
            .ok_or(Error::System("Invitation flow is not deployed".into()))?;
        let version = self
            .flow_store
            .get_version_by_number(&flow_id, version_num)
            .await?
            .ok_or(Error::System("Invitation flow version not found".into()))?;
        let version_id = Uuid::parse_str(&version.id).unwrap_or_default();
        let plan: ExecutionPlan = serde_json::from_str(&version.execution_artifact)
            .map_err(|e| Error::System(format!("Corrupt execution artifact: {}", e)))?;
        Ok((version_id, plan))
    }
}

fn normalize_email(email: &str) -> Result<String> {
    let value = email.trim().to_lowercase();
    if value.is_empty() || !value.contains('@') {
        return Err(Error::Validation("Email address is invalid".to_string()));
    }
    Ok(value)
}

fn generate_invitation_token() -> String {
    Alphanumeric.sample_string(&mut rand::rng(), 48)
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let result = hasher.finalize();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(result)
}
