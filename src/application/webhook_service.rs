use crate::adapters::observability::telemetry_store::TelemetryDatabase;
use crate::domain::events::{EventEnvelope, EVENT_VERSION_V1};
use crate::domain::webhook::{WebhookEndpoint, WebhookSubscription};
use crate::error::{Error, Result};
use crate::ports::transaction_manager::TransactionManager;
use crate::ports::webhook_repository::WebhookRepository;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::sync::Arc;
use std::time::Instant;
use url::Url;
use uuid::Uuid;
use validator::Validate;

pub const WEBHOOK_STATUS_ACTIVE: &str = "active";
pub const WEBHOOK_STATUS_DISABLED_SYSTEM: &str = "disabled_system";
pub const WEBHOOK_STATUS_DISABLED_USER: &str = "disabled_user";

#[derive(Debug, Deserialize, Validate)]
pub struct CreateWebhookPayload {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
    #[validate(length(min = 1, message = "URL is required"))]
    pub url: String,
    pub description: Option<String>,
    pub signing_secret: Option<String>,
    #[serde(default)]
    pub custom_headers: HashMap<String, String>,
    pub http_method: Option<String>,
    #[serde(default)]
    pub subscriptions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWebhookPayload {
    pub name: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub signing_secret: Option<String>,
    pub http_method: Option<String>,
    pub status: Option<String>,
    pub custom_headers: Option<HashMap<String, String>>,
    pub subscriptions: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct WebhookSubscriptionToggle {
    #[validate(length(min = 1, message = "Event type is required"))]
    pub event_type: String,
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateWebhookSubscriptionsPayload {
    #[validate(length(min = 1, message = "At least one subscription toggle is required"))]
    #[validate(nested)]
    pub subscriptions: Vec<WebhookSubscriptionToggle>,
}

#[derive(Debug, Deserialize)]
pub struct TestWebhookPayload {
    pub event_type: Option<String>,
    pub data: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct WebhookEndpointDetails {
    pub endpoint: WebhookEndpoint,
    pub subscriptions: Vec<WebhookSubscription>,
}

#[derive(Debug, Serialize)]
pub struct WebhookTestResult {
    pub status_code: Option<i64>,
    pub response_body: Option<String>,
    pub error: Option<String>,
    pub latency_ms: i64,
}

struct TestDeliveryLogEntry<'a> {
    telemetry_db: &'a TelemetryDatabase,
    event_id: String,
    realm_id: Uuid,
    target_id: String,
    event_type: String,
    payload_json: String,
    response_status: Option<i64>,
    response_body: Option<String>,
    error: Option<String>,
    error_chain: Option<String>,
    latency_ms: i64,
}

pub struct WebhookService {
    repo: Arc<dyn WebhookRepository>,
    tx_manager: Arc<dyn TransactionManager>,
    http_client: reqwest::Client,
    telemetry_db: TelemetryDatabase,
}

impl WebhookService {
    pub fn new(
        repo: Arc<dyn WebhookRepository>,
        tx_manager: Arc<dyn TransactionManager>,
        telemetry_db: TelemetryDatabase,
    ) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("Failed to build HTTP client");
        Self {
            repo,
            tx_manager,
            http_client,
            telemetry_db,
        }
    }

    pub async fn list_endpoints(&self, realm_id: Uuid) -> Result<Vec<WebhookEndpointDetails>> {
        let endpoints = self.repo.list_endpoints(&realm_id).await?;
        let mut details = Vec::with_capacity(endpoints.len());
        for endpoint in endpoints {
            let subscriptions = self.repo.list_subscriptions(&endpoint.id).await?;
            details.push(WebhookEndpointDetails {
                endpoint,
                subscriptions,
            });
        }
        Ok(details)
    }

    pub async fn search_endpoints(
        &self,
        realm_id: Uuid,
        query: &str,
        limit: i64,
    ) -> Result<Vec<WebhookEndpoint>> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Ok(vec![]);
        }
        let bounded = limit.clamp(1, 20);
        self.repo
            .search_endpoints(&realm_id, trimmed, bounded)
            .await
    }

    pub async fn get_endpoint(&self, realm_id: Uuid, id: Uuid) -> Result<WebhookEndpointDetails> {
        let endpoint = self
            .repo
            .find_endpoint(&realm_id, &id)
            .await?
            .ok_or_else(|| Error::NotFound("Webhook endpoint not found".to_string()))?;
        let subscriptions = self.repo.list_subscriptions(&endpoint.id).await?;
        Ok(WebhookEndpointDetails {
            endpoint,
            subscriptions,
        })
    }

