use crate::{
    adapters::web::server::AppState,
    error::{Error, Result},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

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
