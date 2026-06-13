use crate::{
    adapters::web::auth_middleware::{AuthUser, CurrentSessionId},
    domain::{
        audit::NewAuditEvent, pagination::PageRequest, permissions, session::SessionListFilter,
        user::User,
    },
    error::{Error, Result},
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    Extension,
};
use chrono::{DateTime, Days, NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};
use tracing::error;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ListSessionsQuery {
    #[serde(flatten)]
    pub page: PageRequest,
    /// `filter_started={"from":"YYYY-MM-DD","to":"YYYY-MM-DD"}` — range on `created_at`.
    #[serde(default, deserialize_with = "deserialize_optional_date_range_filter")]
    pub filter_started: Option<SessionListFilter>,
}

pub async fn list_sessions_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Query(query): Query<ListSessionsQuery>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let filter = query.filter_started.unwrap_or_default();
    let response = state
        .auth_service
        .list_sessions(realm.id, query.page, filter)
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

#[derive(Deserialize)]
struct DateRangeQueryParam {
    from: Option<String>,
    to: Option<String>,
}

fn deserialize_optional_date_range_filter<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<SessionListFilter>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    let Some(value) = value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };

    let Ok(range) = serde_json::from_str::<DateRangeQueryParam>(&value) else {
        return Ok(None);
    };

    let filter = SessionListFilter {
        started_from: range.from.as_deref().and_then(parse_filter_start),
        started_to_exclusive: range.to.as_deref().and_then(parse_filter_end_exclusive),
    };

    Ok((!filter.is_empty()).then_some(filter))
}

fn parse_filter_start(value: &str) -> Option<DateTime<Utc>> {
    parse_date_only(value)
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .map(|datetime| Utc.from_utc_datetime(&datetime))
        .or_else(|| parse_rfc3339(value))
}

fn parse_filter_end_exclusive(value: &str) -> Option<DateTime<Utc>> {
    parse_date_only(value)
        .and_then(|date| date.checked_add_days(Days::new(1)))
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .map(|datetime| Utc.from_utc_datetime(&datetime))
        .or_else(|| parse_rfc3339(value))
}

fn parse_date_only(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").ok()
}

fn parse_rfc3339(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value.trim())
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

pub async fn revoke_session_handler(
    State(state): State<AppState>,
    Extension(AuthUser(actor)): Extension<AuthUser>,
    Extension(CurrentSessionId(current_sid)): Extension<CurrentSessionId>,
    Path((realm_name, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    // The current session is never revoked from this surface; use a global
    // logout flow for that instead.
    if id == current_sid {
        return Err(Error::Validation(
            "Cannot revoke your current session from this surface.".to_string(),
        ));
    }

    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state.auth_service.logout(id).await?;
    record_session_audit(
        &state,
        realm.id,
        actor.id,
        "session.revoke",
        Some(id.to_string()),
        json!({}),
    )
    .await;

    Ok((StatusCode::NO_CONTENT, ()))
}

/// Tagged payload for bulk and global session revocation.
#[derive(Deserialize)]
#[serde(tag = "scope", rename_all = "snake_case")]
pub enum RevokeSessionsPayload {
    /// Revoke an explicit set of sessions (the caller's current session is excluded).
    Selected { session_ids: Vec<Uuid> },
    /// Revoke all of the caller's active sessions except the current one.
    Others,
    /// Revoke a user's entire account sessions (requires `user:write`).
    User { user_id: Uuid },
}

#[derive(Serialize)]
pub struct RevokeCountResponse {
    pub count: u64,
}

pub async fn revoke_sessions_handler(
    State(state): State<AppState>,
    Extension(AuthUser(actor)): Extension<AuthUser>,
    Extension(CurrentSessionId(current_sid)): Extension<CurrentSessionId>,
    Path(realm_name): Path<String>,
    Json(payload): Json<RevokeSessionsPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let (count, action, target_id, metadata): (u64, &str, Option<String>, Value) = match payload {
        RevokeSessionsPayload::Selected { session_ids } => {
            if session_ids.is_empty() {
                return Err(Error::Validation(
                    "session_ids must not be empty.".to_string(),
                ));
            }
            let requested = session_ids.len();
            let count = state
                .auth_service
                .revoke_sessions(realm.id, &session_ids, Some(current_sid))
                .await?;
            (
                count,
                "session.revoke_bulk",
                None,
                json!({ "count": count, "requested": requested }),
            )
        }
        RevokeSessionsPayload::Others => {
            let count = state
                .auth_service
                .revoke_other_sessions(realm.id, actor.id, current_sid)
                .await?;
            (
                count,
                "session.revoke_others",
                None,
                json!({ "count": count }),
            )
        }
        RevokeSessionsPayload::User { user_id } => {
            // Mass account eviction crosses into the user lifecycle and so
            // additionally requires user:write.
            require_user_write(&state, &actor).await?;
            let count = state
                .auth_service
                .revoke_user_sessions(realm.id, user_id)
                .await?;
            (
                count,
                "session.revoke_user",
                Some(user_id.to_string()),
                json!({ "user_id": user_id, "count": count }),
            )
        }
    };

    record_session_audit(&state, realm.id, actor.id, action, target_id, metadata).await;

    Ok((StatusCode::OK, Json(RevokeCountResponse { count })))
}

pub async fn step_up_session_handler(
    State(state): State<AppState>,
    Extension(AuthUser(actor)): Extension<AuthUser>,
    Extension(CurrentSessionId(current_sid)): Extension<CurrentSessionId>,
    Path((realm_name, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    if id == current_sid {
        return Err(Error::Validation(
            "Cannot force re-authentication on your current session.".to_string(),
        ));
    }

    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let updated = state.auth_service.request_step_up(realm.id, id).await?;
    if !updated {
        return Err(Error::NotFound("Session not found.".to_string()));
    }

    record_session_audit(
        &state,
        realm.id,
        actor.id,
        "session.step_up",
        Some(id.to_string()),
        json!({}),
    )
    .await;

    Ok((StatusCode::NO_CONTENT, ()))
}

async fn require_user_write(state: &AppState, actor: &User) -> Result<()> {
    if state
        .rbac_service
        .user_has_permission(&actor.id, permissions::USER_WRITE)
        .await?
    {
        Ok(())
    } else {
        Err(Error::SecurityViolation(
            "user:write is required to revoke a user's account sessions.".to_string(),
        ))
    }
}

async fn record_session_audit(
    state: &AppState,
    realm_id: Uuid,
    actor_id: Uuid,
    action: &str,
    target_id: Option<String>,
    metadata: Value,
) {
    let event = NewAuditEvent {
        realm_id,
        actor_user_id: Some(actor_id),
        action: action.to_string(),
        target_type: "session".to_string(),
        target_id,
        metadata,
    };

    if let Err(err) = state.audit_service.record(event).await {
        error!("Failed to write session audit event: {:?}", err);
    }
}
