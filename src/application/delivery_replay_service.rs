use crate::application::telemetry_service::TelemetryService;
use crate::application::webhook_service::WebhookService;
use crate::domain::telemetry::DeliveryLog;
use crate::error::{Error, Result};
use crate::ports::http_client::{HttpDeliveryClient, HttpDeliveryRequest};
use crate::ports::telemetry_repository::TelemetryRepository;
use crate::ports::webhook_repository::WebhookRepository;
use chrono::Utc;
use serde::Serialize;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

const MAX_CONSECUTIVE_FAILURES: i64 = 10;

#[derive(Debug, Serialize)]
pub struct ReplayDeliveryResult {
    pub delivery_id: String,
    pub target_type: String,
    pub target_id: String,
    pub response_status: Option<i64>,
    pub error: Option<String>,
    pub latency_ms: Option<i64>,
}

pub struct DeliveryReplayService {
    telemetry_service: Arc<TelemetryService>,
    webhook_service: Arc<WebhookService>,
    telemetry_repo: Arc<dyn TelemetryRepository>,
    webhook_repo: Arc<dyn WebhookRepository>,
    http_client: Arc<dyn HttpDeliveryClient>,
}

impl DeliveryReplayService {
    pub fn new(
        telemetry_service: Arc<TelemetryService>,
        webhook_service: Arc<WebhookService>,
        telemetry_repo: Arc<dyn TelemetryRepository>,
        webhook_repo: Arc<dyn WebhookRepository>,
        http_client: Arc<dyn HttpDeliveryClient>,
    ) -> Self {
        Self {
            telemetry_service,
            webhook_service,
            telemetry_repo,
            webhook_repo,
            http_client,
        }
    }

    pub async fn replay_delivery(&self, delivery_id: &str) -> Result<ReplayDeliveryResult> {
        let Some(log) = self.telemetry_service.get_delivery_log(delivery_id).await? else {
            return Err(Error::NotFound("Delivery log not found".to_string()));
        };

        match log.target_type.as_str() {
            "webhook" => self.replay_webhook(log).await,
            _ => Err(Error::Validation(format!(
                "Unsupported target type {}",
                log.target_type
            ))),
        }
    }

    async fn replay_webhook(&self, log: DeliveryLog) -> Result<ReplayDeliveryResult> {
        if log.payload_compressed {
            return Err(Error::Validation(
                "Compressed payload replay not supported".to_string(),
            ));
        }

        let realm_id = log
            .realm_id
            .ok_or_else(|| Error::Validation("Missing realm_id".to_string()))?;
        let endpoint_id = Uuid::parse_str(&log.target_id)?;
        let details = self
            .webhook_service
            .get_endpoint(realm_id, endpoint_id)
            .await?;
        let endpoint = details.endpoint;

        let signature = sign_payload(&endpoint.signing_secret, &log.payload);
        let start = Instant::now();

        let mut headers = std::collections::HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Reauth-Event-Id".to_string(), log.event_id.clone());
        headers.insert("Reauth-Event-Type".to_string(), log.event_type.clone());
        headers.insert(
            "Reauth-Event-Version".to_string(),
            log.event_version.clone(),
        );
        headers.insert("Reauth-Signature".to_string(), signature);

        for (key, value) in &endpoint.custom_headers {
            headers.insert(key.clone(), value.clone());
        }

        let request = HttpDeliveryRequest {
            method: endpoint.http_method.clone(),
            url: endpoint.url.clone(),
            headers,
            body: log.payload.clone(),
        };

        let response = self.http_client.send(request).await;
        let latency_ms = start.elapsed().as_millis() as i64;

        match response {
            Ok(resp) => {
                let status_code = resp.status_code as i64;
                let body = resp.body;
                let is_success = (200..300).contains(&status_code);
                let error = if is_success {
                    None
                } else {
                    Some(format!("http_{}", status_code))
                };

                let new_delivery_id = self
                    .insert_delivery_log(
                        &log,
                        Some(status_code),
                        Some(body.clone()),
                        error.clone(),
                        None,
                        latency_ms,
                    )
                    .await?;

                if !is_success {
                    self.webhook_repo
                        .record_webhook_failure(
                            &endpoint_id,
                            error.as_deref().unwrap_or(""),
                            MAX_CONSECUTIVE_FAILURES,
                        )
                        .await?;
                } else {
                    self.webhook_repo
                        .record_webhook_success(&endpoint_id)
                        .await?;
                }

                Ok(ReplayDeliveryResult {
                    delivery_id: new_delivery_id,
                    target_type: log.target_type,
                    target_id: log.target_id,
                    response_status: Some(status_code),
                    error,
                    latency_ms: Some(latency_ms),
                })
            }
            Err(err) => {
                let error = err.message.clone();
                let error_chain = err.error_chain.clone();
                let new_delivery_id = self
                    .insert_delivery_log(
                        &log,
                        None,
                        None,
                        Some(error.clone()),
                        serialize_error_chain(&error_chain),
                        latency_ms,
                    )
                    .await?;

                self.webhook_repo
                    .record_webhook_failure(&endpoint_id, &error, MAX_CONSECUTIVE_FAILURES)
                    .await?;

                Ok(ReplayDeliveryResult {
                    delivery_id: new_delivery_id,
                    target_type: log.target_type,
                    target_id: log.target_id,
                    response_status: None,
                    error: Some(error),
                    latency_ms: Some(latency_ms),
                })
            }
        }
    }

    async fn insert_delivery_log(
        &self,
        log: &DeliveryLog,
        response_status: Option<i64>,
        response_body: Option<String>,
        error: Option<String>,
        error_chain: Option<String>,
        latency_ms: i64,
    ) -> Result<String> {
        let delivered_at = Utc::now().to_rfc3339();
        let delivery_id = Uuid::new_v4().to_string();
        let attempt = log.attempt + 1;

        let new_log = DeliveryLog {
            id: delivery_id.clone(),
            event_id: log.event_id.clone(),
            realm_id: log.realm_id,
            target_type: log.target_type.clone(),
            target_id: log.target_id.clone(),
            event_type: log.event_type.clone(),
            event_version: log.event_version.clone(),
            attempt,
            payload: log.payload.clone(),
            payload_compressed: log.payload_compressed,
            response_status,
            response_body,
            error,
            error_chain,
            latency_ms: Some(latency_ms),
            delivered_at,
        };

        self.telemetry_repo.insert_delivery_log(&new_log).await?;

        Ok(delivery_id)
    }
}

fn serialize_error_chain(chain: &[String]) -> Option<String> {
    if chain.is_empty() {
        return None;
    }
    serde_json::to_string(chain).ok()
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
