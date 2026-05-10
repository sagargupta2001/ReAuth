use axum::extract::{Path, Query, State};
use axum::{http::StatusCode, response::IntoResponse, Extension, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::adapters::web::auth_middleware::AuthUser;
use crate::adapters::web::validation::ValidatedJson;
use crate::domain::invitation::{Invitation, InvitationStatus};
use crate::domain::pagination::PageRequest;
use crate::error::{Error, Result};
use crate::AppState;

#[derive(Deserialize, Validate)]
pub struct CreateInvitationPayload {
    #[validate(email(message = "Email address is invalid"))]
    pub email: String,
    #[validate(range(min = 1, max = 365, message = "Expiry must be between 1 and 365 days"))]
    pub expiry_days: i64,
}

#[derive(Deserialize)]
pub struct ListInvitationsQuery {
    #[serde(flatten)]
    pub page: PageRequest,
    pub status: Option<String>,
}

impl ListInvitationsQuery {
    fn statuses(&self) -> Vec<InvitationStatus> {
        self.status
            .as_deref()
            .into_iter()
            .flat_map(|value| value.split(','))
            .filter_map(parse_invitation_status)
            .collect()
    }
}

fn parse_invitation_status(value: &str) -> Option<InvitationStatus> {
    match value.trim().to_lowercase().as_str() {
        "pending" => Some(InvitationStatus::Pending),
        "accepted" => Some(InvitationStatus::Accepted),
        "expired" => Some(InvitationStatus::Expired),
        "revoked" => Some(InvitationStatus::Revoked),
        _ => None,
    }
}

#[derive(Deserialize, Validate)]
pub struct AcceptInvitationPayload {
    #[validate(length(min = 1, message = "Token is required"))]
    pub token: String,
    #[validate(length(min = 3, message = "Username must be at least 3 characters long"))]
    pub username: String,
    #[validate(length(
        min = 8,
        max = 100,
        message = "Password must be between 8 and 100 characters"
    ))]
    pub password: String,
}

#[derive(Serialize)]
pub struct InvitationResponse {
    pub id: Uuid,
    pub email: String,
    pub status: InvitationStatus,
    pub expiry_days: i64,
    pub expires_at: DateTime<Utc>,
    pub resend_count: i64,
    pub last_sent_at: Option<DateTime<Utc>>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct AcceptInvitationResponse {
    pub status: &'static str,
    pub url: String,
}

impl From<Invitation> for InvitationResponse {
    fn from(value: Invitation) -> Self {
        Self {
            id: value.id,
            email: value.email,
            status: value.status,
            expiry_days: value.expiry_days,
            expires_at: value.expires_at,
            resend_count: value.resend_count,
            last_sent_at: value.last_sent_at,
            accepted_at: value.accepted_at,
            revoked_at: value.revoked_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

pub async fn create_invitation_handler(
    State(state): State<AppState>,
    Extension(AuthUser(auth_user)): Extension<AuthUser>,
    Path(realm_name): Path<String>,
    ValidatedJson(payload): ValidatedJson<CreateInvitationPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let invitation = state
        .invitation_service
        .create_invitation(
            realm.id,
            &payload.email,
            payload.expiry_days,
            Some(auth_user.id),
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(InvitationResponse::from(invitation)),
    ))
}

pub async fn list_invitations_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Query(query): Query<ListInvitationsQuery>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let statuses = query.statuses();
    let response = state
        .invitation_service
        .list_invitations(realm.id, query.page, statuses)
        .await?;

    let data = response
        .data
        .into_iter()
        .map(InvitationResponse::from)
        .collect::<Vec<_>>();
    let mapped = crate::domain::pagination::PageResponse {
        data,
        meta: response.meta,
    };

    Ok((StatusCode::OK, Json(mapped)))
}

pub async fn resend_invitation_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let invitation = state
        .invitation_service
        .resend_invitation(realm.id, id)
        .await?;

    Ok((StatusCode::OK, Json(InvitationResponse::from(invitation))))
}

pub async fn revoke_invitation_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let invitation = state
        .invitation_service
        .revoke_invitation(realm.id, id)
        .await?;

    Ok((StatusCode::OK, Json(InvitationResponse::from(invitation))))
}

pub async fn accept_invitation_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    ValidatedJson(payload): ValidatedJson<AcceptInvitationPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name.clone()))?;

    state
        .invitation_service
        .accept_invitation(
            realm.id,
            &payload.token,
            &payload.username,
            &payload.password,
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(AcceptInvitationResponse {
            status: "redirect",
            url: format!("/#/login?realm={}&invited=1", realm_name),
        }),
    ))
}
