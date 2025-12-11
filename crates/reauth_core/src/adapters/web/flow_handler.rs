use crate::application::flow_manager::{CreateDraftRequest, UpdateDraftRequest};
use crate::domain::flow::FlowDraft;
use crate::domain::pagination::PageRequest;
use crate::{
    error::{Error, Result},
    AppState,
};
use axum::extract::Query;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

// A unified DTO for the frontend
#[derive(serde::Serialize)]
pub struct UnifiedFlowDto {
    pub id: Uuid,
    pub alias: String,
    pub description: Option<String>,
    pub r#type: String,
    pub built_in: bool,
    pub is_draft: bool,
}

pub async fn list_flows_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    // 1. Fetch Runtime Flows (The "Live" truth)
    let runtime_flows = state.flow_service.list_flows(realm.id).await?;

    // 2. Fetch Drafts (The "Work in Progress" truth)
    let drafts = state.flow_manager.list_all_drafts(realm.id).await?;

    // 3. Merge Strategy: Use a Map to deduplicate by ID
    let mut flows_map: HashMap<Uuid, UnifiedFlowDto> = HashMap::new();

    // Step A: Populate with Runtime flows first
    for flow in runtime_flows {
        flows_map.insert(
            flow.id,
            UnifiedFlowDto {
                id: flow.id,
                alias: flow.alias,
                description: flow.description,
                r#type: flow.r#type,
                built_in: flow.built_in,
                is_draft: false, // Will be updated if a draft is found
            },
        );
    }

    // Step B: Overlay Drafts
    for draft in drafts {
        flows_map
            .entry(draft.id)
            .and_modify(|existing| {
                // If it exists, it means we have a draft OF a runtime flow.
                // We update the name/desc to match the draft (latest version),
                // but keep 'built_in' from the existing entry.
                existing.alias = draft.name.clone();
                existing.description = draft.description.clone();
                existing.is_draft = true;
            })
            .or_insert_with(|| {
                // If it doesn't exist, it's a pure draft (new custom flow)
                UnifiedFlowDto {
                    id: draft.id,
                    alias: draft.name,
                    description: draft.description,
                    r#type: draft.flow_type,
                    built_in: false, // Drafts are never "built-in" until published
                    is_draft: true,
                }
            });
    }

    // 4. Convert to List & Sort
    let mut unified_list: Vec<UnifiedFlowDto> = flows_map.into_values().collect();

    // Sort: Built-in first, then by Name
    unified_list.sort_by(|a, b| {
        match (a.built_in, b.built_in) {
            (true, false) => std::cmp::Ordering::Less, // Built-in comes first
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.alias.cmp(&b.alias), // Alphabetical otherwise
        }
    });

    Ok((StatusCode::OK, Json(unified_list)))
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

#[derive(serde::Serialize)]
pub struct FlowDraftResponse {
    #[serde(flatten)]
    pub draft: FlowDraft,
    pub active_version: Option<i32>,
    pub built_in: bool,
}

/// GET /api/realms/{realm}/flows/drafts/{id}
/// Gets a specific draft (including the graph JSON).
pub async fn get_draft_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let draft = state.flow_manager.get_draft(id).await?;

    let active_version = state
        .flow_manager
        .get_deployed_version(&realm.id, &draft.flow_type, &id)
        .await?;

    let built_in = state.flow_manager.is_flow_built_in(&id).await?;

    // 4. Return the enriched response
    Ok((
        StatusCode::OK,
        Json(FlowDraftResponse {
            draft,
            active_version,
            built_in,
        }),
    ))
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

#[derive(serde::Deserialize)]
pub struct PublishFlowRequest {
    // Empty for now, ID is in path
}

pub async fn publish_flow_handler(
    State(state): State<AppState>,
    Path((realm_name, flow_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let version = state.flow_manager.publish_flow(realm.id, flow_id).await?;

    Ok((StatusCode::CREATED, Json(version)))
}

/// GET /api/realms/{realm}/flows/{id}/versions
pub async fn list_versions_handler(
    State(state): State<AppState>,
    Path((_realm, flow_id)): Path<(String, Uuid)>,
    Query(req): Query<PageRequest>,
) -> Result<impl IntoResponse> {
    let response = state.flow_manager.list_flow_versions(flow_id, req).await?;
    Ok((StatusCode::OK, Json(response)))
}

#[derive(serde::Deserialize)]
pub struct RollbackRequest {
    pub version_number: i32,
}

/// POST /api/realms/{realm}/flows/{id}/rollback
pub async fn rollback_flow_handler(
    State(state): State<AppState>,
    Path((realm_name, flow_id)): Path<(String, Uuid)>,
    Json(payload): Json<RollbackRequest>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state
        .flow_manager
        .rollback_flow(realm.id, flow_id, payload.version_number)
        .await?;

    Ok((StatusCode::OK, Json(json!({ "success": true }))))
}

#[derive(serde::Deserialize)]
pub struct RestoreDraftRequest {
    pub version_number: i32,
}

/// POST /api/realms/{realm}/flows/{id}/restore-draft
pub async fn restore_draft_handler(
    State(state): State<AppState>,
    Path((realm_name, flow_id)): Path<(String, Uuid)>,
    Json(payload): Json<RestoreDraftRequest>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state
        .flow_manager
        .restore_draft_from_version(realm.id, flow_id, payload.version_number)
        .await?;

    Ok((StatusCode::OK, Json(json!({ "success": true }))))
}