    pub async fn create_endpoint(
        &self,
        realm_id: Uuid,
        payload: CreateWebhookPayload,
    ) -> Result<WebhookEndpointDetails> {
        if payload.subscriptions.is_empty() {
            return Err(Error::Validation(
                "At least one event subscription is required".to_string(),
            ));
        }

        let signing_secret = payload
            .signing_secret
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let http_method = normalize_http_method(payload.http_method.as_deref())?;

        let endpoint = WebhookEndpoint {
            id: Uuid::new_v4(),
            realm_id,
            name: payload.name,
            url: payload.url,
            http_method,
            status: WEBHOOK_STATUS_ACTIVE.to_string(),
            signing_secret,
            custom_headers: payload.custom_headers,
            description: payload.description,
            consecutive_failures: 0,
            last_failure_at: None,
            disabled_at: None,
            disabled_reason: None,
            created_at: "".to_string(),
            updated_at: "".to_string(),
        };

        let mut tx = self.tx_manager.begin().await?;
        let result = async {
            self.repo.create_endpoint(&endpoint, Some(&mut *tx)).await?;
            self.repo
                .upsert_subscriptions(&endpoint.id, &payload.subscriptions, Some(&mut *tx))
                .await?;
            Ok(())
        }
        .await;

        match result {
            Ok(()) => self.tx_manager.commit(tx).await?,
            Err(err) => {
                self.tx_manager.rollback(tx).await?;
                return Err(err);
            }
        }

        self.get_endpoint(realm_id, endpoint.id).await
    }

    pub async fn update_endpoint(
        &self,
        realm_id: Uuid,
        endpoint_id: Uuid,
        payload: UpdateWebhookPayload,
    ) -> Result<WebhookEndpointDetails> {
        let mut endpoint = self
            .repo
            .find_endpoint(&realm_id, &endpoint_id)
            .await?
            .ok_or_else(|| Error::NotFound("Webhook endpoint not found".to_string()))?;
        let previous_url = endpoint.url.clone();
        let previous_name = endpoint.name.clone();

        if let Some(ref name) = payload.name {
            endpoint.name = name.to_string();
        }
        if let Some(ref url) = payload.url {
            endpoint.url = url.to_string();
        }
        if let Some(ref http_method) = payload.http_method {
            endpoint.http_method = normalize_http_method(Some(http_method))?;
        }
        if let Some(description) = payload.description {
            endpoint.description = Some(description);
        }
        if let Some(status) = payload.status {
            endpoint.status = status;
        }
        if let Some(secret) = payload.signing_secret {
            endpoint.signing_secret = secret;
        }
        if let Some(headers) = payload.custom_headers {
            endpoint.custom_headers = headers;
        }
        if payload.name.is_none()
            && payload.url.is_some()
            && should_update_name_from_url(&previous_name, &previous_url)
        {
            endpoint.name = derive_endpoint_name(&endpoint.url);
        }

        let mut tx = self.tx_manager.begin().await?;
        let result = async {
            self.repo.update_endpoint(&endpoint, Some(&mut *tx)).await?;
            if let Some(subscriptions) = payload.subscriptions {
                self.repo
                    .upsert_subscriptions(&endpoint.id, &subscriptions, Some(&mut *tx))
                    .await?;
            }
            Ok(())
        }
        .await;

        match result {
            Ok(()) => self.tx_manager.commit(tx).await?,
            Err(err) => {
                self.tx_manager.rollback(tx).await?;
                return Err(err);
            }
        }

        self.get_endpoint(realm_id, endpoint_id).await
    }

    pub async fn roll_signing_secret(
        &self,
        realm_id: Uuid,
        endpoint_id: Uuid,
    ) -> Result<WebhookEndpointDetails> {
        let mut endpoint = self
            .repo
            .find_endpoint(&realm_id, &endpoint_id)
            .await?
            .ok_or_else(|| Error::NotFound("Webhook endpoint not found".to_string()))?;

        endpoint.signing_secret = Uuid::new_v4().to_string();
        self.repo.update_endpoint(&endpoint, None).await?;

        self.get_endpoint(realm_id, endpoint_id).await
    }

