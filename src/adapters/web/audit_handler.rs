use crate::error::{Error, Result};
use crate::AppState;
use axum::extract::Query;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AuditListQuery {
    pub limit: Option<usize>,
}

const DEFAULT_LIMIT: usize = 100;
const MAX_LIMIT: usize = 500;

// GET /api/realms/{realm}/audits
pub async fn list_audit_events_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Query(query): Query<AuditListQuery>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let events = state.audit_service.list_recent(realm.id, limit).await?;

    Ok((StatusCode::OK, Json(events)))
}
