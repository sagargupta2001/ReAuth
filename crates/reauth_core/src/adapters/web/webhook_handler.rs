use crate::adapters::web::validation::ValidatedJson;
use crate::application::webhook_service::{
    CreateWebhookPayload, TestWebhookPayload, UpdateWebhookPayload,
    UpdateWebhookSubscriptionsPayload,
};
use crate::domain::pagination::PageRequest;
use crate::domain::telemetry::DeliveryLogQuery;
use crate::error::{Error, Result};
use crate::AppState;
use axum::extract::{Path, Query};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct DeliveryLogQueryParams {
    #[serde(flatten)]
    pub page: PageRequest,
    pub event_type: Option<String>,
    pub event_id: Option<String>,
    pub failed: Option<bool>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Deserialize)]
pub struct DisableWebhookPayload {
    pub reason: Option<String>,
}

pub async fn list_webhooks_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let endpoints = state.webhook_service.list_endpoints(realm.id).await?;
    Ok((StatusCode::OK, Json(endpoints)))
}

pub async fn get_webhook_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let endpoint = state.webhook_service.get_endpoint(realm.id, id).await?;
    Ok((StatusCode::OK, Json(endpoint)))
}

pub async fn create_webhook_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    ValidatedJson(payload): ValidatedJson<CreateWebhookPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let endpoint = state
        .webhook_service
        .create_endpoint(realm.id, payload)
        .await?;
    Ok((StatusCode::CREATED, Json(endpoint)))
}

pub async fn update_webhook_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
    Json(payload): Json<UpdateWebhookPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let endpoint = state
        .webhook_service
        .update_endpoint(realm.id, id, payload)
        .await?;
    Ok((StatusCode::OK, Json(endpoint)))
}

pub async fn delete_webhook_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state.webhook_service.delete_endpoint(realm.id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn enable_webhook_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let endpoint = state.webhook_service.enable_endpoint(realm.id, id).await?;
    Ok((StatusCode::OK, Json(endpoint)))
}

pub async fn disable_webhook_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
    Json(payload): Json<DisableWebhookPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let endpoint = state
        .webhook_service
        .disable_endpoint(realm.id, id, payload.reason)
        .await?;
    Ok((StatusCode::OK, Json(endpoint)))
}

pub async fn roll_webhook_secret_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let endpoint = state
        .webhook_service
        .roll_signing_secret(realm.id, id)
        .await?;
    Ok((StatusCode::OK, Json(endpoint)))
}

pub async fn update_webhook_subscriptions_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
    ValidatedJson(payload): ValidatedJson<UpdateWebhookSubscriptionsPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let endpoint = state
        .webhook_service
        .update_subscriptions(realm.id, id, payload)
        .await?;
    Ok((StatusCode::OK, Json(endpoint)))
}

pub async fn test_webhook_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
    Json(payload): Json<TestWebhookPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let result = state
        .webhook_service
        .test_delivery(realm.id, id, payload)
        .await?;
    Ok((StatusCode::OK, Json(result)))
}

pub async fn list_webhook_deliveries_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
    Query(params): Query<DeliveryLogQueryParams>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let query = DeliveryLogQuery {
        page: params.page,
        realm_id: Some(realm.id),
        target_type: Some("webhook".to_string()),
        target_id: Some(id.to_string()),
        event_type: params.event_type,
        event_id: params.event_id,
        failed: params.failed,
        start_time: params.start_time,
        end_time: params.end_time,
    };

    let response = state.telemetry_service.list_delivery_logs(query).await?;
    Ok((StatusCode::OK, Json(response)))
}