    pub async fn delete_endpoint(&self, realm_id: Uuid, endpoint_id: Uuid) -> Result<()> {
        let mut tx = self.tx_manager.begin().await?;
        let result = async {
            self.repo
                .delete_endpoint(&realm_id, &endpoint_id, Some(&mut *tx))
                .await?;
            Ok(())
        }
        .await;

        match result {
            Ok(()) => self.tx_manager.commit(tx).await?,
            Err(err) => {
                self.tx_manager.rollback(tx).await?;
                return Err(err);
            }
        }

        Ok(())
    }

    pub async fn enable_endpoint(
        &self,
        realm_id: Uuid,
        endpoint_id: Uuid,
    ) -> Result<WebhookEndpointDetails> {
        let mut tx = self.tx_manager.begin().await?;
        let result = async {
            self.repo
                .set_endpoint_status(
                    &realm_id,
                    &endpoint_id,
                    WEBHOOK_STATUS_ACTIVE,
                    None,
                    Some(&mut *tx),
                )
                .await?;
            Ok(())
        }
        .await;

        match result {
            Ok(()) => self.tx_manager.commit(tx).await?,
            Err(err) => {
                self.tx_manager.rollback(tx).await?;
                return Err(err);
            }
        }

        self.get_endpoint(realm_id, endpoint_id).await
    }

    pub async fn disable_endpoint(
        &self,
        realm_id: Uuid,
        endpoint_id: Uuid,
        reason: Option<String>,
    ) -> Result<WebhookEndpointDetails> {
        let mut tx = self.tx_manager.begin().await?;
        let result = async {
            self.repo
                .set_endpoint_status(
                    &realm_id,
                    &endpoint_id,
                    WEBHOOK_STATUS_DISABLED_USER,
                    reason.as_deref(),
                    Some(&mut *tx),
                )
                .await?;
            Ok(())
        }
        .await;

        match result {
            Ok(()) => self.tx_manager.commit(tx).await?,
            Err(err) => {
                self.tx_manager.rollback(tx).await?;
                return Err(err);
            }
        }

        self.get_endpoint(realm_id, endpoint_id).await
    }

    pub async fn update_subscriptions(
        &self,
        realm_id: Uuid,
        endpoint_id: Uuid,
        payload: UpdateWebhookSubscriptionsPayload,
    ) -> Result<WebhookEndpointDetails> {
        let _ = self
            .repo
            .find_endpoint(&realm_id, &endpoint_id)
            .await?
            .ok_or_else(|| Error::NotFound("Webhook endpoint not found".to_string()))?;

        let mut tx = self.tx_manager.begin().await?;
        let result = async {
            for subscription in &payload.subscriptions {
                self.repo
                    .set_subscription_enabled(
                        &endpoint_id,
                        &subscription.event_type,
                        subscription.enabled,
                        Some(&mut *tx),
                    )
                    .await?;
            }
            Ok(())
        }
        .await;

        match result {
            Ok(()) => self.tx_manager.commit(tx).await?,
            Err(err) => {
                self.tx_manager.rollback(tx).await?;
                return Err(err);
            }
        }

        self.get_endpoint(realm_id, endpoint_id).await
    }

