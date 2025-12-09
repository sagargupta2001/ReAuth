use crate::application::flow_manager::{CreateDraftRequest, UpdateDraftRequest};
use crate::domain::pagination::PageRequest;
use crate::{
    adapters::web::server::AppState,
    error::{Error, Result},
};
use axum::extract::Query;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

pub async fn list_flows_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let flows = state.flow_service.list_flows(realm.id).await?;

    Ok((StatusCode::OK, Json(flows)))
}

// --- Node Registry (The Palette) ---

/// GET /api/realms/{realm}/flows/nodes
/// Returns available node types (Authenticators, Conditionals) for the builder.
pub async fn list_nodes_handler(State(state): State<AppState>) -> Result<impl IntoResponse> {
    // In the future, we might filter nodes based on Realm settings/plugins
    let nodes = state.node_registry.get_available_nodes();
    Ok((StatusCode::OK, Json(nodes)))
}

/// GET /api/realms/{realm}/flows/drafts
/// Lists all flow drafts for the realm.
pub async fn list_drafts_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Query(req): Query<PageRequest>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let drafts = state.flow_manager.list_drafts(realm.id, req).await?;
    Ok((StatusCode::OK, Json(drafts)))
}

/// POST /api/realms/{realm}/flows/drafts
/// Creates a new flow draft (e.g. empty canvas).
pub async fn create_draft_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Json(payload): Json<CreateDraftRequest>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let draft = state.flow_manager.create_draft(realm.id, payload).await?;
    Ok((StatusCode::CREATED, Json(draft)))
}

/// GET /api/realms/{realm}/flows/drafts/{id}
/// Gets a specific draft (including the graph JSON).
pub async fn get_draft_handler(
    State(state): State<AppState>,
    Path((_realm, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let draft = state.flow_manager.get_draft(id).await?;
    Ok((StatusCode::OK, Json(draft)))
}

/// PUT /api/realms/{realm}/flows/drafts/{id}
/// Updates a draft (saves the graph JSON).
pub async fn update_draft_handler(
    State(state): State<AppState>,
    Path((_realm, id)): Path<(String, Uuid)>,
    Json(payload): Json<UpdateDraftRequest>,
) -> Result<impl IntoResponse> {
    let draft = state.flow_manager.update_draft(id, payload).await?;
    Ok((StatusCode::OK, Json(draft)))
}
