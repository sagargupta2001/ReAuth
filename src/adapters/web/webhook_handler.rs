use crate::adapters::web::validation::ValidatedJson;
use crate::application::webhook_service::{
    CreateWebhookPayload, TestWebhookPayload, UpdateWebhookPayload,
    UpdateWebhookSubscriptionsPayload,
};
use crate::domain::pagination::{PageRequest, PageResponse, SortDirection};
use crate::domain::telemetry::{DeliveryLogQuery, EventRoutingMetrics};
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

#[derive(Deserialize)]
pub struct WebhookListQuery {
    #[serde(flatten)]
    pub page: PageRequest,
}

#[derive(Deserialize)]
pub struct EventRoutingMetricsQuery {
    pub window_hours: Option<i64>,
}

pub async fn list_webhooks_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Query(query): Query<WebhookListQuery>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let mut endpoints = state.webhook_service.list_endpoints(realm.id).await?;
    if let Some(query) = query.page.q.as_ref().map(|value| value.to_lowercase()) {
        endpoints.retain(|details| {
            details.endpoint.name.to_lowercase().contains(&query)
                || details.endpoint.url.to_lowercase().contains(&query)
                || details.endpoint.http_method.to_lowercase().contains(&query)
        });
    }

    let sort_by = query.page.sort_by.as_deref().unwrap_or("updated_at");
    let sort_dir = query.page.sort_dir.unwrap_or(SortDirection::Desc);
    endpoints.sort_by(|a, b| {
        let ordering = match sort_by {
            "name" => a.endpoint.name.cmp(&b.endpoint.name),
            "url" => a.endpoint.url.cmp(&b.endpoint.url),
            "status" => a.endpoint.status.cmp(&b.endpoint.status),
            "http_method" => a.endpoint.http_method.cmp(&b.endpoint.http_method),
            "created_at" => a.endpoint.created_at.cmp(&b.endpoint.created_at),
            "updated_at" => a.endpoint.updated_at.cmp(&b.endpoint.updated_at),
            _ => a.endpoint.updated_at.cmp(&b.endpoint.updated_at),
        };
        match sort_dir {
            SortDirection::Asc => ordering,
            SortDirection::Desc => ordering.reverse(),
        }
    });

    let total = endpoints.len() as i64;
    let per_page = query.page.per_page.clamp(1, 100);
    let page = query.page.page.max(1);
    let start = ((page - 1) * per_page) as usize;
    let data = if start >= endpoints.len() {
        Vec::new()
    } else {
        endpoints
            .into_iter()
            .skip(start)
            .take(per_page as usize)
            .collect()
    };

    Ok((
        StatusCode::OK,
        Json(PageResponse::new(data, total, page, per_page)),
    ))
}

pub async fn event_routing_metrics_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Query(query): Query<EventRoutingMetricsQuery>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let window_hours = query.window_hours.unwrap_or(24).clamp(1, 168);
    let metrics = state
        .telemetry_service
        .get_delivery_metrics(Some(realm.id), window_hours)
        .await?;
    let total_routed = metrics.total_routed;
    let success_rate = if total_routed > 0 {
        metrics.success_count as f64 / total_routed as f64
    } else {
        0.0
    };

    let response = EventRoutingMetrics {
        window_hours,
        total_routed,
        success_rate,
        avg_latency_ms: metrics.avg_latency_ms,
    };

    Ok((StatusCode::OK, Json(response)))
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