    pub async fn test_delivery(
        &self,
        realm_id: Uuid,
        endpoint_id: Uuid,
        payload: TestWebhookPayload,
    ) -> Result<WebhookTestResult> {
        let endpoint = self
            .repo
            .find_endpoint(&realm_id, &endpoint_id)
            .await?
            .ok_or_else(|| Error::NotFound("Webhook endpoint not found".to_string()))?;

        let event_id = Uuid::new_v4();
        let event_type = payload
            .event_type
            .unwrap_or_else(|| "webhook.test".to_string());
        let envelope = EventEnvelope {
            event_id: event_id.to_string(),
            event_type: event_type.clone(),
            event_version: EVENT_VERSION_V1.to_string(),
            occurred_at: Utc::now().to_rfc3339(),
            realm_id: Some(realm_id),
            actor: None,
            data: payload.data.unwrap_or_else(|| {
                serde_json::json!({
                    "message": "ReAuth webhook test",
                })
            }),
        };
        let payload_json = serde_json::to_string(&envelope).unwrap_or_else(|_| "{}".to_string());

        let signature = sign_payload(&endpoint.signing_secret, &payload_json);
        let start = Instant::now();
        let method = parse_http_method(&endpoint.http_method);
        let mut request = self
            .http_client
            .request(method, &endpoint.url)
            .header("Content-Type", "application/json")
            .header("Reauth-Event-Id", event_id.to_string())
            .header("Reauth-Event-Type", event_type.clone())
            .header("Reauth-Event-Version", EVENT_VERSION_V1)
            .header("Reauth-Signature", signature);

        for (key, value) in &endpoint.custom_headers {
            request = request.header(key, value);
        }

        let response = request.body(payload_json.clone()).send().await;

        let latency_ms = start.elapsed().as_millis() as i64;
        let (status_code, response_body, error, error_chain) = match response {
            Ok(resp) => {
                let status = resp.status().as_u16() as i64;
                let body = resp.text().await.unwrap_or_default();
                (Some(status), Some(body), None, None)
            }
            Err(err) => {
                let error = err.to_string();
                let error_chain = collect_error_chain(&err);
                (None, None, Some(error), serialize_error_chain(&error_chain))
            }
        };

        let log_entry = TestDeliveryLogEntry {
            telemetry_db: &self.telemetry_db,
            event_id: event_id.to_string(),
            realm_id,
            target_id: endpoint_id.to_string(),
            event_type: event_type.clone(),
            payload_json: payload_json.clone(),
            response_status: status_code,
            response_body: response_body.clone(),
            error: error.clone(),
            error_chain: error_chain.clone(),
            latency_ms,
        };
        log_test_delivery(log_entry)
            .await
            .map_err(Error::Unexpected)?;

        if let Some(err) = error.clone() {
            return Err(Error::System(format!(
                "Webhook test delivery failed: {}",
                err
            )));
        }

        Ok(WebhookTestResult {
            status_code,
            response_body,
            error,
            latency_ms,
        })
    }
}

async fn log_test_delivery(entry: TestDeliveryLogEntry<'_>) -> anyhow::Result<()> {
    let delivery_id = Uuid::new_v4().to_string();
    let delivered_at = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO delivery_logs (
            id, event_id, realm_id, target_type, target_id, event_type, event_version, attempt,
            payload, payload_compressed, response_status, response_body, error, error_chain, latency_ms, delivered_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(delivery_id)
    .bind(entry.event_id)
    .bind(entry.realm_id.to_string())
    .bind("webhook")
    .bind(entry.target_id)
    .bind(entry.event_type)
    .bind(EVENT_VERSION_V1)
    .bind(1_i64)
    .bind(entry.payload_json)
    .bind(false)
    .bind(entry.response_status)
    .bind(entry.response_body)
    .bind(entry.error)
    .bind(entry.error_chain)
    .bind(entry.latency_ms)
    .bind(delivered_at)
    .execute(&**entry.telemetry_db)
    .await?;

    Ok(())
}

fn collect_error_chain(error: &reqwest::Error) -> Vec<String> {
    let mut chain = Vec::new();
    let mut current: Option<&(dyn StdError + 'static)> = Some(error);
    while let Some(err) = current {
        chain.push(err.to_string());
        current = err.source();
    }
    chain
}

fn serialize_error_chain(chain: &[String]) -> Option<String> {
    if chain.is_empty() {
        return None;
    }
    serde_json::to_string(chain).ok()
}

fn normalize_http_method(method: Option<&str>) -> Result<String> {
    let normalized = method.unwrap_or("POST").trim().to_uppercase();
    match normalized.as_str() {
        "POST" | "PUT" => Ok(normalized),
        _ => Err(Error::Validation(
            "Unsupported webhook HTTP method. Use POST or PUT.".to_string(),
        )),
    }
}

fn parse_http_method(method: &str) -> reqwest::Method {
    reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::POST)
}

fn should_update_name_from_url(name: &str, url: &str) -> bool {
    let derived = derive_endpoint_name(url);
    name.eq_ignore_ascii_case(&derived) || name.eq_ignore_ascii_case(url)
}

fn derive_endpoint_name(url: &str) -> String {
    if let Ok(parsed) = Url::parse(url) {
        if let Some(host) = parsed.host_str() {
            return host.to_string();
        }
    }

    let trimmed = url.trim();
    if trimmed.is_empty() {
        return url.to_string();
    }

    let with_scheme = format!("https://{}", trimmed);
    if let Ok(parsed) = Url::parse(&with_scheme) {
        if let Some(host) = parsed.host_str() {
            return host.to_string();
        }
    }

    url.to_string()
}

fn sign_payload(secret: &str, payload: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .unwrap_or_else(|_| Hmac::<Sha256>::new_from_slice(b"").unwrap());
    mac.update(payload.as_bytes());
    let result = mac.finalize().into_bytes();
    hex::encode(result)
}
